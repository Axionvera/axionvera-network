use axionvera_vault_contract::{VaultContract, VaultContractClient};
use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal, Val, Vec};

fn authorize(
    env: &Env,
    contract_id: &Address,
    signer: &Address,
    fn_name: &'static str,
    args: Vec<Val>,
) {
    env.mock_auths(&[MockAuth {
        address: signer,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name,
            args,
            sub_invokes: &[],
        },
    }]);
}

fn setup_initialized_vault() -> (Env, Address, VaultContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register(VaultContract, ());
    let client = VaultContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let deposit_token = Address::generate(&env);
    let reward_token = Address::generate(&env);

    authorize(
        &env,
        &contract_id,
        &admin,
        "initialize",
        (&admin, &deposit_token, &reward_token).into_val(&env),
    );

    client.initialize(&admin, &deposit_token, &reward_token);

    (env, contract_id, client, admin)
}

#[test]
fn admin_can_set_reward_balance() {
    let (env, contract_id, client, admin) = setup_initialized_vault();

    authorize(
        &env,
        &contract_id,
        &admin,
        "set_reward_balance",
        (500_i128,).into_val(&env),
    );

    client.set_reward_balance(&500);
}

#[test]
fn admin_can_set_claimable_reward() {
    let (env, contract_id, client, admin) = setup_initialized_vault();
    let user = Address::generate(&env);

    authorize(
        &env,
        &contract_id,
        &admin,
        "set_claimable_reward",
        (&user, &250_i128).into_val(&env),
    );

    client.set_claimable_reward(&user, &250);

    assert_eq!(client.pending_rewards(&user), 250);
}

#[test]
fn non_admin_cannot_set_reward_balance() {
    let (env, contract_id, client, _admin) = setup_initialized_vault();
    let non_admin = Address::generate(&env);

    authorize(
        &env,
        &contract_id,
        &non_admin,
        "set_reward_balance",
        (500_i128,).into_val(&env),
    );

    assert!(client.try_set_reward_balance(&500).is_err());
}

#[test]
fn non_admin_cannot_set_claimable_reward() {
    let (env, contract_id, client, _admin) = setup_initialized_vault();
    let non_admin = Address::generate(&env);
    let user = Address::generate(&env);

    authorize(
        &env,
        &contract_id,
        &non_admin,
        "set_claimable_reward",
        (&user, &250_i128).into_val(&env),
    );

    assert!(client.try_set_claimable_reward(&user, &250).is_err());
    assert_eq!(client.pending_rewards(&user), 0);
}
