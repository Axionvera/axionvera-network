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
    campaign_admin: Address,
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

    let client: CampaignContractClient<'static> = unsafe { core::mem::transmute(client) };

    Setup {
        env,
        client,
        contract_id,
        campaign_id,
        campaign_admin,
        verifier,
        agent,
        token_address,
    }
}

#[test]
fn available_unused_funds_excludes_allocated_rewards() {
    let s = setup();

    assert_eq!(s.client.available_unused_funds(&s.campaign_id), 100);

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(s.client.available_unused_funds(&s.campaign_id), 98);
}

#[test]
fn cannot_withdraw_unused_funds_before_campaign_is_closed() {
    let s = setup();

    let result = s.client.try_withdraw_unused_funds(&s.campaign_id, &10);

    assert_eq!(result, Err(Ok(CampaignError::CampaignNotClosed)));
}

#[test]
fn closed_campaign_admin_can_withdraw_unused_funds() {
    let s = setup();

    let token = TokenClient::new(&s.env, &s.token_address);

    assert_eq!(token.balance(&s.campaign_admin), 900);

    assert_eq!(token.balance(&s.contract_id), 100);

    s.client.close_campaign(&s.campaign_id);

    let remaining = s.client.withdraw_unused_funds(&s.campaign_id, &40);

    assert_eq!(remaining, 60);

    assert_eq!(token.balance(&s.campaign_admin), 940);

    assert_eq!(token.balance(&s.contract_id), 60);

    let campaign = s.client.get_campaign(&s.campaign_id);

    assert_eq!(campaign.funded_amount, 100);
    assert_eq!(campaign.withdrawn_amount, 40);
}

#[test]
fn allocated_agent_rewards_cannot_be_withdrawn_by_admin() {
    let s = setup();

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    s.client.close_campaign(&s.campaign_id);

    assert_eq!(s.client.available_unused_funds(&s.campaign_id), 98);

    let result = s.client.try_withdraw_unused_funds(&s.campaign_id, &99);

    assert_eq!(result, Err(Ok(CampaignError::InsufficientUnusedFunds)));

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 2);
}

#[test]
fn admin_can_withdraw_all_unused_while_agent_reward_remains_reserved() {
    let s = setup();

    let token = TokenClient::new(&s.env, &s.token_address);

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    s.client.close_campaign(&s.campaign_id);

    let remaining = s.client.withdraw_unused_funds(&s.campaign_id, &98);

    assert_eq!(remaining, 0);

    // Only the agent's allocated reward remains in the contract.
    assert_eq!(token.balance(&s.contract_id), 2);

    assert_eq!(token.balance(&s.campaign_admin), 998);

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 2);

    // Agent can still claim the reserved amount.
    let claimed = s.client.claim_reward(&s.campaign_id, &s.agent);

    assert_eq!(claimed, 2);

    assert_eq!(token.balance(&s.agent), 2);

    assert_eq!(token.balance(&s.contract_id), 0);

    let campaign = s.client.get_campaign(&s.campaign_id);

    assert_eq!(campaign.funded_amount, 100);
    assert_eq!(campaign.allocated_amount, 2);
    assert_eq!(campaign.claimed_amount, 2);
    assert_eq!(campaign.withdrawn_amount, 98);
}

#[test]
fn cannot_withdraw_more_than_remaining_unused_balance() {
    let s = setup();

    s.client.close_campaign(&s.campaign_id);

    assert_eq!(s.client.withdraw_unused_funds(&s.campaign_id, &60,), 40);

    let result = s.client.try_withdraw_unused_funds(&s.campaign_id, &41);

    assert_eq!(result, Err(Ok(CampaignError::InsufficientUnusedFunds)));

    assert_eq!(s.client.available_unused_funds(&s.campaign_id), 40);
}

#[test]
fn zero_or_negative_unused_withdrawal_is_rejected() {
    let s = setup();

    s.client.close_campaign(&s.campaign_id);

    assert_eq!(
        s.client.try_withdraw_unused_funds(&s.campaign_id, &0,),
        Err(Ok(CampaignError::InvalidAmount))
    );

    assert_eq!(
        s.client.try_withdraw_unused_funds(&s.campaign_id, &-1,),
        Err(Ok(CampaignError::InvalidAmount))
    );
}
