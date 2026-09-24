use axionvera_campaign_contract::{CampaignContract, CampaignContractClient};

use soroban_sdk::{
    testutils::{Address as _, Events},
    token::StellarAssetClient,
    vec, Address, Env, FromVal, IntoVal, String, Symbol,
};

#[test]
fn initialize_emits_stable_event_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);

    client.initialize(&protocol_admin);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id,
                vec![
                    &env,
                    Symbol::new(&env, "campaign").into_val(&env),
                    Symbol::new(&env, "init").into_val(&env),
                ],
                protocol_admin.into_val(&env),
            )
        ]
    );
}

#[test]
fn create_campaign_emits_stable_event_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Event Shape Campaign"),
        &100,
        &1_000,
        &0,
    );

    assert_eq!(campaign_id, 1);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id,
                vec![
                    &env,
                    Symbol::new(&env, "campaign").into_val(&env),
                    Symbol::new(&env, "created").into_val(&env),
                ],
                (campaign_id, campaign_admin, reward_token).into_val(&env),
            )
        ]
    );
}

#[test]
fn fund_campaign_emits_stable_event_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);

    client.initialize(&protocol_admin);

    let issuer = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(issuer);
    let reward_token = asset.address();

    StellarAssetClient::new(&env, &reward_token).mint(&campaign_admin, &1_000);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Funding Event Campaign"),
        &100,
        &1_000,
        &0,
    );

    let funded = client.fund_campaign(&campaign_id, &250);

    assert_eq!(funded, 250);

    let events = env.events().all();
    let campaign_event = events
        .get(events.len() - 1)
        .expect("campaign funded event must be emitted");

    let (event_contract, topics, data) = campaign_event;

    assert_eq!(event_contract, contract_id);

    assert_eq!(
        topics,
        vec![
            &env,
            Symbol::new(&env, "campaign").into_val(&env),
            Symbol::new(&env, "funded").into_val(&env),
        ]
    );

    let payload = <(u64, Address, i128)>::from_val(&env, &data);

    assert_eq!(payload, (campaign_id, campaign_admin, 250_i128));
}

#[test]
fn reward_allocation_emits_stable_event_shape() {
    use soroban_sdk::testutils::Ledger;

    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(100);

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let verifier = Address::generate(&env);
    let agent = Address::generate(&env);

    client.initialize(&protocol_admin);

    let issuer = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(issuer);
    let reward_token = asset.address();

    StellarAssetClient::new(&env, &reward_token).mint(&campaign_admin, &1_000);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Activation Event Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.fund_campaign(&campaign_id, &100);
    client.add_verifier(&campaign_id, &verifier);

    let milestone = String::from_str(&env, "KYC_VERIFIED");
    let merchant_ref = String::from_str(&env, "merchant-event-001");

    client.add_activation_rule(&campaign_id, &milestone, &2);

    let reward = client.verify_and_allocate_reward(
        &campaign_id,
        &verifier,
        &agent,
        &merchant_ref,
        &milestone,
    );

    assert_eq!(reward, 2);

    let events = env.events().all();

    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id,
                vec![
                    &env,
                    Symbol::new(&env, "campaign").into_val(&env),
                    Symbol::new(&env, "activate").into_val(&env),
                    Symbol::new(&env, "reward").into_val(&env),
                ],
                (
                    campaign_id,
                    verifier,
                    agent,
                    merchant_ref,
                    milestone,
                    2_i128,
                )
                    .into_val(&env),
            )
        ]
    );
}

#[test]
fn reward_claim_emits_stable_event_shape() {
    use soroban_sdk::testutils::Ledger;

    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(100);

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let verifier = Address::generate(&env);
    let agent = Address::generate(&env);

    client.initialize(&protocol_admin);

    let issuer = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(issuer);
    let reward_token = asset.address();

    StellarAssetClient::new(&env, &reward_token).mint(&campaign_admin, &1_000);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Claim Event Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.fund_campaign(&campaign_id, &100);
    client.add_verifier(&campaign_id, &verifier);

    let milestone = String::from_str(&env, "KYC_VERIFIED");
    let merchant_ref = String::from_str(&env, "merchant-claim-001");

    client.add_activation_rule(&campaign_id, &milestone, &2);

    client.verify_and_allocate_reward(&campaign_id, &verifier, &agent, &merchant_ref, &milestone);

    let claimed = client.claim_reward(&campaign_id, &agent);

    assert_eq!(claimed, 2);

    let events = env.events().all();
    let campaign_event = events
        .get(events.len() - 1)
        .expect("campaign reward claim event must be emitted");

    let (event_contract, topics, data) = campaign_event;

    assert_eq!(event_contract, contract_id);

    assert_eq!(
        topics,
        vec![
            &env,
            Symbol::new(&env, "campaign").into_val(&env),
            Symbol::new(&env, "reward").into_val(&env),
            Symbol::new(&env, "claim").into_val(&env),
        ]
    );

    let payload = <(u64, Address, i128)>::from_val(&env, &data);

    assert_eq!(payload, (campaign_id, agent, 2_i128));
}

#[test]
fn unused_fund_withdrawal_emits_stable_event_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);

    client.initialize(&protocol_admin);

    let issuer = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(issuer);
    let reward_token = asset.address();

    StellarAssetClient::new(&env, &reward_token).mint(&campaign_admin, &1_000);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Withdrawal Event Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.fund_campaign(&campaign_id, &100);
    client.close_campaign(&campaign_id);

    let remaining = client.withdraw_unused_funds(&campaign_id, &40);

    assert_eq!(remaining, 60);

    let events = env.events().all();
    let campaign_event = events
        .get(events.len() - 1)
        .expect("unused fund withdrawal event must be emitted");

    let (event_contract, topics, data) = campaign_event;

    assert_eq!(event_contract, contract_id);

    assert_eq!(
        topics,
        vec![
            &env,
            Symbol::new(&env, "campaign").into_val(&env),
            Symbol::new(&env, "unused").into_val(&env),
            Symbol::new(&env, "withdraw").into_val(&env),
        ]
    );

    let payload = <(u64, Address, i128)>::from_val(&env, &data);

    assert_eq!(payload, (campaign_id, campaign_admin, 40_i128));
}

#[test]
fn pause_campaign_emits_stable_event_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Pause Event Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.pause_campaign(&campaign_id);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id,
                vec![
                    &env,
                    Symbol::new(&env, "campaign").into_val(&env),
                    Symbol::new(&env, "paused").into_val(&env),
                ],
                (campaign_id, campaign_admin).into_val(&env),
            )
        ]
    );
}

#[test]
fn resume_campaign_emits_stable_event_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Resume Event Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.pause_campaign(&campaign_id);
    client.resume_campaign(&campaign_id);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id,
                vec![
                    &env,
                    Symbol::new(&env, "campaign").into_val(&env),
                    Symbol::new(&env, "resumed").into_val(&env),
                ],
                (campaign_id, campaign_admin).into_val(&env),
            )
        ]
    );
}

#[test]
fn close_campaign_emits_stable_event_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Close Event Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.close_campaign(&campaign_id);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id,
                vec![
                    &env,
                    Symbol::new(&env, "campaign").into_val(&env),
                    Symbol::new(&env, "closed").into_val(&env),
                ],
                (campaign_id, campaign_admin).into_val(&env),
            )
        ]
    );
}

#[test]
fn activation_rule_added_emits_stable_event_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Rule Event Campaign"),
        &100,
        &1_000,
        &0,
    );

    let milestone = String::from_str(&env, "KYC_VERIFIED");

    client.add_activation_rule(&campaign_id, &milestone, &2);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id,
                vec![
                    &env,
                    Symbol::new(&env, "campaign").into_val(&env),
                    Symbol::new(&env, "rule").into_val(&env),
                    Symbol::new(&env, "added").into_val(&env),
                ],
                (campaign_id, milestone, 2_i128).into_val(&env),
            )
        ]
    );
}

#[test]
fn verifier_added_emits_stable_event_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);
    let verifier = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Verifier Event Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.add_verifier(&campaign_id, &verifier);

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id,
                vec![
                    &env,
                    Symbol::new(&env, "campaign").into_val(&env),
                    Symbol::new(&env, "verifyr").into_val(&env),
                    Symbol::new(&env, "added").into_val(&env),
                ],
                (campaign_id, verifier).into_val(&env),
            )
        ]
    );
}

#[test]
fn verifier_removed_emits_stable_event_shape() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);
    let verifier = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Verifier Removal Event Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.add_verifier(&campaign_id, &verifier);
    client.remove_verifier(&campaign_id, &verifier);

    let events = env.events().all();
    let campaign_event = events
        .get(events.len() - 1)
        .expect("campaign verifier removed event must be emitted");

    let (event_contract, topics, data) = campaign_event;

    assert_eq!(event_contract, contract_id);

    assert_eq!(
        topics,
        vec![
            &env,
            Symbol::new(&env, "campaign").into_val(&env),
            Symbol::new(&env, "verifyr").into_val(&env),
            Symbol::new(&env, "removed").into_val(&env),
        ]
    );

    let payload = <(u64, Address)>::from_val(&env, &data);
    assert_eq!(payload, (campaign_id, verifier));
}
