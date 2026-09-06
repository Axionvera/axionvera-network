// Mocked post-deployment smoke test flow
// Validates contract ID resolution, initialization input loading,
// read verification, and sequential lifecycle expectation checks in a safe mock environment.

use axionvera_vault_contract::VaultContract;
use serde_json::Value;
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{symbol_short, Address, Env, Symbol, TryFromVal, Val, Vec};

const REGISTRY_FIXTURE: &str = include_str!("../../../examples/contract-id-registry.json");
const INIT_INPUT_FIXTURE: &str = include_str!("../../../examples/vault-initialization-input.json");
const SMOKE_REPORT_FIXTURE: &str =
    include_str!("../../../examples/mock-post-deployment-smoke-output.json");

#[test]
fn test_fixture_loading_and_contract_id_resolution() {
    let registry: Value = serde_json::from_str(REGISTRY_FIXTURE)
        .expect("contract-id-registry.json must be valid JSON");
    assert_eq!(registry["schema_version"], "1");
    assert_eq!(
        registry["environments"]["testnet"]["contracts"]["axionvera_vault_contract"]["contract_id"],
        "CONTRACT_ID_PLACEHOLDER"
    );

    let init_input: Value = serde_json::from_str(INIT_INPUT_FIXTURE)
        .expect("vault-initialization-input.json must be valid JSON");
    assert_eq!(init_input["schema_version"], "1");
    assert_eq!(init_input["network"], "testnet");
    assert_eq!(init_input["contract_id"], "CONTRACT_ID_PLACEHOLDER");
    assert_eq!(
        init_input["maintainer_initialization_boundary"]["required"],
        true
    );

    let smoke_report: Value = serde_json::from_str(SMOKE_REPORT_FIXTURE)
        .expect("mock-post-deployment-smoke-output.json must be valid JSON");
    assert_eq!(smoke_report["overall_status"], "PASSED");
    assert_eq!(
        smoke_report["maintainer_smoke_boundary"]["no_live_rpc_used"],
        true
    );
}

#[test]
fn test_mocked_post_deployment_smoke_flow_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultContract, ());
    let client = axionvera_vault_contract::VaultContractClient::new(&env, &contract_id);

    // Stage 1: Pre-initialization check
    assert!(
        !client.is_initialized(),
        "Vault must not be initialized before setup"
    );

    // Stage 2: Initialization with parameters matching the initialization fixture roles
    let admin = Address::generate(&env);
    let deposit_token = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&admin, &deposit_token, &reward_token);

    // Check init event immediately after invocation
    let init_events = env.events().all();
    assert_eq!(init_events.len(), 1, "initialize must emit 1 event");
    let init_topics: Vec<Val> = init_events.get(0).unwrap().1;
    let init_topic: Symbol = Symbol::try_from_val(&env, &init_topics.get(1).unwrap()).unwrap();
    assert_eq!(init_topic, symbol_short!("init"));

    // Stage 3: Post-deployment read verification
    assert!(client.is_initialized(), "Vault must report initialized");
    assert_eq!(client.owner(), admin, "Admin must match initialized owner");
    assert_eq!(
        client.deposit_token(),
        deposit_token,
        "Deposit token must match"
    );
    assert_eq!(
        client.reward_token(),
        reward_token,
        "Reward token must match"
    );
    assert_eq!(
        client.total_deposits(),
        0,
        "Initial total deposits must be 0"
    );

    // Stage 4: Deposit accounting simulation
    let user = Address::generate(&env);
    let deposit_amount: i128 = 1000;

    let new_bal = client.deposit(&user, &deposit_amount);
    assert_eq!(new_bal, deposit_amount);

    let dep_events = env.events().all();
    assert_eq!(dep_events.len(), 1, "deposit must emit 1 event");
    let dep_topics: Vec<Val> = dep_events.get(0).unwrap().1;
    let dep_topic: Symbol = Symbol::try_from_val(&env, &dep_topics.get(1).unwrap()).unwrap();
    assert_eq!(dep_topic, symbol_short!("deposit"));

    assert_eq!(
        client.user_balance(&user),
        deposit_amount,
        "User balance mismatch after deposit"
    );
    assert_eq!(
        client.total_deposits(),
        deposit_amount,
        "Total deposits mismatch after deposit"
    );

    // Stage 5: Reward allocation and claim simulation
    let reward_funding: i128 = 500;
    let user_claimable: i128 = 250;

    client.set_reward_balance(&reward_funding);
    client.set_claimable_reward(&user, &user_claimable);
    assert_eq!(
        client.pending_rewards(&user),
        500,
        "Pending rewards proportional to share"
    );

    let claimed = client.claim_rewards(&user);
    assert_eq!(claimed, user_claimable, "Claimed amount mismatch");

    let claim_events = env.events().all();
    assert_eq!(claim_events.len(), 1, "claim must emit 1 event");
    let claim_topics: Vec<Val> = claim_events.get(0).unwrap().1;
    let claim_topic: Symbol = Symbol::try_from_val(&env, &claim_topics.get(1).unwrap()).unwrap();
    assert_eq!(claim_topic, symbol_short!("claim"));

    // Stage 6: Withdrawal and balance reconciliation
    let rem_bal = client.withdraw(&user, &deposit_amount);
    assert_eq!(rem_bal, 0);

    let with_events = env.events().all();
    assert_eq!(with_events.len(), 1, "withdraw must emit 1 event");
    let with_topics: Vec<Val> = with_events.get(0).unwrap().1;
    let with_topic: Symbol = Symbol::try_from_val(&env, &with_topics.get(1).unwrap()).unwrap();
    assert_eq!(with_topic, symbol_short!("withdraw"));

    assert_eq!(
        client.user_balance(&user),
        0,
        "User balance must be 0 after full withdrawal"
    );
    assert_eq!(
        client.total_deposits(),
        0,
        "Total deposits must return to 0"
    );
}
