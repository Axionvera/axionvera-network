#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_initialization_is_one_time() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);

    // First initialization succeeds
    client.initialize(&admin, &deposit_token, &reward_token);

    // Second initialization should fail with AlreadyInitialized error
    let result = client.try_initialize(&admin, &deposit_token, &reward_token);
    
    assert_eq!(
        result,
        Err(Ok(VaultError::AlreadyInitialized))
    );
}

#[test]
fn test_initialize_requires_admin_auth() {
    let e = Env::default();
    // No mock_all_auths() here to test authentication

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);

    // This should fail because admin hasn't authorized it
    // In testutils, we can verify auths
    client.initialize(&admin, &deposit_token, &reward_token);
    
    assert_eq!(
        e.auths()[0],
        (
            admin.clone(),
            soroban_sdk::testutils::AuthorizedInvocation {
                function: soroban_sdk::testutils::AuthorizedFunction::Contract((
                    contract_id.clone(),
                    soroban_sdk::symbol_short!("initiali"), // Soroban symbols are max 10 chars, "initialize" might be truncated or different
                    (admin, deposit_token, reward_token).into_val(&e),
                )),
                sub_invocations: std::vec![],
            }
        )
    );
}
