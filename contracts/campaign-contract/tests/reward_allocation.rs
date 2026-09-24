use axionvera_campaign_contract::{CampaignContract, CampaignContractClient, CampaignError};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, String,
};

struct Setup {
    env: Env,
    client: CampaignContractClient<'static>,
    campaign_id: u64,
    verifier: Address,
    agent: Address,
}

fn setup(funding: i128, cap: i128) -> Setup {
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
    let token_address = asset.address();

    StellarAssetClient::new(&env, &token_address).mint(&campaign_admin, &10_000);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &token_address,
        &String::from_str(&env, "Ibadan Merchant Activation"),
        &100,
        &1_000,
        &cap,
    );

    if funding > 0 {
        client.fund_campaign(&campaign_id, &funding);
    }

    client.add_verifier(&campaign_id, &verifier);

    client.add_activation_rule(&campaign_id, &String::from_str(&env, "KYC_VERIFIED"), &2);

    client.add_activation_rule(
        &campaign_id,
        &String::from_str(&env, "FIRST_TRANSACTION"),
        &3,
    );

    let client: CampaignContractClient<'static> = unsafe { core::mem::transmute(client) };

    Setup {
        env,
        client,
        campaign_id,
        verifier,
        agent,
    }
}

#[test]
fn verifier_allocates_reward_to_agent() {
    let s = setup(100, 0);

    let reward = s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(reward, 2);

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 2);

    assert_eq!(s.client.agent_total_earned(&s.campaign_id, &s.agent), 2);

    let campaign = s.client.get_campaign(&s.campaign_id);

    assert_eq!(campaign.funded_amount, 100);
    assert_eq!(campaign.allocated_amount, 2);
    assert_eq!(campaign.claimed_amount, 0);
}

#[test]
fn multiple_milestones_accumulate_rewards() {
    let s = setup(100, 0);

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "FIRST_TRANSACTION"),
    );

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 5);

    assert_eq!(s.client.agent_total_earned(&s.campaign_id, &s.agent), 5);

    assert_eq!(s.client.get_campaign(&s.campaign_id).allocated_amount, 5);
}

#[test]
fn duplicate_merchant_milestone_is_rejected() {
    let s = setup(100, 0);

    let merchant = String::from_str(&s.env, "merchant-001");

    let milestone = String::from_str(&s.env, "KYC_VERIFIED");

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &merchant,
        &milestone,
    );

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &merchant,
        &milestone,
    );

    assert_eq!(result, Err(Ok(CampaignError::DuplicateActivation)));

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 2);
}

#[test]
fn unauthorized_verifier_is_rejected() {
    let s = setup(100, 0);

    let unauthorized = Address::generate(&s.env);

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &unauthorized,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(result, Err(Ok(CampaignError::UnauthorizedVerifier)));

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 0);
}

#[test]
fn insufficient_campaign_budget_is_rejected() {
    let s = setup(1, 0);

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(result, Err(Ok(CampaignError::InsufficientCampaignFunds)));

    assert_eq!(s.client.get_campaign(&s.campaign_id).allocated_amount, 0);
}

#[test]
fn per_agent_cap_is_enforced() {
    let s = setup(100, 4);

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "FIRST_TRANSACTION"),
    );

    assert_eq!(result, Err(Ok(CampaignError::AgentCapExceeded)));

    assert_eq!(s.client.agent_total_earned(&s.campaign_id, &s.agent), 2);

    assert_eq!(s.client.get_campaign(&s.campaign_id).allocated_amount, 2);
}

#[test]
fn same_milestone_can_reward_different_merchants() {
    let s = setup(100, 0);

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-002"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 4);

    assert_eq!(s.client.agent_total_earned(&s.campaign_id, &s.agent), 4);
}

#[test]
fn unknown_activation_rule_is_rejected() {
    let s = setup(100, 0);

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "UNKNOWN"),
    );

    assert_eq!(result, Err(Ok(CampaignError::RuleNotFound)));
}

#[test]
fn registered_verifier_must_authorize_reward_allocation() {
    let s = setup(100, 0);

    let merchant = String::from_str(&s.env, "merchant-auth-test");
    let milestone = String::from_str(&s.env, "KYC_VERIFIED");

    // The verifier is registered by setup(), but no account
    // authorisation is provided for this invocation.
    s.env.set_auths(&[]);

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &merchant,
        &milestone,
    );

    assert!(result.is_err());

    // Failed authentication must not allocate or reserve any reward.
    s.env.mock_all_auths();

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 0);
    assert_eq!(s.client.agent_total_earned(&s.campaign_id, &s.agent), 0);
    assert_eq!(s.client.get_campaign(&s.campaign_id).allocated_amount, 0);

    // The failed attempt must also not consume the merchant/milestone pair.
    let reward = s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &merchant,
        &milestone,
    );

    assert_eq!(reward, 2);
}
