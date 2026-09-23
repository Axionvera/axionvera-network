use axionvera_campaign_contract::{
    CampaignContract, CampaignContractClient, CampaignError, CampaignStatus,
};
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
fn campaign_can_be_paused_and_resumed() {
    let s = setup();

    s.client.pause_campaign(&s.campaign_id);

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Paused
    );

    s.client.resume_campaign(&s.campaign_id);

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Active
    );
}

#[test]
fn paused_campaign_rejects_new_reward_allocations() {
    let s = setup();

    s.client.pause_campaign(&s.campaign_id);

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(result, Err(Ok(CampaignError::CampaignNotActive)));

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 0);
}

#[test]
fn allocation_works_again_after_resume() {
    let s = setup();

    s.client.pause_campaign(&s.campaign_id);
    s.client.resume_campaign(&s.campaign_id);

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
fn campaign_can_be_closed_from_active() {
    let s = setup();

    s.client.close_campaign(&s.campaign_id);

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Closed
    );
}

#[test]
fn campaign_can_be_closed_while_paused() {
    let s = setup();

    s.client.pause_campaign(&s.campaign_id);
    s.client.close_campaign(&s.campaign_id);

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Closed
    );
}

#[test]
fn closed_campaign_rejects_new_allocations() {
    let s = setup();

    s.client.close_campaign(&s.campaign_id);

    let result = s.client.try_verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    assert_eq!(result, Err(Ok(CampaignError::CampaignNotActive)));
}

#[test]
fn closed_campaign_cannot_be_resumed() {
    let s = setup();

    s.client.close_campaign(&s.campaign_id);

    let result = s.client.try_resume_campaign(&s.campaign_id);

    assert_eq!(result, Err(Ok(CampaignError::CampaignNotPaused)));
}

#[test]
fn already_closed_campaign_cannot_be_closed_again() {
    let s = setup();

    s.client.close_campaign(&s.campaign_id);

    let result = s.client.try_close_campaign(&s.campaign_id);

    assert_eq!(result, Err(Ok(CampaignError::CampaignAlreadyClosed)));
}

#[test]
fn allocated_reward_can_be_claimed_while_paused() {
    let s = setup();

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    s.client.pause_campaign(&s.campaign_id);

    let claimed = s.client.claim_reward(&s.campaign_id, &s.agent);

    assert_eq!(claimed, 2);

    let token = TokenClient::new(&s.env, &s.token_address);

    assert_eq!(token.balance(&s.agent), 2);
    assert_eq!(token.balance(&s.contract_id), 98);
}

#[test]
fn allocated_reward_can_be_claimed_after_close() {
    let s = setup();

    s.client.verify_and_allocate_reward(
        &s.campaign_id,
        &s.verifier,
        &s.agent,
        &String::from_str(&s.env, "merchant-001"),
        &String::from_str(&s.env, "KYC_VERIFIED"),
    );

    s.client.close_campaign(&s.campaign_id);

    let claimed = s.client.claim_reward(&s.campaign_id, &s.agent);

    assert_eq!(claimed, 2);

    assert_eq!(s.client.claimable_reward(&s.campaign_id, &s.agent), 0);

    let campaign = s.client.get_campaign(&s.campaign_id);

    assert_eq!(campaign.allocated_amount, 2);
    assert_eq!(campaign.claimed_amount, 2);
}

#[test]
fn campaign_admin_must_authorize_pause() {
    let s = setup();

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Active
    );

    // No campaign-admin authorisation is provided.
    s.env.set_auths(&[]);

    let result = s.client.try_pause_campaign(&s.campaign_id);

    assert!(result.is_err());

    // Failed authentication must not change campaign state.
    s.env.mock_all_auths();

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Active
    );

    // A properly authorised pause must still work afterwards.
    s.client.pause_campaign(&s.campaign_id);

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Paused
    );
}

#[test]
fn campaign_admin_must_authorize_resume() {
    let s = setup();

    s.client.pause_campaign(&s.campaign_id);

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Paused
    );

    // No campaign-admin authorisation is provided.
    s.env.set_auths(&[]);

    let result = s.client.try_resume_campaign(&s.campaign_id);

    assert!(result.is_err());

    // Failed authentication must not change campaign state.
    s.env.mock_all_auths();

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Paused
    );

    // A properly authorised resume must still work afterwards.
    s.client.resume_campaign(&s.campaign_id);

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Active
    );
}

#[test]
fn campaign_admin_must_authorize_close() {
    let s = setup();

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Active
    );

    // No campaign-admin authorisation is provided.
    s.env.set_auths(&[]);

    let result = s.client.try_close_campaign(&s.campaign_id);

    assert!(result.is_err());

    // Failed authentication must not change campaign state.
    s.env.mock_all_auths();

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Active
    );

    // A properly authorised close must still work afterwards.
    s.client.close_campaign(&s.campaign_id);

    assert_eq!(
        s.client.get_campaign(&s.campaign_id).status,
        CampaignStatus::Closed
    );
}
