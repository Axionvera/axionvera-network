use crate::error::{NetworkError, Result};
use crate::signing::Signer;
use async_trait::async_trait;
use aws_config::meta::region::RegionProviderChain;
use aws_config::{BehaviorVersion, ConfigLoader};
use aws_sdk_kms::primitives::Blob;
use aws_sdk_kms::Client as KmsClient;
use aws_smithy_runtime::client::http::hyper_014::HyperClientBuilder;
use aws_smithy_runtime_api::client::http::SharedHttpClient;
use aws_types::region::Region;
use ed25519_dalek::PublicKey;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, error, info, instrument, warn};

/// AWS KMS signer configuration
#[derive(Debug, Clone)]
pub struct AwsKmsConfig {
    pub key_id: String,
    pub region: String,
    pub profile: Option<String>,
    pub endpoint_url: Option<String>,
    pub timeout_ms: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for AwsKmsConfig {
    fn default() -> Self {
        Self {
            key_id: String::new(),
            region: "us-east-1".to_string(),
            profile: None,
            endpoint_url: None,
            timeout_ms: 30000, // 30 seconds default timeout
            max_retries: 3,
            retry_delay_ms: 1000, // 1 second default retry delay
        }
    }
}

/// AWS KMS signer implementation
pub struct AwsKmsSigner {
    client: KmsClient,
    config: AwsKmsConfig,
    http_client: SharedHttpClient,
}

impl AwsKmsSigner {
    /// Create a new AWS KMS signer
    pub async fn new(key_id: String, region: String, profile: Option<String>) -> Result<Self> {
        let config = AwsKmsConfig {
            key_id,
            region,
            profile,
            ..Default::default()
        };
        
        Self::with_config(config).await
    }
    
    /// Create a new AWS KMS signer with custom configuration
    pub async fn with_config(config: AwsKmsConfig) -> Result<Self> {
        info!("Initializing AWS KMS signer with key_id: {}", config.key_id);
        
        // Create custom HTTP client with timeout configuration
        let http_client = HyperClientBuilder::new()
            .hyper_builder(
                hyper::Client::builder()
                    .pool_idle_timeout(Duration::from_secs(30))
                    .pool_max_idle_per_host(10)
                    .http2_keep_alive_interval(Duration::from_secs(30))
                    .http2_keep_alive_timeout(Duration::from_secs(10))
            )
            .build_https()
            .map_err(|e| NetworkError::Kms(format!("Failed to create HTTP client: {}", e)))?;
        
        let shared_http_client = SharedHttpClient::new(http_client);
        
        // Configure AWS SDK
        let region_provider = RegionProviderChain::first_try(Some(Region::new(config.region.clone())))
            .or_default_provider();
        
        let mut config_loader = aws_config::defaults(BehaviorVersion::latest())
            .region(region_provider)
            .http_client(shared_http_client.clone());
        
        // Set profile if provided
        if let Some(profile) = &config.profile {
            config_loader = config_loader.profile_name(profile);
        }
        
        // Set custom endpoint if provided (useful for testing with LocalStack)
        if let Some(endpoint_url) = &config.endpoint_url {
            config_loader = config_loader.endpoint_url(endpoint_url);
        }
        
        let sdk_config = config_loader
            .load()
            .await
            .map_err(|e| NetworkError::Kms(format!("Failed to load AWS config: {}", e)))?;
        
        // Create KMS client
        let client = aws_sdk_kms::Client::new(&sdk_config);
        
        let signer = Self {
            client,
            config,
            http_client: shared_http_client,
        };
        
        // Test the connection by performing a health check
        signer.health_check().await?;
        
        info!("AWS KMS signer initialized successfully");
        Ok(signer)
    }
    
    /// Sign a message with retry logic and error handling
    async fn sign_with_retry(&self, message: &[u8]) -> Result<Vec<u8>> {
        let mut last_error = None;
        
        for attempt in 1..=self.config.max_retries {
            match self.attempt_sign(message).await {
                Ok(signature) => {
                    if attempt > 1 {
                        info!("AWS KMS signing succeeded on attempt {}", attempt);
                    }
                    return Ok(signature);
                }
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    // Check if we should retry
                    if !self.should_retry(&e) || attempt == self.config.max_retries {
                        break;
                    }
                    
                    warn!(
                        "AWS KMS signing attempt {} failed, retrying in {}ms: {}",
                        attempt, self.config.retry_delay_ms, e
                    );
                    
                    tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| NetworkError::Kms("All retry attempts failed".to_string())))
    }
    
    /// Attempt to sign a message (single attempt)
    async fn attempt_sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        // Hash the message first (KMS expects the hash for signing operations)
        let mut hasher = Sha256::new();
        hasher.update(message);
        let message_hash = hasher.finalize();
        
        debug!("Signing message hash: {:x}", message_hash);
        
        // Create signing request
        let sign_request = self.client
            .sign()
            .key_id(&self.config.key_id)
            .message(Blob::new(message_hash))
            .signing_algorithm(aws_sdk_kms::types::SigningAlgorithmSpec::EcdsaSha256)
            .message_type(aws_sdk_kms::types::MessageType::Digest);
        
        // Execute with timeout
        let sign_response = timeout(
            Duration::from_millis(self.config.timeout_ms),
            sign_request.send()
        )
        .await
        .map_err(|_| NetworkError::KmsTimeout(format!("Signing operation timed out after {}ms", self.config.timeout_ms)))?
        .map_err(|e| self.handle_kms_error(e))?;
        
        // Extract signature
        let signature_bytes = sign_response
            .signature()
            .ok_or_else(|| NetworkError::Kms("No signature returned from KMS".to_string()))?
            .as_ref()
            .to_vec();
        
        debug!("Successfully signed message, signature length: {} bytes", signature_bytes.len());
        Ok(signature_bytes)
    }
    
    /// Get public key with retry logic
    async fn get_public_key_with_retry(&self) -> Result<PublicKey> {
        let mut last_error = None;
        
        for attempt in 1..=self.config.max_retries {
            match self.attempt_get_public_key().await {
                Ok(public_key) => {
                    if attempt > 1 {
                        info!("AWS KMS public key retrieval succeeded on attempt {}", attempt);
                    }
                    return Ok(public_key);
                }
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    // Check if we should retry
                    if !self.should_retry(&e) || attempt == self.config.max_retries {
                        break;
                    }
                    
                    warn!(
                        "AWS KMS public key retrieval attempt {} failed, retrying in {}ms: {}",
                        attempt, self.config.retry_delay_ms, e
                    );
                    
                    tokio::time::sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| NetworkError::Kms("All retry attempts failed".to_string())))
    }
    
    /// Attempt to get public key (single attempt)
    async fn attempt_get_public_key(&self) -> Result<PublicKey> {
        debug!("Retrieving public key from AWS KMS");
        
        // Create get public key request
        let get_public_key_request = self.client
            .get_public_key()
            .key_id(&self.config.key_id);
        
        // Execute with timeout
        let response = timeout(
            Duration::from_millis(self.config.timeout_ms),
            get_public_key_request.send()
        )
        .await
        .map_err(|_| NetworkError::KmsTimeout(format!("Public key retrieval timed out after {}ms", self.config.timeout_ms)))?
        .map_err(|e| self.handle_kms_error(e))?;
        
        // Extract public key bytes
        let public_key_bytes = response
            .public_key()
            .ok_or_else(|| NetworkError::Kms("No public key returned from KMS".to_string()))?
            .as_ref()
            .to_vec();
        
        // Convert to ed25519 public key
        let public_key = PublicKey::from_bytes(&public_key_bytes)
            .map_err(|e| NetworkError::Kms(format!("Invalid public key format: {}", e)))?;
        
        debug!("Successfully retrieved public key from AWS KMS");
        Ok(public_key)
    }
    
    /// Determine if an error is retryable
    fn should_retry(&self, error: &NetworkError) -> bool {
        match error {
            NetworkError::KmsTimeout(_) => true,
            NetworkError::Kms(ref msg) if msg.contains("Throttling") => true,
            NetworkError::Kms(ref msg) if msg.contains("Rate exceeded") => true,
            NetworkError::Kms(ref msg) if msg.contains("Internal failure") => true,
            NetworkError::Kms(ref msg) if msg.contains("Service unavailable") => true,
            NetworkError::Kms(ref msg) if msg.contains("Request timeout") => true,
            _ => false,
        }
    }
    
    /// Handle AWS KMS specific errors and convert them to NetworkError
    fn handle_kms_error(&self, error: aws_sdk_kms::Error) -> NetworkError {
        match error {
            aws_sdk_kms::Error::NotFoundException(msg) => {
                NetworkError::Kms(format!("KMS key not found: {}", msg))
            }
            aws_sdk_kms::Error::DisabledException(msg) => {
                NetworkError::Kms(format!("KMS key is disabled: {}", msg))
            }
            aws_sdk_kms::Error::KeyUnavailableException(msg) => {
                NetworkError::Kms(format!("KMS key is unavailable: {}", msg))
            }
            aws_sdk_kms::Error::InvalidKeyUsageException(msg) => {
                NetworkError::Kms(format!("Invalid key usage for signing: {}", msg))
            }
            aws_sdk_kms::Error::InvalidGrantTokenException(msg) => {
                NetworkError::Kms(format!("Invalid grant token: {}", msg))
            }
            aws_sdk_kms::Error::KmsInternalException(msg) => {
                NetworkError::Kms(format!("KMS internal error: {}", msg))
            }
            aws_sdk_kms::Error::KmsInvalidStateException(msg) => {
                NetworkError::Kms(format!("KMS invalid state: {}", msg))
            }
            aws_sdk_kms::Error::DependencyTimeoutException(msg) => {
                NetworkError::KmsTimeout(format!("KMS dependency timeout: {}", msg))
            }
            aws_sdk_kms::Error::LimitExceededException(msg) => {
                NetworkError::KmsRateLimit(format!("KMS rate limit exceeded: {}", msg))
            }
            _ => NetworkError::Kms(format!("KMS error: {}", error)),
        }
    }
}

#[async_trait]
impl Signer for AwsKmsSigner {
    async fn get_public_key(&self) -> Result<PublicKey> {
        self.get_public_key_with_retry().await
    }
    
    async fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        self.sign_with_retry(message).await
    }
    
    async fn get_key_id(&self) -> Result<String> {
        Ok(self.config.key_id.clone())
    }
    
    async fn health_check(&self) -> Result<bool> {
        debug!("Performing health check for AWS KMS signer");
        
        match self.attempt_get_public_key().await {
            Ok(_) => {
                debug!("AWS KMS signer health check passed");
                Ok(true)
            }
            Err(e) => {
                warn!("AWS KMS signer health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_aws_kms_config_default() {
        let config = AwsKmsConfig::default();
        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
    }
    
    #[tokio::test]
    async fn test_should_retry() {
        let config = AwsKmsConfig::default();
        let signer = AwsKmsSigner {
            client: aws_sdk_kms::Client::from_conf(
                aws_sdk_kms::Config::builder()
                    .region(Region::new("us-east-1"))
                    .build()
            ),
            config,
            http_client: SharedHttpClient::new(
                HyperClientBuilder::new().build_https().unwrap()
            ),
        };
        
        // Test retryable errors
        assert!(signer.should_retry(&NetworkError::KmsTimeout("test".to_string())));
        assert!(signer.should_retry(&NetworkError::Kms("Throttling exception".to_string())));
        assert!(signer.should_retry(&NetworkError::Kms("Rate exceeded".to_string())));
        assert!(signer.should_retry(&NetworkError::Kms("Internal failure".to_string())));
        
        // Test non-retryable errors
        assert!(!signer.should_retry(&NetworkError::Kms("Key not found".to_string())));
        assert!(!signer.should_retry(&NetworkError::Crypto("Invalid signature".to_string())));
    }
}


//! On-chain-style network parameters from genesis, scheduled upgrades, and activation epochs.
//!
//! Upgrades are **announced** when the `ParameterUpgrade` transaction is accepted and recorded
//! against the current chain tip. They **apply** at `activation_epoch_height` so every honest
//! node that processes the same blocks transitions at the same height.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::Path;

/// Root genesis document loaded from JSON (see `config/genesis.example.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisDocument {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub chain_id: String,
    #[serde(default)]
    pub genesis_time_rfc3339: Option<String>,
    pub network_parameters: NetworkParameters,
    pub parameter_upgrade_governance: GovernanceConfig,
    #[serde(default = "default_min_activation_delay_blocks")]
    pub min_activation_delay_blocks: u64,
}

fn default_schema_version() -> u32 {
    1
}

fn default_min_activation_delay_blocks() -> u64 {
    100
}

/// Live tunable network limits (consensus-relevant in a full implementation).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkParameters {
    pub max_block_body_bytes: u64,
    pub min_base_fee: u64,
    pub max_transactions_per_block: u32,
}

/// Partial update merged at activation (unset fields keep the previous value).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkParametersPatch {
    pub max_block_body_bytes: Option<u64>,
    pub min_base_fee: Option<u64>,
    pub max_transactions_per_block: Option<u32>,
}

impl NetworkParameters {
    pub fn apply_patch(&self, patch: &NetworkParametersPatch) -> NetworkParameters {
        NetworkParameters {
            max_block_body_bytes: patch
                .max_block_body_bytes
                .unwrap_or(self.max_block_body_bytes),
            min_base_fee: patch.min_base_fee.unwrap_or(self.min_base_fee),
            max_transactions_per_block: patch
                .max_transactions_per_block
                .unwrap_or(self.max_transactions_per_block),
        }
    }
}

fn merge_patches(into: &mut NetworkParametersPatch, add: &NetworkParametersPatch) {
    if add.max_block_body_bytes.is_some() {
        into.max_block_body_bytes = add.max_block_body_bytes;
    }
    if add.min_base_fee.is_some() {
        into.min_base_fee = add.min_base_fee;
    }
    if add.max_transactions_per_block.is_some() {
        into.max_transactions_per_block = add.max_transactions_per_block;
    }
}

/// Who may submit a `ParameterUpgrade` transaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum GovernanceConfig {
    /// One of the listed keys must sign (see `authorize_upgrade`); addresses are compared case-insensitively.
    AdminKeys { keys: Vec<String> },
    /// A set of distinct DAO members must endorse the upgrade; each id must appear in `members`.
    Dao {
        members: Vec<String>,
        min_approvals: u32,
    },
}

/// Recorded upgrade for APIs and audit (may still be in the future relative to tip).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledUpgradeRecord {
    pub transaction_id: String,
    pub announced_at_height: u64,
    pub activation_epoch_height: u64,
    pub patch: NetworkParametersPatch,
}

/// In-memory registry: genesis base + scheduled activations by block height.
#[derive(Debug, Clone)]
pub struct ChainParameterRegistry {
    chain_id: String,
    governance: GovernanceConfig,
    min_activation_delay_blocks: u64,
    /// Last finalized / logical chain tip used for upgrade validation and status.
    current_height: u64,
    genesis_parameters: NetworkParameters,
    /// Patches keyed by activation height (merged if several txs target the same height).
    activations: BTreeMap<u64, NetworkParametersPatch>,
    /// All submitted upgrades (including already activated) for history APIs.
    upgrade_history: Vec<ScheduledUpgradeRecord>,
}

impl ChainParameterRegistry {
    pub fn from_genesis_file(path: &Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read genesis file {}: {}", path.display(), e))?;
        let doc: GenesisDocument = serde_json::from_str(&raw)
            .map_err(|e| format!("Invalid genesis JSON: {}", e))?;
        Ok(Self::from_genesis_document(doc))
    }

    pub fn from_genesis_document(doc: GenesisDocument) -> Self {
        Self {
            chain_id: doc.chain_id,
            governance: doc.parameter_upgrade_governance,
            min_activation_delay_blocks: doc.min_activation_delay_blocks,
            current_height: 0,
            genesis_parameters: doc.network_parameters.clone(),
            activations: BTreeMap::new(),
            upgrade_history: Vec::new(),
        }
    }

    /// Default used when no `GENESIS_CONFIG_PATH` is set (local dev only).
    pub fn development_default() -> Self {
        let doc = GenesisDocument {
            schema_version: 1,
            chain_id: "axionvera-dev".to_string(),
            genesis_time_rfc3339: None,
            network_parameters: NetworkParameters {
                max_block_body_bytes: 2_097_152,
                min_base_fee: 100,
                max_transactions_per_block: 1000,
            },
            parameter_upgrade_governance: GovernanceConfig::AdminKeys {
                keys: vec!["dev-admin".to_string()],
            },
            min_activation_delay_blocks: 10,
        };
        Self::from_genesis_document(doc)
    }

    pub fn chain_id(&self) -> &str {
        &self.chain_id
    }

    pub fn current_height(&self) -> u64 {
        self.current_height
    }

    pub fn min_activation_delay_blocks(&self) -> u64 {
        self.min_activation_delay_blocks
    }

    pub fn genesis_parameters(&self) -> &NetworkParameters {
        &self.genesis_parameters
    }

    /// Effective parameters at `height` (genesis + all activations with key ≤ height).
    pub fn effective_parameters_at(&self, height: u64) -> NetworkParameters {
        let mut p = self.genesis_parameters.clone();
        for (&h, patch) in self.activations.iter() {
            if h <= height {
                p = p.apply_patch(patch);
            }
        }
        p
    }

    /// Parameters currently enforced at the chain tip.
    pub fn active_parameters(&self) -> NetworkParameters {
        self.effective_parameters_at(self.current_height)
    }

    /// Upgrades whose activation height is strictly greater than the current tip (announced, not yet live).
    pub fn pending_upgrades(&self) -> Vec<ScheduledUpgradeRecord> {
        self.upgrade_history
            .iter()
            .filter(|r| r.activation_epoch_height > self.current_height)
            .cloned()
            .collect()
    }

    /// Advance the logical chain tip (hook for consensus / sync). Applying blocks should call this.
    pub fn set_chain_tip_height(&mut self, height: u64) {
        self.current_height = height;
    }

    /// Submit a parameter upgrade: validates governance, delay, and non-empty patch; schedules activation.
    pub fn submit_parameter_upgrade(
        &mut self,
        patch: NetworkParametersPatch,
        activation_epoch_height: u64,
        proposer_address: &str,
        dao_voter_addresses: &[String],
    ) -> Result<String, String> {
        if !patch_has_changes(&patch) {
            return Err("parameter patch must set at least one field".to_string());
        }

        self.authorize_upgrade(proposer_address, dao_voter_addresses)?;

        let tip = self.current_height;
        let min_h = tip.saturating_add(self.min_activation_delay_blocks);
        if activation_epoch_height < min_h {
            return Err(format!(
                "activation_epoch_height {} must be >= {} (tip {} + min_delay {})",
                activation_epoch_height, min_h, tip, self.min_activation_delay_blocks
            ));
        }

        self.activations
            .entry(activation_epoch_height)
            .and_modify(|existing| merge_patches(existing, &patch))
            .or_insert_with(|| patch.clone());

        let tx_id = format!("0x{:064x}", uuid::Uuid::new_v4().as_u128());
        self.upgrade_history.push(ScheduledUpgradeRecord {
            transaction_id: tx_id.clone(),
            announced_at_height: tip,
            activation_epoch_height,
            patch,
        });

        Ok(tx_id)
    }

    fn authorize_upgrade(
        &self,
        proposer_address: &str,
        dao_voter_addresses: &[String],
    ) -> Result<(), String> {
        match &self.governance {
            GovernanceConfig::AdminKeys { keys } => {
                if dao_voter_addresses.iter().any(|v| !v.is_empty()) {
                    return Err("dao_voter_addresses must be empty for admin_keys governance".to_string());
                }
                let p = normalize_id(proposer_address);
                if p.is_empty() {
                    return Err("proposer_address is required for admin_keys governance".to_string());
                }
                let ok = keys.iter().any(|k| normalize_id(k) == p);
                if !ok {
                    return Err("proposer is not an authorized admin key".to_string());
                }
                Ok(())
            }
            GovernanceConfig::Dao {
                members,
                min_approvals,
            } => {
                if *min_approvals == 0 {
                    return Err("dao min_approvals must be > 0".to_string());
                }
                let member_set: HashSet<String> = members.iter().map(|m| normalize_id(m)).collect();
                let mut seen = HashSet::new();
                let mut valid = 0u32;
                for v in dao_voter_addresses {
                    let n = normalize_id(v);
                    if n.is_empty() {
                        continue;
                    }
                    if !member_set.contains(&n) {
                        return Err(format!("unknown dao voter: {}", v));
                    }
                    if seen.insert(n) {
                        valid += 1;
                    }
                }
                if valid < *min_approvals {
                    return Err(format!(
                        "dao consensus requires {} distinct member approvals, got {}",
                        min_approvals, valid
                    ));
                }
                Ok(())
            }
        }
    }
}

fn normalize_id(s: &str) -> String {
    s.trim().to_ascii_lowercase()
}

fn patch_has_changes(p: &NetworkParametersPatch) -> bool {
    p.max_block_body_bytes.is_some()
        || p.min_base_fee.is_some()
        || p.max_transactions_per_block.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_registry_admin() -> ChainParameterRegistry {
        let doc = GenesisDocument {
            schema_version: 1,
            chain_id: "test".to_string(),
            genesis_time_rfc3339: None,
            network_parameters: NetworkParameters {
                max_block_body_bytes: 1_000_000,
                min_base_fee: 50,
                max_transactions_per_block: 500,
            },
            parameter_upgrade_governance: GovernanceConfig::AdminKeys {
                keys: vec!["Admin_ABC".to_string()],
            },
            min_activation_delay_blocks: 5,
        };
        ChainParameterRegistry::from_genesis_document(doc)
    }

    #[test]
    fn activation_epoch_applies_only_after_height() {
        let mut r = test_registry_admin();
        r.set_chain_tip_height(100);
        let patch = NetworkParametersPatch {
            min_base_fee: Some(99),
            ..Default::default()
        };
        let tx = r
            .submit_parameter_upgrade(patch.clone(), 106, "admin_abc", &[])
            .unwrap();
        assert!(!tx.is_empty());
        assert_eq!(r.active_parameters().min_base_fee, 50);
        assert_eq!(r.effective_parameters_at(105).min_base_fee, 50);
        assert_eq!(r.effective_parameters_at(106).min_base_fee, 99);
        r.set_chain_tip_height(200);
        assert_eq!(r.active_parameters().min_base_fee, 99);
    }

    #[test]
    fn dao_requires_distinct_members() {
        let doc = GenesisDocument {
            schema_version: 1,
            chain_id: "test-dao".to_string(),
            genesis_time_rfc3339: None,
            network_parameters: NetworkParameters {
                max_block_body_bytes: 1,
                min_base_fee: 1,
                max_transactions_per_block: 1,
            },
            parameter_upgrade_governance: GovernanceConfig::Dao {
                members: vec!["m1".to_string(), "m2".to_string(), "m3".to_string()],
                min_approvals: 2,
            },
            min_activation_delay_blocks: 1,
        };
        let mut r = ChainParameterRegistry::from_genesis_document(doc);
        r.set_chain_tip_height(10);
        let patch = NetworkParametersPatch {
            max_block_body_bytes: Some(999),
            ..Default::default()
        };
        assert!(r
            .submit_parameter_upgrade(patch.clone(), 12, "", &["m1".to_string()])
            .is_err());
        r.submit_parameter_upgrade(patch, 12, "", &["m1".to_string(), "m2".to_string()])
            .unwrap();
        assert_eq!(r.pending_upgrades().len(), 1);
    }
}
