use axionvera_campaign_contract::{CampaignContract, CampaignContractClient, CampaignError};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, Env, String,
};

#[test]
fn funding_campaign_moves_real_tokens_into_contract() {
    let env = Env::default();
    env.mock_all_auths();

    // Deploy campaign contract.
    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);

    client.initialize(&protocol_admin);

    // Deploy a Stellar test asset.
    let issuer = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(issuer);

    let token_address = stellar_asset.address();

    let token_admin = StellarAssetClient::new(&env, &token_address);
    let token = TokenClient::new(&env, &token_address);

    // Give the campaign admin 10,000 test tokens.
    token_admin.mint(&campaign_admin, &10_000);

    assert_eq!(token.balance(&campaign_admin), 10_000);
    assert_eq!(token.balance(&contract_id), 0);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &token_address,
        &String::from_str(&env, "Ibadan Merchant Activation"),
        &100,
        &1_000,
        &0,
    );

    let funded = client.fund_campaign(&campaign_id, &2_500);

    assert_eq!(funded, 2_500);

    // Prove the tokens actually moved.
    assert_eq!(token.balance(&campaign_admin), 7_500);
    assert_eq!(token.balance(&contract_id), 2_500);

    // Prove campaign accounting matches the token movement.
    let campaign = client.get_campaign(&campaign_id);

    assert_eq!(campaign.funded_amount, 2_500);
    assert_eq!(campaign.allocated_amount, 0);
    assert_eq!(campaign.claimed_amount, 0);
}

#[test]
fn multiple_funding_transactions_accumulate() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);

    client.initialize(&protocol_admin);

    let issuer = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(issuer);

    let token_address = stellar_asset.address();

    let token_admin = StellarAssetClient::new(&env, &token_address);
    let token = TokenClient::new(&env, &token_address);

    token_admin.mint(&campaign_admin, &10_000);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &token_address,
        &String::from_str(&env, "Merchant Growth"),
        &100,
        &1_000,
        &0,
    );

    assert_eq!(client.fund_campaign(&campaign_id, &1_000), 1_000);
    assert_eq!(client.fund_campaign(&campaign_id, &500), 1_500);

    let campaign = client.get_campaign(&campaign_id);

    assert_eq!(campaign.funded_amount, 1_500);
    assert_eq!(token.balance(&contract_id), 1_500);
    assert_eq!(token.balance(&campaign_admin), 8_500);
}

#[test]
fn rejects_zero_or_negative_funding() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);

    client.initialize(&protocol_admin);

    let issuer = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(issuer);

    let token_address = stellar_asset.address();

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &token_address,
        &String::from_str(&env, "Invalid Funding Test"),
        &100,
        &1_000,
        &0,
    );

    assert_eq!(
        client.try_fund_campaign(&campaign_id, &0),
        Err(Ok(CampaignError::InvalidAmount))
    );

    assert_eq!(
        client.try_fund_campaign(&campaign_id, &-1),
        Err(Ok(CampaignError::InvalidAmount))
    );

    assert_eq!(client.get_campaign(&campaign_id).funded_amount, 0);
}

#[test]
fn campaign_admin_must_authorize_campaign_funding() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);

    client.initialize(&protocol_admin);

    let issuer = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(issuer);
    let token_address = stellar_asset.address();

    let token_admin = StellarAssetClient::new(&env, &token_address);
    let token = TokenClient::new(&env, &token_address);

    token_admin.mint(&campaign_admin, &1_000);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &token_address,
        &String::from_str(&env, "Funding Auth Test"),
        &100,
        &1_000,
        &0,
    );

    assert_eq!(token.balance(&campaign_admin), 1_000);
    assert_eq!(token.balance(&contract_id), 0);

    // No campaign-admin authorisation is provided.
    env.set_auths(&[]);

    let result = client.try_fund_campaign(&campaign_id, &100);

    assert!(result.is_err());

    // Failed authentication must not move tokens or alter accounting.
    env.mock_all_auths();

    assert_eq!(token.balance(&campaign_admin), 1_000);
    assert_eq!(token.balance(&contract_id), 0);
    assert_eq!(client.get_campaign(&campaign_id).funded_amount, 0);

    // Properly authorised funding must still work afterwards.
    let funded = client.fund_campaign(&campaign_id, &100);

    assert_eq!(funded, 100);
    assert_eq!(token.balance(&campaign_admin), 900);
    assert_eq!(token.balance(&contract_id), 100);
    assert_eq!(client.get_campaign(&campaign_id).funded_amount, 100);
}

#[test]
fn ended_campaign_cannot_be_funded() {
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

    soroban_sdk::token::StellarAssetClient::new(&env, &reward_token).mint(&campaign_admin, &1_000);

    env.ledger().set_timestamp(100);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Ended Funding Campaign"),
        &100,
        &200,
        &0,
    );

    // Exact end boundary is no longer part of the active time window.
    env.ledger().set_timestamp(200);

    let result = client.try_fund_campaign(&campaign_id, &100);

    assert_eq!(
        result,
        Err(Ok(
            axionvera_campaign_contract::CampaignError::CampaignEnded
        ))
    );

    assert_eq!(client.get_campaign(&campaign_id).funded_amount, 0);
}

#[test]
fn campaign_can_be_funded_before_start_time() {
    use soroban_sdk::testutils::Ledger;

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

    soroban_sdk::token::StellarAssetClient::new(&env, &reward_token).mint(&campaign_admin, &1_000);

    env.ledger().set_timestamp(100);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Pre-funded Campaign"),
        &200,
        &1_000,
        &0,
    );

    // Treasury preparation is allowed before the campaign starts.
    assert_eq!(client.fund_campaign(&campaign_id, &100), 100);

    assert_eq!(client.get_campaign(&campaign_id).funded_amount, 100);
}

#[test]
fn paused_campaign_cannot_be_funded() {
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

    soroban_sdk::token::StellarAssetClient::new(&env, &reward_token).mint(&campaign_admin, &1_000);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Paused Funding Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.pause_campaign(&campaign_id);

    let result = client.try_fund_campaign(&campaign_id, &100);

    assert_eq!(
        result,
        Err(Ok(
            axionvera_campaign_contract::CampaignError::CampaignNotActive
        ))
    );

    assert_eq!(client.get_campaign(&campaign_id).funded_amount, 0);
}
