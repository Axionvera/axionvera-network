use axionvera_campaign_contract::{CampaignContract, CampaignContractClient, CampaignError};
use soroban_sdk::{
    testutils::Address as _,
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
