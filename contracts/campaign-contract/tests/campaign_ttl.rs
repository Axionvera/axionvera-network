use axionvera_campaign_contract::{CampaignContract, CampaignContractClient};

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, String,
};

#[test]
fn campaign_and_instance_state_survive_beyond_minimum_ttl() {
    let env = Env::default();
    env.mock_all_auths();

    // Use deliberately small TTL values so the test can simulate ageing
    // without advancing millions of ledgers.
    env.ledger().set_sequence_number(1);
    env.ledger().set_min_persistent_entry_ttl(5);
    env.ledger().set_max_entry_ttl(100);

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "TTL Test Campaign"),
        &100,
        &1_000,
        &0,
    );

    assert!(client.is_initialized());
    assert_eq!(client.get_campaign(&campaign_id).id, campaign_id);

    // Move well beyond the minimum persistent TTL of 5 ledgers.
    env.ledger().set_sequence_number(20);

    // Axionvera should keep both its contract instance and Campaign record
    // alive beyond the network minimum TTL.
    assert!(client.is_initialized());

    let campaign = client.get_campaign(&campaign_id);
    assert_eq!(campaign.id, campaign_id);
    assert_eq!(campaign.admin, campaign_admin);
}

#[test]
fn allocated_reward_survives_beyond_minimum_ttl_until_claimed() {
    use soroban_sdk::token::StellarAssetClient;

    let env = Env::default();
    env.mock_all_auths();

    env.ledger().set_sequence_number(1);
    env.ledger().set_timestamp(100);
    env.ledger().set_min_persistent_entry_ttl(5);
    env.ledger().set_max_entry_ttl(100);

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
        &String::from_str(&env, "Claimable TTL Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.fund_campaign(&campaign_id, &100);
    client.add_verifier(&campaign_id, &verifier);

    let milestone = String::from_str(&env, "KYC_VERIFIED");

    client.add_activation_rule(&campaign_id, &milestone, &2);

    client.verify_and_allocate_reward(
        &campaign_id,
        &verifier,
        &agent,
        &String::from_str(&env, "merchant-ttl-claim"),
        &milestone,
    );

    assert_eq!(client.claimable_reward(&campaign_id, &agent), 2);

    // Move beyond the minimum persistent TTL.
    env.ledger().set_sequence_number(20);

    // The already allocated reward must remain claimable.
    assert_eq!(client.claim_reward(&campaign_id, &agent), 2);
}

#[test]
fn agent_total_earned_survives_beyond_minimum_ttl() {
    use soroban_sdk::token::StellarAssetClient;

    let env = Env::default();
    env.mock_all_auths();

    env.ledger().set_sequence_number(1);
    env.ledger().set_timestamp(100);
    env.ledger().set_min_persistent_entry_ttl(5);
    env.ledger().set_max_entry_ttl(100);

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
        &String::from_str(&env, "Earned TTL Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.fund_campaign(&campaign_id, &100);
    client.add_verifier(&campaign_id, &verifier);

    let milestone = String::from_str(&env, "KYC_VERIFIED");

    client.add_activation_rule(&campaign_id, &milestone, &2);

    client.verify_and_allocate_reward(
        &campaign_id,
        &verifier,
        &agent,
        &String::from_str(&env, "merchant-earned-ttl"),
        &milestone,
    );

    assert_eq!(client.agent_total_earned(&campaign_id, &agent), 2);

    // Move beyond the minimum persistent TTL.
    env.ledger().set_sequence_number(20);

    // Historical earnings accounting must remain available.
    assert_eq!(client.agent_total_earned(&campaign_id, &agent), 2);
}

#[test]
fn processed_activation_survives_beyond_minimum_ttl() {
    use soroban_sdk::token::StellarAssetClient;

    let env = Env::default();
    env.mock_all_auths();

    env.ledger().set_sequence_number(1);
    env.ledger().set_timestamp(100);
    env.ledger().set_min_persistent_entry_ttl(5);
    env.ledger().set_max_entry_ttl(100);

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
        &String::from_str(&env, "Duplicate Protection TTL Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.fund_campaign(&campaign_id, &100);
    client.add_verifier(&campaign_id, &verifier);

    let milestone = String::from_str(&env, "KYC_VERIFIED");
    let merchant_ref = String::from_str(&env, "merchant-duplicate-ttl");

    client.add_activation_rule(&campaign_id, &milestone, &2);

    assert_eq!(
        client.verify_and_allocate_reward(
            &campaign_id,
            &verifier,
            &agent,
            &merchant_ref,
            &milestone,
        ),
        2
    );

    // Move beyond the minimum persistent TTL.
    env.ledger().set_sequence_number(20);

    // The original processed marker must still prevent the same
    // merchant milestone from being rewarded twice.
    let result = client.try_verify_and_allocate_reward(
        &campaign_id,
        &verifier,
        &agent,
        &merchant_ref,
        &milestone,
    );

    assert_eq!(
        result,
        Err(Ok(
            axionvera_campaign_contract::CampaignError::DuplicateActivation
        ))
    );

    assert_eq!(client.claimable_reward(&campaign_id, &agent), 2);
}

#[test]
fn accessing_campaign_refreshes_its_ttl_for_long_running_use() {
    let env = Env::default();
    env.mock_all_auths();

    env.ledger().set_sequence_number(1);
    env.ledger().set_min_persistent_entry_ttl(5);
    env.ledger().set_max_entry_ttl(100);

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Long Running TTL Campaign"),
        &100,
        &1_000,
        &0,
    );

    // The Campaign entry was originally extended close to the configured
    // maximum TTL of 100 ledgers.
    assert_eq!(client.get_campaign(&campaign_id).id, campaign_id);

    // Move past half of that lifetime while the entry is still accessible.
    env.ledger().set_sequence_number(60);

    // Accessing an active Campaign should refresh its remaining lifetime.
    assert_eq!(client.get_campaign(&campaign_id).id, campaign_id);

    // This is beyond the Campaign's original lifetime from ledger 1.
    // It should still exist because the access at ledger 60 refreshed it.
    env.ledger().set_sequence_number(120);

    let campaign = client.get_campaign(&campaign_id);

    assert_eq!(campaign.id, campaign_id);
    assert_eq!(campaign.admin, campaign_admin);
}

#[test]
fn allocation_dependencies_refresh_during_long_running_campaign() {
    use soroban_sdk::token::StellarAssetClient;

    let env = Env::default();
    env.mock_all_auths();

    env.ledger().set_sequence_number(1);
    env.ledger().set_timestamp(100);
    env.ledger().set_min_persistent_entry_ttl(5);
    env.ledger().set_max_entry_ttl(100);

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
        &String::from_str(&env, "Long Running Allocation Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.fund_campaign(&campaign_id, &100);
    client.add_verifier(&campaign_id, &verifier);

    let milestone = String::from_str(&env, "KYC_VERIFIED");
    client.add_activation_rule(&campaign_id, &milestone, &2);

    client.verify_and_allocate_reward(
        &campaign_id,
        &verifier,
        &agent,
        &String::from_str(&env, "merchant-long-running-1"),
        &milestone,
    );

    // Still within the original TTL, but below the refresh threshold.
    // This successful allocation should refresh all dependencies it uses.
    env.ledger().set_sequence_number(60);

    assert_eq!(
        client.verify_and_allocate_reward(
            &campaign_id,
            &verifier,
            &agent,
            &String::from_str(&env, "merchant-long-running-2"),
            &milestone,
        ),
        2
    );

    // Beyond the original lifetime from ledger 1. A third successful
    // activation should still work if the ledger-60 use refreshed the
    // verifier and activation rule as well as the campaign/instance.
    env.ledger().set_sequence_number(120);

    assert_eq!(
        client.verify_and_allocate_reward(
            &campaign_id,
            &verifier,
            &agent,
            &String::from_str(&env, "merchant-long-running-3"),
            &milestone,
        ),
        2
    );
}

#[test]
fn is_initialized_refreshes_instance_ttl_for_long_running_use() {
    let env = Env::default();
    env.mock_all_auths();

    env.ledger().set_sequence_number(1);
    env.ledger().set_min_persistent_entry_ttl(5);
    env.ledger().set_max_entry_ttl(100);

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);

    client.initialize(&protocol_admin);

    assert!(client.is_initialized());

    // Still alive, but now below the refresh threshold.
    env.ledger().set_sequence_number(60);

    // A successful initialization-status read should refresh the instance.
    assert!(client.is_initialized());

    // Beyond the instance's original lifetime from ledger 1.
    env.ledger().set_sequence_number(120);

    assert!(client.is_initialized());
}
