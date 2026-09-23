use axionvera_campaign_contract::{CampaignContract, CampaignContractClient};

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    Address, Env, String,
};

#[test]
fn campaign_accounting_and_token_balance_invariants_hold_across_full_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(100);

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let verifier = Address::generate(&env);
    let agent_a = Address::generate(&env);
    let agent_b = Address::generate(&env);

    client.initialize(&protocol_admin);

    let issuer = Address::generate(&env);
    let asset = env.register_stellar_asset_contract_v2(issuer);
    let token_address = asset.address();

    let token_admin = StellarAssetClient::new(&env, &token_address);
    let token = TokenClient::new(&env, &token_address);

    token_admin.mint(&campaign_admin, &1_000);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &token_address,
        &String::from_str(&env, "Invariant Test Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.fund_campaign(&campaign_id, &100);
    client.add_verifier(&campaign_id, &verifier);

    let kyc = String::from_str(&env, "KYC_VERIFIED");
    let first_tx = String::from_str(&env, "FIRST_TRANSACTION");

    client.add_activation_rule(&campaign_id, &kyc, &2);
    client.add_activation_rule(&campaign_id, &first_tx, &3);

    // Initial funded state.
    let campaign = client.get_campaign(&campaign_id);

    assert_eq!(campaign.funded_amount, 100);
    assert_eq!(campaign.allocated_amount, 0);
    assert_eq!(campaign.claimed_amount, 0);
    assert_eq!(campaign.withdrawn_amount, 0);
    assert_eq!(client.available_unused_funds(&campaign_id), 100);
    assert_eq!(token.balance(&contract_id), 100);

    // Allocate 5 to agent A and 2 to agent B.
    client.verify_and_allocate_reward(
        &campaign_id,
        &verifier,
        &agent_a,
        &String::from_str(&env, "merchant-a"),
        &kyc,
    );

    client.verify_and_allocate_reward(
        &campaign_id,
        &verifier,
        &agent_a,
        &String::from_str(&env, "merchant-a"),
        &first_tx,
    );

    client.verify_and_allocate_reward(
        &campaign_id,
        &verifier,
        &agent_b,
        &String::from_str(&env, "merchant-b"),
        &kyc,
    );

    let campaign = client.get_campaign(&campaign_id);

    assert_eq!(campaign.allocated_amount, 7);
    assert_eq!(campaign.claimed_amount, 0);
    assert_eq!(campaign.withdrawn_amount, 0);

    assert_eq!(client.claimable_reward(&campaign_id, &agent_a), 5);
    assert_eq!(client.claimable_reward(&campaign_id, &agent_b), 2);

    assert_eq!(client.available_unused_funds(&campaign_id), 93);
    assert_eq!(token.balance(&contract_id), 100);

    assert_eq!(
        campaign.funded_amount,
        campaign.allocated_amount
            + client.available_unused_funds(&campaign_id)
            + campaign.withdrawn_amount
    );

    assert_eq!(
        token.balance(&contract_id),
        campaign.funded_amount - campaign.claimed_amount - campaign.withdrawn_amount
    );

    // Agent A claims 5.
    assert_eq!(client.claim_reward(&campaign_id, &agent_a), 5);

    let campaign = client.get_campaign(&campaign_id);

    assert_eq!(campaign.allocated_amount, 7);
    assert_eq!(campaign.claimed_amount, 5);
    assert_eq!(campaign.withdrawn_amount, 0);
    assert_eq!(client.available_unused_funds(&campaign_id), 93);
    assert_eq!(token.balance(&contract_id), 95);

    assert_eq!(
        token.balance(&contract_id),
        campaign.funded_amount - campaign.claimed_amount - campaign.withdrawn_amount
    );

    // Close the campaign and withdraw every genuinely unused token.
    client.close_campaign(&campaign_id);

    assert_eq!(client.withdraw_unused_funds(&campaign_id, &93), 0);

    let campaign = client.get_campaign(&campaign_id);

    assert_eq!(campaign.funded_amount, 100);
    assert_eq!(campaign.allocated_amount, 7);
    assert_eq!(campaign.claimed_amount, 5);
    assert_eq!(campaign.withdrawn_amount, 93);
    assert_eq!(client.available_unused_funds(&campaign_id), 0);

    // Only agent B's still-unclaimed allocated reward may remain.
    assert_eq!(token.balance(&contract_id), 2);
    assert_eq!(
        token.balance(&contract_id),
        campaign.allocated_amount - campaign.claimed_amount
    );

    // Agent B can still claim that reserved balance.
    assert_eq!(client.claim_reward(&campaign_id, &agent_b), 2);

    let campaign = client.get_campaign(&campaign_id);

    assert_eq!(campaign.funded_amount, 100);
    assert_eq!(campaign.allocated_amount, 7);
    assert_eq!(campaign.claimed_amount, 7);
    assert_eq!(campaign.withdrawn_amount, 93);

    assert_eq!(client.claimable_reward(&campaign_id, &agent_a), 0);
    assert_eq!(client.claimable_reward(&campaign_id, &agent_b), 0);
    assert_eq!(token.balance(&contract_id), 0);

    // Final conservation invariant.
    assert_eq!(
        campaign.funded_amount,
        campaign.claimed_amount + campaign.withdrawn_amount
    );
}
