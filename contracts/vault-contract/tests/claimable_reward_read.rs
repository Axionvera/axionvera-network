use axionvera_vault_contract::{VaultContract, VaultContractClient, VaultError};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

#[test]
fn claimable_rewards_returns_stored_amount_and_resets_after_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultContract, ());
    let client = VaultContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin, &Address::generate(&env), &Address::generate(&env));

    assert_eq!(client.claimable_rewards(&user), 0);

    client.set_claimable_reward(&user, &10);
    assert_eq!(client.claimable_rewards(&user), 10);

    assert_eq!(client.claim_rewards(&user), 10);
    assert_eq!(client.claimable_rewards(&user), 0);
}

#[test]
fn claimable_rewards_rejected_before_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultContract, ());
    let client = VaultContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    assert_eq!(
        client.try_claimable_rewards(&user).unwrap_err().unwrap(),
        VaultError::NotInitialized
    );
}
