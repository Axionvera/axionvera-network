use std::time::Duration;
use serde::{Deserialize, Serialize};
use crate::config::SorobanConfig;
use crate::error::{NetworkError, Result};
use tracing::{debug, error};

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

pub struct SorobanRpcClient {
    config: SorobanConfig,
    http_client: reqwest::Client,
}

impl SorobanRpcClient {
    pub fn new(config: SorobanConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
        }
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

        debug!("Soroban RPC call: {} with params: {:?}", method, serde_json::to_string(&request).unwrap_or_default());

        let response = self.http_client
            .post(&self.config.rpc_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| NetworkError::Connection(format!("Failed to send Soroban RPC request: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Soroban RPC HTTP error: {} - {}", status, text);
            return Err(NetworkError::SorobanRpc(format!("HTTP error {}: {}", status, text)));
        }

        let json_response: JsonRpcResponse<R> = response.json()
            .await
            .map_err(|e| NetworkError::Serialization(e.into()))?;

        if let Some(error) = json_response.error {
            error!("Soroban RPC error {}: {}", error.code, error.message);
            return Err(NetworkError::SorobanRpc(format!("RPC error {}: {}", error.code, error.message)));
        }

        json_response.result.ok_or_else(|| NetworkError::SorobanRpc("Empty result in Soroban RPC response".to_string()))
    }

    pub async fn get_health(&self) -> Result<HealthResponse> {
        self.call("getHealth", ()).await
    }

    pub async fn get_transaction(&self, hash: &str) -> Result<GetTransactionResponse> {
        #[derive(Serialize)]
        struct Params { hash: String }
        self.call("getTransaction", Params { hash: hash.to_string() }).await
    }

    pub async fn simulate_transaction(&self, transaction_xdr: &str) -> Result<SimulateTransactionResponse> {
        #[derive(Serialize)]
        struct Params { transaction: String }
        self.call("simulateTransaction", Params { transaction: transaction_xdr.to_string() }).await
    }

    pub async fn send_transaction(&self, transaction_xdr: &str) -> Result<SendTransactionResponse> {
        #[derive(Serialize)]
        struct Params { transaction: String }
        self.call("sendTransaction", Params { transaction: transaction_xdr.to_string() }).await
    }
}
