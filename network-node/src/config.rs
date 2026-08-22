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
}
