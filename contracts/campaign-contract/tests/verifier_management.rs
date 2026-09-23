use axionvera_campaign_contract::{CampaignContract, CampaignContractClient, CampaignError};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup_campaign() -> (Env, CampaignContractClient<'static>, u64, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Ibadan Merchant Activation"),
        &100,
        &1_000,
        &0,
    );

    let client: CampaignContractClient<'static> = unsafe { core::mem::transmute(client) };

    (env, client, campaign_id, campaign_admin)
}

#[test]
fn campaign_admin_can_add_verifier() {
    let (env, client, campaign_id, _) = setup_campaign();

    let verifier = Address::generate(&env);

    assert!(!client.is_verifier(&campaign_id, &verifier,));

    client.add_verifier(&campaign_id, &verifier);

    assert!(client.is_verifier(&campaign_id, &verifier,));
}

#[test]
fn campaign_admin_can_remove_verifier() {
    let (env, client, campaign_id, _) = setup_campaign();

    let verifier = Address::generate(&env);

    client.add_verifier(&campaign_id, &verifier);

    assert!(client.is_verifier(&campaign_id, &verifier,));

    client.remove_verifier(&campaign_id, &verifier);

    assert!(!client.is_verifier(&campaign_id, &verifier,));
}

#[test]
fn rejects_duplicate_verifier() {
    let (env, client, campaign_id, _) = setup_campaign();

    let verifier = Address::generate(&env);

    client.add_verifier(&campaign_id, &verifier);

    let result = client.try_add_verifier(&campaign_id, &verifier);

    assert_eq!(result, Err(Ok(CampaignError::VerifierAlreadyExists)));
}

#[test]
fn rejects_removing_missing_verifier() {
    let (env, client, campaign_id, _) = setup_campaign();

    let verifier = Address::generate(&env);

    let result = client.try_remove_verifier(&campaign_id, &verifier);

    assert_eq!(result, Err(Ok(CampaignError::VerifierNotFound)));
}

#[test]
fn verifier_permissions_are_scoped_per_campaign() {
    let (env, client, campaign_id_1, campaign_admin) = setup_campaign();

    let reward_token = Address::generate(&env);
    let verifier = Address::generate(&env);

    let campaign_id_2 = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Lagos Merchant Activation"),
        &100,
        &1_000,
        &0,
    );

    client.add_verifier(&campaign_id_1, &verifier);

    assert!(client.is_verifier(&campaign_id_1, &verifier,));

    assert!(!client.is_verifier(&campaign_id_2, &verifier,));
}

#[test]
fn missing_campaign_returns_error() {
    let (env, client, _, _) = setup_campaign();

    let verifier = Address::generate(&env);

    let result = client.try_is_verifier(&999, &verifier);

    assert_eq!(result, Err(Ok(CampaignError::CampaignNotFound)));
}
