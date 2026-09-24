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

fn setup() -> Setup {
    let env = Env::default();
    env.mock_all_auths();

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
        campaign_id,
        verifier,
        agent,
    }
}

#[test]
fn allocation_before_start_is_rejected() {
    let s = setup();

    s.env.ledger().set_timestamp(99);

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(result, Err(Ok(CampaignError::CampaignNotStarted)));
}

#[test]
fn allocation_exactly_at_start_is_allowed() {
    let s = setup();

    s.env.ledger().set_timestamp(100);

    let reward = s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(reward, 2);
}

#[test]
fn allocation_just_before_end_is_allowed() {
    let s = setup();

    s.env.ledger().set_timestamp(999);

    let reward = s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(reward, 2);
}

#[test]
fn allocation_exactly_at_end_is_rejected() {
    let s = setup();

    s.env.ledger().set_timestamp(1_000);

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(result, Err(Ok(CampaignError::CampaignEnded)));
}

#[test]
fn allocation_after_end_is_rejected() {
    let s = setup();

    s.env.ledger().set_timestamp(1_001);

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(result, Err(Ok(CampaignError::CampaignEnded)));
}

#[test]
fn already_allocated_reward_can_be_claimed_after_campaign_ends() {
    let s = setup();

    s.env.ledger().set_timestamp(100);

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    s.env.ledger().set_timestamp(1_001);

    let claimed = s.client.claim_reward(&s.campaign_id, &s.agent);

    assert_eq!(claimed, 2);

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 0);
}
