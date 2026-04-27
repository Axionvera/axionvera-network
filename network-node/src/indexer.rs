use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument};

use crate::database::ConnectionPool;
use crate::error::NetworkError;
use crate::soroban_rpc_client::SorobanRpcClient;
use crate::stellar_service::StellarService;

#[derive(Debug, Serialize, Deserialize)]
struct GetEventsParams {
    #[serde(rename = "startLedger")]
    start_ledger: u32,
    filters: Vec<EventFilter>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventFilter {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(rename = "contractIds")]
    contract_ids: Vec<String>,
    topics: Vec<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventsResult {
    events: Vec<SorobanEvent>,
    #[serde(rename = "latestLedger")]
    latest_ledger: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct SorobanEvent {
    #[serde(rename = "type")]
    event_type: String,
    ledger: u32,
    #[serde(rename = "contractId")]
    contract_id: String,
    id: String,
    topic: Vec<String>,
    value: SorobanEventValue,
}

#[derive(Debug, Serialize, Deserialize)]
struct SorobanEventValue {
    xdr: String,
}

pub struct EventIndexer {
    stellar_service: Arc<StellarService>,
    connection_pool: ConnectionPool,
    soroban_rpc_client: Arc<SorobanRpcClient>,
    contract_id: String,
    polling_interval_secs: u64,
}

impl EventIndexer {
    pub fn new(
        stellar_service: Arc<StellarService>,
        connection_pool: ConnectionPool,
        soroban_rpc_client: Arc<SorobanRpcClient>,
        contract_id: String,
        polling_interval_secs: u64,
    ) -> Self {
        Self {
            stellar_service,
            connection_pool,
            soroban_rpc_client,
            contract_id,
            polling_interval_secs,
        }
    }

    #[instrument(skip(self))]
    pub async fn start(&self, shutdown_token: CancellationToken) -> Result<(), NetworkError> {
        info!(
            "Starting Soroban Event Indexer for contract: {}",
            self.contract_id
        );

        // Keep these dependencies initialized and available as the indexer evolves.
        let _ = (&self.stellar_service, &self.connection_pool);

        let mut interval = time::interval(Duration::from_secs(self.polling_interval_secs));
        let mut current_ledger: u32 = 0;

        loop {
            tokio::select! {
                _ = shutdown_token.cancelled() => {
                    info!("Event indexer stopped gracefully");
                    return Ok(());
                }
                _ = interval.tick() => {}
            }

            let filter = EventFilter {
                event_type: "contract".to_string(),
                contract_ids: vec![self.contract_id.clone()],
                topics: vec![vec!["AxionveraVault".to_string()]],
            };

            let params = GetEventsParams {
                start_ledger: current_ledger,
                filters: vec![filter],
            };

            match self
                .soroban_rpc_client
                .call::<_, EventsResult>("getEvents", params)
                .await
            {
                Ok(events_result) => {
                    for event in events_result.events {
                        info!(
                            event_id = %event.id,
                            ledger = event.ledger,
                            "Parsed AxionveraVault Soroban event"
                        );
                        debug!(xdr = %event.value.xdr, "Event XDR payload");
                    }

                    current_ledger = events_result.latest_ledger + 1;
                }
                Err(e) => {
                    error!(error = %e, "Failed to fetch Soroban events");
                }
            }
        }
    }
}
