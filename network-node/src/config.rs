use serde::{Deserialize, Serialize};

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
