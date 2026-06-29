#![cfg(test)]
use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_pause_unpause_flow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, SecurityContract);
    let client = SecurityContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    
    // Initialize
    client.init(&admin);
    assert_eq!(client.is_paused(), false);

    // Pause (Requires mocking admin auth in actual Soroban test setup)
    env.mock_all_auths();
    client.pause();
    assert_eq!(client.is_paused(), true);

    // Unpause
    client.unpause();
    assert_eq!(client.is_paused(), false);
}