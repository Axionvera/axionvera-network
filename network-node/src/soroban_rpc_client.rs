use crate::config::SorobanConfig;
use crate::error::{NetworkError, Result};
use metrics::counter;
use reqwest::StatusCode;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    pub params: T,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcResponse<T> {
    pub jsonrpc: String,
    pub id: u64,
    pub result: Option<T>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTransactionResponse {
    pub status: String,
    pub latest_ledger: u32,
    pub latest_ledger_close_time: String,
    pub oldest_ledger: u32,
    pub oldest_ledger_close_time: String,
    pub result_xdr: Option<String>,
    pub result_meta_xdr: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulateTransactionResponse {
    pub latest_ledger: u32,
    pub results: Option<Vec<SimulateHostFunctionResult>>,
    pub min_resource_fee: Option<String>,
    pub events: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SimulateHostFunctionResult {
    pub auth: Option<Vec<String>>,
    pub xdr: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendTransactionResponse {
    pub hash: String,
    pub status: String,
    pub latest_ledger: u32,
    pub latest_ledger_close_time: String,
    pub error_result_xdr: Option<String>,
}

pub struct SorobanClient {
    client: ClientWithMiddleware,
}

impl SorobanClient {
    fn new(timeout: Duration) -> Self {
        let reqwest_client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(reqwest_client)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Self { client }
    }

    async fn post_json<T: Serialize + ?Sized>(
        &self,
        url: &str,
        body: &T,
    ) -> std::result::Result<reqwest::Response, reqwest_middleware::Error> {
        self.client.post(url).json(body).send().await
    }
}

pub struct SorobanRpcClient {
    rpc_urls: Vec<String>,
    http_client: SorobanClient,
}

impl SorobanRpcClient {
    pub fn new(config: SorobanConfig) -> Self {
        Self {
            rpc_urls: config.rpc_endpoints(),
            http_client: SorobanClient::new(Duration::from_secs(30)),
        }
    }

    fn track_failover(&self, from_index: usize, reason: &str) {
        if from_index + 1 >= self.rpc_urls.len() {
            return;
        }

        let from = &self.rpc_urls[from_index];
        let to = &self.rpc_urls[from_index + 1];

        counter!("soroban_rpc_failovers_total").increment(1);
        warn!(
            from_rpc_url = %from,
            to_rpc_url = %to,
            reason = %reason,
            "Failing over Soroban RPC endpoint"
        );
    }

    pub async fn call<P, R>(&self, method: &str, params: P) -> Result<R>
    where
        P: Serialize,
        R: for<'de> Deserialize<'de>,
    {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: 1,
            method: method.to_string(),
            params,
        };

        debug!(
            "Soroban RPC call: {} with params: {:?}",
            method,
            serde_json::to_string(&request).unwrap_or_default()
        );

        let mut last_error: Option<NetworkError> = None;

        for (index, rpc_url) in self.rpc_urls.iter().enumerate() {
            let response = match self.http_client.post_json(rpc_url, &request).await {
                Ok(response) => response,
                Err(e) => {
                    let err = NetworkError::Connection(format!(
                        "Failed to send Soroban RPC request to {}: {}",
                        rpc_url, e
                    ));

                    if index + 1 < self.rpc_urls.len() {
                        self.track_failover(index, "request_error");
                        last_error = Some(err);
                        continue;
                    }

                    return Err(err);
                }
            };

            if response.status() == StatusCode::SERVICE_UNAVAILABLE
                && index + 1 < self.rpc_urls.len()
            {
                let text = response.text().await.unwrap_or_default();
                warn!(
                    rpc_url = %rpc_url,
                    status = %StatusCode::SERVICE_UNAVAILABLE,
                    body = %text,
                    "Soroban RPC endpoint unavailable"
                );
                self.track_failover(index, "http_503");
                last_error = Some(NetworkError::SorobanRpc(format!(
                    "HTTP error {} from {}: {}",
                    StatusCode::SERVICE_UNAVAILABLE,
                    rpc_url,
                    text
                )));
                continue;
            }

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                error!("Soroban RPC HTTP error: {} - {}", status, text);
                return Err(NetworkError::SorobanRpc(format!(
                    "HTTP error {}: {}",
                    status, text
                )));
            }

            let json_response: JsonRpcResponse<R> = response
                .json()
                .await
                .map_err(|e| NetworkError::Serialization(e.into()))?;

            if let Some(error) = json_response.error {
                error!("Soroban RPC error {}: {}", error.code, error.message);
                return Err(NetworkError::SorobanRpc(format!(
                    "RPC error {}: {}",
                    error.code, error.message
                )));
            }

            return json_response.result.ok_or_else(|| {
                NetworkError::SorobanRpc("Empty result in Soroban RPC response".to_string())
            });
        }

        Err(last_error.unwrap_or_else(|| {
            NetworkError::SorobanRpc("No Soroban RPC endpoints available".to_string())
        }))
    }

    pub async fn get_health(&self) -> Result<HealthResponse> {
        self.call("getHealth", ()).await
    }

    pub async fn get_transaction(&self, hash: &str) -> Result<GetTransactionResponse> {
        #[derive(Serialize)]
        struct Params {
            hash: String,
        }
        self.call(
            "getTransaction",
            Params {
                hash: hash.to_string(),
            },
        )
        .await
    }

    pub async fn simulate_transaction(
        &self,
        transaction_xdr: &str,
    ) -> Result<SimulateTransactionResponse> {
        #[derive(Serialize)]
        struct Params {
            transaction: String,
        }
        self.call(
            "simulateTransaction",
            Params {
                transaction: transaction_xdr.to_string(),
            },
        )
        .await
    }

    pub async fn send_transaction(&self, transaction_xdr: &str) -> Result<SendTransactionResponse> {
        #[derive(Serialize)]
        struct Params {
            transaction: String,
        }
        self.call(
            "sendTransaction",
            Params {
                transaction: transaction_xdr.to_string(),
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode as AxumStatusCode;
    use axum::{routing::post, Json, Router};
    use serde_json::json;
    use tokio::net::TcpListener;

    async fn spawn_test_server(app: Router) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        format!("http://{}", address)
    }

    #[tokio::test]
    async fn falls_back_to_secondary_rpc_on_503() {
        let primary_url = spawn_test_server(Router::new().route(
            "/",
            post(|| async { (AxumStatusCode::SERVICE_UNAVAILABLE, "service unavailable") }),
        ))
        .await;

        let secondary_url = spawn_test_server(Router::new().route(
            "/",
            post(|| async {
                Json(json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": {
                        "status": "healthy"
                    }
                }))
            }),
        ))
        .await;

        let config = SorobanConfig {
            rpc_url: primary_url,
            rpc_urls: vec![secondary_url],
            ..SorobanConfig::default()
        };

        let client = SorobanRpcClient::new(config);
        let health = client
            .get_health()
            .await
            .expect("health check should succeed");

        assert_eq!(health.status, "healthy");
    }
}
