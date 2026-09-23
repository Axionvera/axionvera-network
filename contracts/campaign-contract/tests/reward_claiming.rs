use axionvera_campaign_contract::{CampaignContract, CampaignContractClient, CampaignError};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, Env, String,
};

struct Setup {
    env: Env,
    client: CampaignContractClient<'static>,
    contract_id: Address,
    campaign_id: u64,
    verifier: Address,
    agent: Address,
    token_address: Address,
}

fn setup() -> Setup {
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

    StellarAssetClient::new(&env, &token_address).mint(&campaign_admin, &1_000);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &token_address,
        &String::from_str(&env, "Ibadan Merchant Activation"),
        &100,
        &1_000,
        &0,
    );

    client.fund_campaign(&campaign_id, &100);
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
        contract_id,
        campaign_id,
        verifier,
        agent,
        token_address,
    }
}

#[test]
fn claim_transfers_real_tokens_to_agent() {
    let s = setup();

    let token = TokenClient::new(&s.env, &s.token_address);

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(token.balance(&s.agent), 0);
    assert_eq!(token.balance(&s.contract_id), 100);

    let claimed = s.client.claim_reward(&s.campaign_id, &s.agent);

    assert_eq!(claimed, 2);

    assert_eq!(token.balance(&s.agent), 2);
    assert_eq!(token.balance(&s.contract_id), 98);

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 0);

    let campaign = s.client.get_campaign(&s.campaign_id);

    assert_eq!(campaign.funded_amount, 100);
    assert_eq!(campaign.allocated_amount, 2);
    assert_eq!(campaign.claimed_amount, 2);
}

#[test]
fn multiple_allocations_can_be_claimed_together() {
    let s = setup();

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

    let claimed = s.client.claim_reward(&s.campaign_id, &s.agent);

    assert_eq!(claimed, 5);

    let token = TokenClient::new(&s.env, &s.token_address);

    assert_eq!(token.balance(&s.agent), 5);
    assert_eq!(token.balance(&s.contract_id), 95);

    let campaign = s.client.get_campaign(&s.campaign_id);

    assert_eq!(campaign.allocated_amount, 5);
    assert_eq!(campaign.claimed_amount, 5);

    assert_eq!(s.client.agent_total_earned(&s.campaign_id, &s.agent), 5);
}

#[test]
fn repeated_claim_without_new_rewards_is_rejected() {
    let s = setup();

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    s.client.claim_reward(&s.campaign_id, &s.agent);

    let result = s.client.try_claim_reward(&s.campaign_id, &s.agent);

    assert_eq!(result, Err(Ok(CampaignError::NothingToClaim)));

    let token = TokenClient::new(&s.env, &s.token_address);

    assert_eq!(token.balance(&s.agent), 2);
}

#[test]
fn different_agents_have_independent_claimable_balances() {
    let s = setup();

    let second_agent = Address::generate(&s.env);

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
        &second_agent,
        &String::from_str(&s.env, "merchant-002"),
        &String::from_str(&s.env, "FIRST_TRANSACTION"),
    );

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 2);

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &second_agent), 3);

    s.client.claim_reward(&s.campaign_id, &s.agent);

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 0);

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &second_agent), 3);
}

#[test]
fn new_rewards_can_be_claimed_after_previous_claim() {
    let s = setup();

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(s.client.claim_reward(&s.campaign_id, &s.agent), 2);

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "FIRST_TRANSACTION"),
    );

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 3);

    assert_eq!(s.client.claim_reward(&s.campaign_id, &s.agent), 3);

    let token = TokenClient::new(&s.env, &s.token_address);

    assert_eq!(token.balance(&s.agent), 5);

    let campaign = s.client.get_campaign(&s.campaign_id);

    assert_eq!(campaign.allocated_amount, 5);
    assert_eq!(campaign.claimed_amount, 5);

    assert_eq!(s.client.agent_total_earned(&s.campaign_id, &s.agent), 5);
}

#[test]
fn agent_with_no_reward_cannot_claim() {
    let s = setup();

    let result = s.client.try_claim_reward(&s.campaign_id, &s.agent);

    assert_eq!(result, Err(Ok(CampaignError::NothingToClaim)));
}
