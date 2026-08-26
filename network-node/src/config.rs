use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Supported Soroban network identifiers.
const SUPPORTED_NETWORKS: [&str; 4] = ["local", "testnet", "mainnet", "futurenet"];

/// Supported deployment environments.
const SUPPORTED_ENVIRONMENTS: [&str; 3] = ["development", "staging", "production"];

/// Errors returned when validating a [`NodeConfig`].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    #[error("network name must not be empty")]
    EmptyNetworkName,
    #[error("unsupported network name '{0}'; expected one of: local, testnet, mainnet, futurenet")]
    UnsupportedNetworkName(String),
    #[error("RPC URL must not be empty")]
    EmptyRpcUrl,
    #[error("invalid RPC URL '{0}'")]
    InvalidRpcUrl(String),
    #[error("invalid port in RPC URL '{0}'")]
    InvalidRpcPort(String),
    #[error("environment must not be empty")]
    EmptyEnvironment,
    #[error("unsupported environment '{0}'; expected one of: development, staging, production")]
    UnsupportedEnvironment(String),
}

/// Errors returned by [`load_config`].
#[derive(Debug, Error)]
pub enum LoadError {
    /// The file could not be opened or read.
    #[error("could not read config file '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    /// The file contents were not valid JSON or did not match [`NodeConfig`].
    #[error("could not parse config file '{path}': {source}")]
    Parse {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    /// The file was loaded and parsed but the resulting config failed
    /// validation.
    #[error("invalid config loaded from '{path}': {source}")]
    Validation {
        path: String,
        #[source]
        source: ConfigError,
    },
}

/// Load a [`NodeConfig`] from a JSON file at `path`, then validate it.
///
/// Steps:
/// 1. Read the file contents from `path`.
/// 2. Deserialise the contents as [`NodeConfig`] using `serde_json`.
/// 3. Call [`NodeConfig::validate`] on the result.
///
/// Any failure in those three steps is returned as the appropriate
/// [`LoadError`] variant.
///
/// # Errors
///
/// - [`LoadError::Io`] — the file does not exist or cannot be read.
/// - [`LoadError::Parse`] — the file is not valid JSON or is missing required
///   fields.
/// - [`LoadError::Validation`] — the file was parsed but the config values
///   failed validation (e.g. unsupported network name or invalid RPC URL).
pub fn load_config(path: impl AsRef<Path>) -> Result<NodeConfig, LoadError> {
    let path_str = path.as_ref().to_string_lossy().into_owned();

    let contents = std::fs::read_to_string(path.as_ref()).map_err(|source| LoadError::Io {
        path: path_str.clone(),
        source,
    })?;

    let config: NodeConfig =
        serde_json::from_str(&contents).map_err(|source| LoadError::Parse {
            path: path_str.clone(),
            source,
        })?;

    config
        .validate()
        .map_err(|source| LoadError::Validation {
            path: path_str,
            source,
        })?;

    Ok(config)
}

/// Lightweight configuration for the network node.
///
/// Field names follow the SDK config direction (`network`, `rpcUrl`) using
/// Rust snake_case. Defaults target a local Stellar/Soroban development stack
/// so the node can start without extra environment setup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Target network identifier (`local`, `testnet`, `mainnet`, `futurenet`).
    pub network_name: String,
    /// Soroban RPC endpoint URL.
    pub rpc_url: String,
    /// Deployment environment name (`development`, `staging`, `production`).
    pub environment: String,
}

impl NodeConfig {
    /// Validates the configuration, returning the first invalid field.
    ///
    /// Fields are checked in the order `network_name`, `rpc_url`,
    /// `environment`. The `network_name` and `environment` values must be
    /// non-empty and supported, and the `rpc_url` must be a structurally
    /// valid `http`/`https` URL with a valid port (when one is present).
    ///
    /// # Errors
    ///
    /// Returns a [`ConfigError`] describing the first field that fails
    /// validation.
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_network_name(&self.network_name)?;
        validate_rpc_url(&self.rpc_url)?;
        validate_environment(&self.environment)?;
        Ok(())
    }
}

fn validate_network_name(network_name: &str) -> Result<(), ConfigError> {
    if network_name.trim().is_empty() {
        return Err(ConfigError::EmptyNetworkName);
    }
    if !SUPPORTED_NETWORKS.contains(&network_name) {
        return Err(ConfigError::UnsupportedNetworkName(
            network_name.to_string(),
        ));
    }
    Ok(())
}

fn validate_environment(environment: &str) -> Result<(), ConfigError> {
    if environment.trim().is_empty() {
        return Err(ConfigError::EmptyEnvironment);
    }
    if !SUPPORTED_ENVIRONMENTS.contains(&environment) {
        return Err(ConfigError::UnsupportedEnvironment(environment.to_string()));
    }
    Ok(())
}

/// Validates the shape of an RPC URL and, when a port is explicitly present,
/// that the port is a valid TCP port number (`1..=65535`).
fn validate_rpc_url(rpc_url: &str) -> Result<(), ConfigError> {
    if rpc_url.trim().is_empty() {
        return Err(ConfigError::EmptyRpcUrl);
    }

    // `scheme://` must be present and the scheme must look like a scheme.
    let (scheme, rest) = rpc_url
        .split_once("://")
        .ok_or_else(|| ConfigError::InvalidRpcUrl(rpc_url.to_string()))?;
    if scheme.is_empty()
        || !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    {
        return Err(ConfigError::InvalidRpcUrl(rpc_url.to_string()));
    }

    // The authority runs from the end of `scheme://` up to the first path,
    // query, or fragment delimiter.
    let authority_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    if authority.is_empty() {
        return Err(ConfigError::InvalidRpcUrl(rpc_url.to_string()));
    }

    // Drop any `user:password@` prefix before reading the host and port.
    let host_port = authority.rsplit('@').next().unwrap_or(authority);

    // Parse the host and, when present, the port.
    let (host, port) = if let Some(after_bracket) = host_port.strip_prefix('[') {
        // IPv6 literal, e.g. `[::1]:8000`.
        let end = after_bracket
            .find(']')
            .ok_or_else(|| ConfigError::InvalidRpcUrl(rpc_url.to_string()))?;
        let host = &after_bracket[..end];
        if host.is_empty() {
            return Err(ConfigError::InvalidRpcUrl(rpc_url.to_string()));
        }
        let after = &after_bracket[end + 1..];
        match after.strip_prefix(':') {
            Some(port) => (host, Some(port)),
            None if after.is_empty() => (host, None),
            None => return Err(ConfigError::InvalidRpcUrl(rpc_url.to_string())),
        }
    } else if let Some((host, port)) = host_port.rsplit_once(':') {
        if host.is_empty() || host.contains(':') {
            return Err(ConfigError::InvalidRpcUrl(rpc_url.to_string()));
        }
        (host, Some(port))
    } else {
        (host_port, None)
    };

    if host.is_empty() {
        return Err(ConfigError::InvalidRpcUrl(rpc_url.to_string()));
    }

    if let Some(port) = port {
        let parsed: u16 = port
            .parse()
            .map_err(|_| ConfigError::InvalidRpcPort(rpc_url.to_string()))?;
        if parsed == 0 {
            return Err(ConfigError::InvalidRpcPort(rpc_url.to_string()));
        }
    }

    Ok(())
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network_name: "local".to_string(),
            rpc_url: "http://localhost:8000/soroban/rpc".to_string(),
            environment: "development".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(network_name: &str, rpc_url: &str, environment: &str) -> NodeConfig {
        NodeConfig {
            network_name: network_name.to_string(),
            rpc_url: rpc_url.to_string(),
            environment: environment.to_string(),
        }
    }

    // -------------------------------------------------------------------------
    // Happy path
    // -------------------------------------------------------------------------

    #[test]
    fn default_config_validates_successfully() {
        assert_eq!(NodeConfig::default().validate(), Ok(()));
    }

    #[test]
    fn all_supported_networks_and_environments_validate() {
        for network_name in SUPPORTED_NETWORKS {
            for environment in SUPPORTED_ENVIRONMENTS {
                let config = config(
                    network_name,
                    "http://localhost:8000/soroban/rpc",
                    environment,
                );
                assert_eq!(
                    config.validate(),
                    Ok(()),
                    "expected {network_name}/{environment} to validate"
                );
            }
        }
    }

    #[test]
    fn rpc_url_without_explicit_port_validates() {
        let config = config("testnet", "https://soroban-rpc.example.com", "staging");
        assert_eq!(config.validate(), Ok(()));
    }

    #[test]
    fn rpc_url_with_ipv6_host_and_port_validates() {
        let config = config("local", "http://[::1]:8000/soroban/rpc", "development");
        assert_eq!(config.validate(), Ok(()));
    }

    #[test]
    fn rpc_url_with_boundary_ports_validates() {
        let min = config("local", "http://localhost:1/rpc", "development");
        let max = config("local", "http://localhost:65535/rpc", "development");
        assert_eq!(min.validate(), Ok(()));
        assert_eq!(max.validate(), Ok(()));
    }

    // -------------------------------------------------------------------------
    // network_name validation
    // -------------------------------------------------------------------------

    #[test]
    fn rejects_empty_network_name() {
        assert_eq!(
            config("", "http://localhost:8000/rpc", "development").validate(),
            Err(ConfigError::EmptyNetworkName)
        );
    }

    #[test]
    fn rejects_whitespace_only_network_name() {
        assert_eq!(
            config("   ", "http://localhost:8000/rpc", "development").validate(),
            Err(ConfigError::EmptyNetworkName)
        );
    }

    #[test]
    fn rejects_unsupported_network_name() {
        assert_eq!(
            config("mainnet2", "http://localhost:8000/rpc", "development").validate(),
            Err(ConfigError::UnsupportedNetworkName("mainnet2".to_string()))
        );
    }

    // -------------------------------------------------------------------------
    // rpc_url validation
    // -------------------------------------------------------------------------

    #[test]
    fn rejects_empty_rpc_url() {
        assert_eq!(
            config("local", "", "development").validate(),
            Err(ConfigError::EmptyRpcUrl)
        );
    }

    #[test]
    fn rejects_whitespace_only_rpc_url() {
        assert_eq!(
            config("local", "   ", "development").validate(),
            Err(ConfigError::EmptyRpcUrl)
        );
    }

    #[test]
    fn rejects_rpc_url_without_scheme() {
        assert_eq!(
            config("local", "localhost:8000/soroban/rpc", "development").validate(),
            Err(ConfigError::InvalidRpcUrl(
                "localhost:8000/soroban/rpc".to_string()
            ))
        );
    }

    #[test]
    fn rejects_rpc_url_without_host() {
        assert_eq!(
            config("local", "http:///soroban/rpc", "development").validate(),
            Err(ConfigError::InvalidRpcUrl(
                "http:///soroban/rpc".to_string()
            ))
        );
    }

    #[test]
    fn rejects_non_numeric_port() {
        assert_eq!(
            config("local", "http://localhost:abc/soroban/rpc", "development").validate(),
            Err(ConfigError::InvalidRpcPort(
                "http://localhost:abc/soroban/rpc".to_string()
            ))
        );
    }

    #[test]
    fn rejects_zero_port() {
        assert_eq!(
            config("local", "http://localhost:0/soroban/rpc", "development").validate(),
            Err(ConfigError::InvalidRpcPort(
                "http://localhost:0/soroban/rpc".to_string()
            ))
        );
    }

    #[test]
    fn rejects_out_of_range_port() {
        assert_eq!(
            config("local", "http://localhost:70000/soroban/rpc", "development").validate(),
            Err(ConfigError::InvalidRpcPort(
                "http://localhost:70000/soroban/rpc".to_string()
            ))
        );
    }

    #[test]
    fn rejects_unbracketed_colon_in_host() {
        assert_eq!(
            config("local", "http://localhost:8000:9000/rpc", "development").validate(),
            Err(ConfigError::InvalidRpcUrl(
                "http://localhost:8000:9000/rpc".to_string()
            ))
        );
    }

    // -------------------------------------------------------------------------
    // environment validation
    // -------------------------------------------------------------------------

    #[test]
    fn rejects_empty_environment() {
        assert_eq!(
            config("local", "http://localhost:8000/rpc", "").validate(),
            Err(ConfigError::EmptyEnvironment)
        );
    }

    #[test]
    fn rejects_whitespace_only_environment() {
        assert_eq!(
            config("local", "http://localhost:8000/rpc", "  ").validate(),
            Err(ConfigError::EmptyEnvironment)
        );
    }

    #[test]
    fn rejects_unsupported_environment() {
        assert_eq!(
            config("local", "http://localhost:8000/rpc", "prod").validate(),
            Err(ConfigError::UnsupportedEnvironment("prod".to_string()))
        );
    }

    // -------------------------------------------------------------------------
    // Error messages
    // -------------------------------------------------------------------------

    #[test]
    fn error_messages_are_clear_and_descriptive() {
        assert_eq!(
            ConfigError::EmptyNetworkName.to_string(),
            "network name must not be empty"
        );
        assert_eq!(
            ConfigError::UnsupportedNetworkName("foo".to_string()).to_string(),
            "unsupported network name 'foo'; expected one of: local, testnet, mainnet, futurenet"
        );
        assert_eq!(
            ConfigError::EmptyRpcUrl.to_string(),
            "RPC URL must not be empty"
        );
        assert_eq!(
            ConfigError::InvalidRpcUrl("nope".to_string()).to_string(),
            "invalid RPC URL 'nope'"
        );
        assert_eq!(
            ConfigError::InvalidRpcPort("http://localhost:0/rpc".to_string()).to_string(),
            "invalid port in RPC URL 'http://localhost:0/rpc'"
        );
        assert_eq!(
            ConfigError::EmptyEnvironment.to_string(),
            "environment must not be empty"
        );
        assert_eq!(
            ConfigError::UnsupportedEnvironment("prod".to_string()).to_string(),
            "unsupported environment 'prod'; expected one of: development, staging, production"
        );
    }

    // -------------------------------------------------------------------------
    // Existing serialization tests
    // -------------------------------------------------------------------------

    #[test]
    fn default_config_uses_local_development_values() {
        let config = NodeConfig::default();
        assert_eq!(config.network_name, "local");
        assert_eq!(config.rpc_url, "http://localhost:8000/soroban/rpc");
        assert_eq!(config.environment, "development");
    }

    #[test]
    fn default_config_round_trips_through_json() {
        let config = NodeConfig::default();
        let encoded = serde_json::to_string(&config).expect("serialize node config");
        let decoded: NodeConfig = serde_json::from_str(&encoded).expect("deserialize node config");
        assert_eq!(decoded, config);
    }

    // -------------------------------------------------------------------------
    // load_config tests
    // -------------------------------------------------------------------------

    use std::io::Write as _;

    /// Write `contents` to a named temp file and return the path.  The file
    /// lives for the lifetime of the returned [`tempfile::NamedTempFile`];
    /// keep the value alive for the duration of the test.
    fn write_temp_file(contents: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().expect("create temp file");
        file.write_all(contents.as_bytes())
            .expect("write temp file");
        file
    }

    #[test]
    fn load_config_happy_path_returns_validated_config() {
        let json = r#"{
            "network_name": "testnet",
            "rpc_url": "https://soroban-rpc.example.com",
            "environment": "staging"
        }"#;
        let file = write_temp_file(json);
        let result = load_config(file.path());
        assert!(result.is_ok(), "expected Ok, got {:?}", result);
        let loaded = result.unwrap();
        assert_eq!(loaded.network_name, "testnet");
        assert_eq!(loaded.rpc_url, "https://soroban-rpc.example.com");
        assert_eq!(loaded.environment, "staging");
    }

    #[test]
    fn load_config_returns_default_equivalent_when_file_has_default_values() {
        let default = NodeConfig::default();
        let json = serde_json::to_string(&default).expect("serialize default config");
        let file = write_temp_file(&json);
        let loaded = load_config(file.path()).expect("load default config");
        assert_eq!(loaded, default);
    }

    #[test]
    fn load_config_returns_io_error_when_file_does_not_exist() {
        let result = load_config("/tmp/axionvera_nonexistent_config_file_12345.json");
        assert!(
            matches!(result, Err(LoadError::Io { .. })),
            "expected LoadError::Io, got {:?}",
            result
        );
    }

    #[test]
    fn load_config_io_error_message_contains_path() {
        let path = "/tmp/axionvera_nonexistent_config_file_67890.json";
        let err = load_config(path).unwrap_err();
        assert!(
            err.to_string().contains(path),
            "error message should contain the path, got: {err}"
        );
    }

    #[test]
    fn load_config_returns_parse_error_for_malformed_json() {
        let file = write_temp_file("this is not json {{{");
        let result = load_config(file.path());
        assert!(
            matches!(result, Err(LoadError::Parse { .. })),
            "expected LoadError::Parse, got {:?}",
            result
        );
    }

    #[test]
    fn load_config_returns_parse_error_for_empty_file() {
        let file = write_temp_file("");
        let result = load_config(file.path());
        assert!(
            matches!(result, Err(LoadError::Parse { .. })),
            "expected LoadError::Parse for empty file, got {:?}",
            result
        );
    }

    #[test]
    fn load_config_returns_parse_error_for_missing_required_fields() {
        // Valid JSON but missing all NodeConfig fields.
        let file = write_temp_file(r#"{"foo": "bar"}"#);
        let result = load_config(file.path());
        assert!(
            matches!(result, Err(LoadError::Parse { .. })),
            "expected LoadError::Parse for missing fields, got {:?}",
            result
        );
    }

    #[test]
    fn load_config_returns_parse_error_for_wrong_value_types() {
        // network_name is a number instead of a string.
        let file = write_temp_file(
            r#"{"network_name": 42, "rpc_url": "http://localhost:8000", "environment": "development"}"#,
        );
        let result = load_config(file.path());
        assert!(
            matches!(result, Err(LoadError::Parse { .. })),
            "expected LoadError::Parse for wrong types, got {:?}",
            result
        );
    }

    #[test]
    fn load_config_returns_validation_error_for_unsupported_network() {
        let json = r#"{
            "network_name": "devnet",
            "rpc_url": "http://localhost:8000/soroban/rpc",
            "environment": "development"
        }"#;
        let file = write_temp_file(json);
        let result = load_config(file.path());
        assert!(
            matches!(
                result,
                Err(LoadError::Validation {
                    source: ConfigError::UnsupportedNetworkName(_),
                    ..
                })
            ),
            "expected LoadError::Validation(UnsupportedNetworkName), got {:?}",
            result
        );
    }

    #[test]
    fn load_config_returns_validation_error_for_invalid_rpc_url() {
        let json = r#"{
            "network_name": "local",
            "rpc_url": "not-a-url",
            "environment": "development"
        }"#;
        let file = write_temp_file(json);
        let result = load_config(file.path());
        assert!(
            matches!(
                result,
                Err(LoadError::Validation {
                    source: ConfigError::InvalidRpcUrl(_),
                    ..
                })
            ),
            "expected LoadError::Validation(InvalidRpcUrl), got {:?}",
            result
        );
    }

    #[test]
    fn load_config_returns_validation_error_for_unsupported_environment() {
        let json = r#"{
            "network_name": "mainnet",
            "rpc_url": "https://soroban-rpc.example.com",
            "environment": "prod"
        }"#;
        let file = write_temp_file(json);
        let result = load_config(file.path());
        assert!(
            matches!(
                result,
                Err(LoadError::Validation {
                    source: ConfigError::UnsupportedEnvironment(_),
                    ..
                })
            ),
            "expected LoadError::Validation(UnsupportedEnvironment), got {:?}",
            result
        );
    }

    #[test]
    fn load_config_validation_error_message_contains_path_and_cause() {
        let json = r#"{
            "network_name": "badnet",
            "rpc_url": "http://localhost:8000/soroban/rpc",
            "environment": "development"
        }"#;
        let file = write_temp_file(json);
        let err = load_config(file.path()).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("badnet") || msg.contains("invalid config"),
            "error message should mention the cause, got: {msg}"
        );
    }

    #[test]
    fn load_config_all_supported_networks_and_environments_load_successfully() {
        for network_name in SUPPORTED_NETWORKS {
            for environment in SUPPORTED_ENVIRONMENTS {
                let json = format!(
                    r#"{{"network_name":"{network_name}","rpc_url":"http://localhost:8000/soroban/rpc","environment":"{environment}"}}"#
                );
                let file = write_temp_file(&json);
                let result = load_config(file.path());
                assert!(
                    result.is_ok(),
                    "expected Ok for {network_name}/{environment}, got {:?}",
                    result
                );
            }
        }
    }
