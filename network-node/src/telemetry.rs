use tonic::metadata::{MetadataMap, MetadataValue};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize structured JSON logging for Datadog/CloudWatch
pub fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with(fmt::layer().json())
        .init();
}

/// Extract traceparent from gRPC metadata for distributed tracing
pub fn extract_traceparent_grpc(metadata: &MetadataMap) -> Option<String> {
    metadata
        .get("traceparent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Inject traceparent into gRPC metadata for distributed tracing
pub fn inject_traceparent_grpc(metadata: &mut MetadataMap, traceparent: &str) {
    if let Ok(value) = MetadataValue::try_from(traceparent) {
        metadata.insert("traceparent", value);
    }
}
