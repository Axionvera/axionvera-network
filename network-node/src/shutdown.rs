use std::time::Duration;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

/// Shutdown handler manages graceful shutdown process using CancellationTokens
pub struct ShutdownHandler {
    grace_period: Duration,
    token: CancellationToken,
}

impl ShutdownHandler {
    /// Create a new shutdown handler
    pub fn new(grace_period: Duration) -> Self {
        Self {
            grace_period,
            token: CancellationToken::new(),
        }
    }

    /// Get a reference to the cancellation token
    pub fn token(&self) -> CancellationToken {
        self.token.clone()
    }

    /// Start listening for shutdown signals
    pub fn start(&self) -> CancellationToken {
        let token = self.token.clone();

        // Spawn signal handler
        tokio::spawn(async move {
            #[cfg(unix)]
            {
                let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("Failed to install SIGTERM handler");
                let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                    .expect("Failed to install SIGINT handler");

                tokio::select! {
                    _ = sigterm.recv() => {
                        info!("Received SIGTERM, initiating graceful shutdown");
                    }
                    _ = sigint.recv() => {
                        info!("Received SIGINT, initiating graceful shutdown");
                    }
                }
            }

            #[cfg(not(unix))]
            {
                if let Err(e) = signal::ctrl_c().await {
                    error!("Failed to install CTRL+C handler: {}", e);
                } else {
                    info!("Received CTRL+C, initiating graceful shutdown");
                }
            }

            token.cancel();
        });

        self.token.clone()
    }

    /// Get the grace period
    pub fn grace_period(&self) -> Duration {
        self.grace_period
    }
}
