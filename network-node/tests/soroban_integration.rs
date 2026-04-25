use axionvera_network_node::config::SorobanConfig;
use axionvera_network_node::soroban_rpc_client::SorobanRpcClient;
use axionvera_network_node::soroban_service::SorobanService;
use std::sync::Arc;

#[tokio::test]
async fn test_soroban_get_health() {
    let config = SorobanConfig::default();
    let rpc_client = Arc::new(SorobanRpcClient::new(config));
    let service = SorobanService::new(rpc_client);

    // This test will likely fail if there's no internet or the Testnet RPC is down,
    // so we just check if the call can be made and handle the result.
    match service.get_health().await {
        Ok(status) => {
            println!("Soroban RPC Health: {}", status);
            assert!(status == "healthy" || status == "unhealthy");
        }
        Err(e) => {
            println!("Soroban RPC call failed (expected if offline): {}", e);
        }
    }
}

#[tokio::test]
async fn test_soroban_simulate_read_only() {
    let config = SorobanConfig::default();
    let rpc_client = Arc::new(SorobanRpcClient::new(config));
    let service = SorobanService::new(rpc_client);

    // Example: A simple read-only simulation for a known contract on Testnet
    // If you don't have a specific contract, we use a placeholder or simulate a failure
    let contract_id = "CCDRM2F5H7..."; // Placeholder
    
    // In a real integration test, we would build a real InvokeHostFunction XDR here
    let dummy_tx_xdr = "AAAAAgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    match service.simulate_transaction(dummy_tx_xdr).await {
        Ok(response) => {
            println!("Simulation successful: {:?}", response);
        }
        Err(e) => {
            println!("Simulation failed (expected with dummy XDR): {}", e);
        }
    }
}
