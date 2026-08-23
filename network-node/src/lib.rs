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

    // Happy path tests
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

    // Test all required fields are present
    #[test]
    fn health_response_has_all_required_fields() {
        let resp = health_status(None);

        // Status field
        assert!(!resp.status.is_empty(), "Status should not be empty");
        assert_eq!(resp.status, "ok");

        // Service name field
        assert!(
            !resp.service_name.is_empty(),
            "Service name should not be empty"
        );
        assert_eq!(resp.service_name, "axionvera-network-node");

        // Version field
        assert!(!resp.version.is_empty(), "Version should not be empty");
        assert_eq!(resp.version, env!("CARGO_PKG_VERSION"));

        // Environment field (optional but should be present in struct)
        assert_eq!(resp.environment, None);
    }

    // Test with various environment values
    #[test]
    fn health_status_with_production_env() {
        let resp = health_status(Some("production".to_string()));
        assert_eq!(resp.environment, Some("production".to_string()));
        assert_eq!(resp.status, "ok");
    }

    #[test]
    fn health_status_with_staging_env() {
        let resp = health_status(Some("staging".to_string()));
        assert_eq!(resp.environment, Some("staging".to_string()));
        assert_eq!(resp.status, "ok");
    }

    #[test]
    fn health_status_with_development_env() {
        let resp = health_status(Some("development".to_string()));
        assert_eq!(resp.environment, Some("development".to_string()));
        assert_eq!(resp.status, "ok");
    }

    // Test serialization support
    #[test]
    fn health_response_serializes_correctly() {
        let resp = health_status(Some("test-env".to_string()));
        let json = serde_json::to_string(&resp).expect("Failed to serialize");

        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"service_name\":\"axionvera-network-node\""));
        assert!(json.contains("\"version\":"));
        assert!(json.contains("\"environment\":\"test-env\""));
    }

    #[test]
    fn health_response_serializes_without_environment() {
        let resp = health_status(None);
        let json = serde_json::to_string(&resp).expect("Failed to serialize");

        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"service_name\":\"axionvera-network-node\""));
        assert!(json.contains("\"version\":"));
        assert!(json.contains("\"environment\":null"));
    }

    // Test deserialization support
    #[test]
    fn health_response_deserializes_correctly() {
        let json = r#"{"status":"ok","service_name":"axionvera-network-node","version":"0.1.0","environment":"test-env"}"#;
        let resp: HealthResponse = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(resp.status, "ok");
        assert_eq!(resp.service_name, "axionvera-network-node");
        assert_eq!(resp.version, "0.1.0");
        assert_eq!(resp.environment, Some("test-env".to_string()));
    }

    #[test]
    fn health_response_deserializes_without_environment() {
        let json = r#"{"status":"ok","service_name":"axionvera-network-node","version":"0.1.0","environment":null}"#;
        let resp: HealthResponse = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(resp.status, "ok");
        assert_eq!(resp.service_name, "axionvera-network-node");
        assert_eq!(resp.version, "0.1.0");
        assert_eq!(resp.environment, None);
    }

    // Test response consistency
    #[test]
    fn health_response_maintains_consistency() {
        let resp1 = health_status(Some("env1".to_string()));
        let resp2 = health_status(Some("env1".to_string()));

        assert_eq!(resp1.status, resp2.status);
        assert_eq!(resp1.service_name, resp2.service_name);
        assert_eq!(resp1.version, resp2.version);
        assert_eq!(resp1.environment, resp2.environment);
    }

    // Test response equality
    #[test]
    fn health_response_equality() {
        let resp1 = HealthResponse {
            status: "ok".to_string(),
            service_name: "axionvera-network-node".to_string(),
            version: "0.1.0".to_string(),
            environment: Some("test".to_string()),
        };

        let resp2 = HealthResponse {
            status: "ok".to_string(),
            service_name: "axionvera-network-node".to_string(),
            version: "0.1.0".to_string(),
            environment: Some("test".to_string()),
        };

        assert_eq!(resp1, resp2);
    }

    // Test empty string environment
    #[test]
    fn health_status_with_empty_string_env() {
        let resp = health_status(Some(String::new()));
        assert_eq!(resp.status, "ok");
        assert_eq!(resp.environment, Some(String::new()));
    }

    // Test long environment string
    #[test]
    fn health_status_with_long_env_string() {
        let long_env = "production-us-west-2-cluster-a".to_string();
        let resp = health_status(Some(long_env.clone()));
        assert_eq!(resp.environment, Some(long_env));
        assert_eq!(resp.status, "ok");
    }

    // Test struct derives
    #[test]
    fn health_response_debug_format() {
        let resp = health_status(Some("test".to_string()));
        let debug_str = format!("{:?}", resp);

        assert!(debug_str.contains("HealthResponse"));
        assert!(debug_str.contains("status"));
        assert!(debug_str.contains("service_name"));
        assert!(debug_str.contains("version"));
        assert!(debug_str.contains("environment"));
    }

    #[test]
    fn health_response_clone() {
        let resp1 = health_status(Some("test".to_string()));
        let resp2 = resp1.clone();

        assert_eq!(resp1, resp2);
        assert_eq!(resp1.status, resp2.status);
    }
}
