#![cfg(test)]

//! Integration tests for the AxionVera Vault contract.
//!
//! These tests verify the core functionality of the contract, including
//! initialization, security guards, and basic interaction flows.

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env};

/// Verifies that the contract can only be initialized once.
//!
//! This test attempts to call `initialize` twice and expects the second call
//! to fail with the `AlreadyInitialized` error code.
#[test]
fn test_initialization_is_one_time() {
    let e = Env::default();
    e.mock_all_auths();

    // Register the contract in the test environment.
    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    // Generate random addresses for the test.
    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);

    // First initialization should succeed.
    client.initialize(&admin, &deposit_token, &reward_token);

    // Second initialization MUST fail with the specific error code.
    let result = client.try_initialize(&admin, &deposit_token, &reward_token);
    
    assert_eq!(
        result,
        Err(Ok(VaultError::AlreadyInitialized))
    );
}

/// Verifies that the `initialize` function requires the admin's authorization.
//!
//! This test ensures that an attacker cannot initialize the contract with someone
//! else's address as the admin without their signature.
#[test]
fn test_initialize_requires_admin_auth() {
    let e = Env::default();
    // Do NOT mock auths to verify that require_auth() actually triggers.

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let deposit_token = Address::generate(&e);
    let reward_token = Address::generate(&e);

    // This call should panic because admin hasn't authorized it in the mock environment.
    // In a real environment, this would require a signature.
    let result = client.try_initialize(&admin, &deposit_token, &reward_token);
    
    // Check if it's an authentication error
    assert!(result.is_err());
}

/// Verifies that the contract cannot be initialized with identical tokens.
#[test]
fn test_initialize_fails_with_same_tokens() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let token = Address::generate(&e);

    let result = client.try_initialize(&admin, &token, &token);
    
    assert_eq!(
        result,
        Err(Ok(VaultError::InvalidTokenConfiguration))
    );
}
