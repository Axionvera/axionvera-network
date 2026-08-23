use serde::{Deserialize, Serialize};

pub mod config;

pub use config::{ConfigError, NodeConfig};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: String,
    pub service_name: String,
    pub version: String,
    pub environment: Option<String>,
}

pub fn health_status(environment: Option<String>) -> HealthResponse {
    HealthResponse {
        status: "ok".to_string(),
        service_name: "axionvera-network-node".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_status_with_none_env() {
        let resp = health_status(None);
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.service_name, "axionvera-network-node");
        assert_eq!(resp.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(resp.environment, None);
    }

    #[test]
    fn health_status_with_some_env() {
        let resp = health_status(Some("test-env".to_string()));
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.service_name, "axionvera-network-node");
        assert_eq!(resp.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(resp.environment, Some("test-env".to_string()));
    }
}
