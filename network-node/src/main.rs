use axionvera_network_node::NetworkNode;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let config = axionvera_network_node::config::NetworkConfig::from_env()?;
    
    let log_level = config.log_level.parse::<tracing::Level>()
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(format!("axionvera_network_node={}", log_level)))
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting axionvera-network node v{}", env!("CARGO_PKG_VERSION"));

    // Create and start the network node
    let node = NetworkNode::new(config).await?;
    
    if let Err(e) = node.start().await {
        error!("Network node failed: {}", e);
        std::process::exit(1);
    }

    info!("Network node shutdown complete");
    Ok(())
}
