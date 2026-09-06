// MVP demo scenario integration test
// Validates that the vault lifecycle executes faithfully according to the MVP demo scenario fixtures

use axionvera_vault_contract::VaultContract;
use serde_json::Value;
use soroban_sdk::testutils::{Address as _, Events};
use soroban_sdk::{symbol_short, Address, Env, Symbol, TryFromVal, Val, Vec};

const SCENARIO_FIXTURE: &str = include_str!("../../../examples/mvp-demo-scenario/scenario.json");
const TRANSITIONS_FIXTURE: &str =
    include_str!("../../../examples/mvp-demo-scenario/state-transitions.json");
const EVENTS_FIXTURE: &str = include_str!("../../../examples/mvp-demo-scenario/event-outputs.json");

#[test]
fn test_mvp_demo_scenario_fixtures_consistency() {
    let scenario: Value =
        serde_json::from_str(SCENARIO_FIXTURE).expect("scenario.json must be valid JSON");
    assert_eq!(scenario["schema_version"], "1");
    assert_eq!(scenario["steps"].as_array().unwrap().len(), 5);

    let transitions: Value = serde_json::from_str(TRANSITIONS_FIXTURE)
        .expect("state-transitions.json must be valid JSON");
    assert_eq!(transitions["transitions"].as_array().unwrap().len(), 5);

    let events: Value =
        serde_json::from_str(EVENTS_FIXTURE).expect("event-outputs.json must be valid JSON");
    assert_eq!(events["events"].as_array().unwrap().len(), 5);
}

#[test]
fn test_mvp_demo_scenario_execution_flow() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(VaultContract, ());
    let client = axionvera_vault_contract::VaultContractClient::new(&env, &contract_id);

    // Step 1: Initialize
    let admin = Address::generate(&env);
    let deposit_token = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&admin, &deposit_token, &reward_token);

    let init_events = env.events().all();
    assert_eq!(init_events.len(), 1);
    let init_topics: Vec<Val> = init_events.get(0).unwrap().1;
    let init_topic: Symbol = Symbol::try_from_val(&env, &init_topics.get(1).unwrap()).unwrap();
    assert_eq!(init_topic, symbol_short!("init"));

    assert!(client.is_initialized());
    assert_eq!(client.owner(), admin);
    assert_eq!(client.total_deposits(), 0);

    // Step 2: Deposit 1000
    let user_1 = Address::generate(&env);
    let dep_res = client.deposit(&user_1, &1000);
    assert_eq!(dep_res, 1000);

    let dep_events = env.events().all();
    assert_eq!(dep_events.len(), 1);
    let dep_topics: Vec<Val> = dep_events.get(0).unwrap().1;
    let dep_topic: Symbol = Symbol::try_from_val(&env, &dep_topics.get(1).unwrap()).unwrap();
    assert_eq!(dep_topic, symbol_short!("deposit"));

    assert_eq!(client.user_balance(&user_1), 1000);
    assert_eq!(client.total_deposits(), 1000);

    // Step 3: Reward setup
    client.set_reward_balance(&500);
    client.set_claimable_reward(&user_1, &250);
    assert_eq!(client.pending_rewards(&user_1), 500);

    // Step 4: Claim
    let claimed = client.claim_rewards(&user_1);
    assert_eq!(claimed, 250);

    let claim_events = env.events().all();
    assert_eq!(claim_events.len(), 1);
    let claim_topics: Vec<Val> = claim_events.get(0).unwrap().1;
    let claim_topic: Symbol = Symbol::try_from_val(&env, &claim_topics.get(1).unwrap()).unwrap();
    assert_eq!(claim_topic, symbol_short!("claim"));

    // Step 5: Withdraw 400
    let rem_balance = client.withdraw(&user_1, &400);
    assert_eq!(rem_balance, 600);

    let with_events = env.events().all();
    assert_eq!(with_events.len(), 1);
    let with_topics: Vec<Val> = with_events.get(0).unwrap().1;
    let with_topic: Symbol = Symbol::try_from_val(&env, &with_topics.get(1).unwrap()).unwrap();
    assert_eq!(with_topic, symbol_short!("withdraw"));

    assert_eq!(client.user_balance(&user_1), 600);
    assert_eq!(client.total_deposits(), 600);
}
