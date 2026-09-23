use axionvera_campaign_contract::{
    CampaignContract, CampaignContractClient, CampaignError, CampaignStatus,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup() -> (Env, CampaignContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    // Extend lifetimes for this test helper only.
    let client: CampaignContractClient<'static> = unsafe { core::mem::transmute(client) };

    (env, client, protocol_admin, reward_token)
}

#[test]
fn initializes_protocol() {
    let (env, client, protocol_admin, _) = setup();

    assert!(client.is_initialized());
    assert_eq!(client.protocol_admin(), protocol_admin);
    assert_eq!(client.next_campaign_id(), 1);

    drop(env);
}

#[test]
fn creates_campaign_and_increments_id() {
    let (env, client, _, reward_token) = setup();

    let campaign_admin = Address::generate(&env);

    let first_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Ibadan Merchant Activation"),
        &100,
        &1_000,
        &0,
    );

    assert_eq!(first_id, 1);
    assert_eq!(client.next_campaign_id(), 2);

    let campaign = client.get_campaign(&first_id);

    assert_eq!(campaign.id, 1);
    assert_eq!(campaign.admin, campaign_admin);
    assert_eq!(campaign.reward_token, reward_token);
    assert_eq!(campaign.start_time, 100);
    assert_eq!(campaign.end_time, 1_000);
    assert_eq!(campaign.status, CampaignStatus::Active);
    assert_eq!(campaign.funded_amount, 0);
    assert_eq!(campaign.allocated_amount, 0);
    assert_eq!(campaign.claimed_amount, 0);
    assert_eq!(campaign.per_agent_cap, 0);

    drop(env);
}

#[test]
fn supports_multiple_campaigns() {
    let (env, client, _, reward_token) = setup();

    let admin = Address::generate(&env);

    let id_1 = client.create_campaign(
        &admin,
        &reward_token,
        &String::from_str(&env, "Campaign One"),
        &100,
        &200,
        &0,
    );

    let id_2 = client.create_campaign(
        &admin,
        &reward_token,
        &String::from_str(&env, "Campaign Two"),
        &300,
        &400,
        &100,
    );

    assert_eq!(id_1, 1);
    assert_eq!(id_2, 2);
    assert_eq!(client.next_campaign_id(), 3);

    drop(env);
}

#[test]
fn rejects_invalid_time_range() {
    let (env, client, _, reward_token) = setup();

    let admin = Address::generate(&env);

    let result = client.try_create_campaign(
        &admin,
        &reward_token,
        &String::from_str(&env, "Invalid Campaign"),
        &500,
        &500,
        &0,
    );

    assert_eq!(result, Err(Ok(CampaignError::InvalidTimeRange)));

    drop(env);
}

#[test]
fn rejects_negative_agent_cap() {
    let (env, client, _, reward_token) = setup();

    let admin = Address::generate(&env);

    let result = client.try_create_campaign(
        &admin,
        &reward_token,
        &String::from_str(&env, "Invalid Cap"),
        &100,
        &200,
        &-1,
    );

    assert_eq!(result, Err(Ok(CampaignError::InvalidCap)));

    drop(env);
}
