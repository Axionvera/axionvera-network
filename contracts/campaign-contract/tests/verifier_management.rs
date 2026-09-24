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

#[test]
fn campaign_admin_must_authorize_adding_verifier() {
    let (env, client, campaign_id, _) = setup_campaign();

    let verifier = Address::generate(&env);

    assert!(!client.is_verifier(&campaign_id, &verifier));

    // No campaign-admin authorisation is provided.
    env.set_auths(&[]);

    let result = client.try_add_verifier(&campaign_id, &verifier);

    assert!(result.is_err());

    // Failed authentication must not create verifier permission.
    env.mock_all_auths();

    assert!(!client.is_verifier(&campaign_id, &verifier));

    // A properly authorised call must still work afterwards.
    client.add_verifier(&campaign_id, &verifier);

    assert!(client.is_verifier(&campaign_id, &verifier));
}

#[test]
fn campaign_admin_must_authorize_removing_verifier() {
    let (env, client, campaign_id, _) = setup_campaign();

    let verifier = Address::generate(&env);

    client.add_verifier(&campaign_id, &verifier);
    assert!(client.is_verifier(&campaign_id, &verifier));

    // No campaign-admin authorisation is provided.
    env.set_auths(&[]);

    let result = client.try_remove_verifier(&campaign_id, &verifier);

    assert!(result.is_err());

    // Failed authentication must not revoke verifier permission.
    env.mock_all_auths();

    assert!(client.is_verifier(&campaign_id, &verifier));

    // A properly authorised removal must still work afterwards.
    client.remove_verifier(&campaign_id, &verifier);

    assert!(!client.is_verifier(&campaign_id, &verifier));
}

#[test]
fn closed_campaign_cannot_add_verifier() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);
    let verifier = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Closed Verifier Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.close_campaign(&campaign_id);

    let result = client.try_add_verifier(&campaign_id, &verifier);

    assert_eq!(
        result,
        Err(Ok(
            axionvera_campaign_contract::CampaignError::CampaignNotActive
        ))
    );

    assert!(!client.is_verifier(&campaign_id, &verifier));
}

#[test]
fn closed_campaign_cannot_remove_verifier() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);
    let verifier = Address::generate(&env);

    client.initialize(&protocol_admin);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Closed Verifier Removal Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.add_verifier(&campaign_id, &verifier);
    client.close_campaign(&campaign_id);

    let result = client.try_remove_verifier(&campaign_id, &verifier);

    assert_eq!(
        result,
        Err(Ok(
            axionvera_campaign_contract::CampaignError::CampaignNotActive
        ))
    );

    assert!(client.is_verifier(&campaign_id, &verifier));
}

#[test]
fn ended_campaign_cannot_add_verifier() {
    use soroban_sdk::testutils::Ledger;

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);
    let verifier = Address::generate(&env);

    client.initialize(&protocol_admin);

    env.ledger().set_timestamp(100);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Ended Verifier Campaign"),
        &100,
        &200,
        &0,
    );

    env.ledger().set_timestamp(200);

    let result = client.try_add_verifier(&campaign_id, &verifier);

    assert_eq!(
        result,
        Err(Ok(
            axionvera_campaign_contract::CampaignError::CampaignEnded
        ))
    );

    assert!(!client.is_verifier(&campaign_id, &verifier));
}

#[test]
fn ended_campaign_cannot_remove_verifier() {
    use soroban_sdk::testutils::Ledger;

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);
    let verifier = Address::generate(&env);

    client.initialize(&protocol_admin);

    env.ledger().set_timestamp(100);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Ended Verifier Removal Campaign"),
        &100,
        &200,
        &0,
    );

    client.add_verifier(&campaign_id, &verifier);

    env.ledger().set_timestamp(200);

    let result = client.try_remove_verifier(&campaign_id, &verifier);

    assert_eq!(
        result,
        Err(Ok(
            axionvera_campaign_contract::CampaignError::CampaignEnded
        ))
    );

    assert!(client.is_verifier(&campaign_id, &verifier));
}

#[test]
fn paused_campaign_can_manage_verifiers_before_end() {
    use soroban_sdk::testutils::Ledger;

    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);
    let verifier_a = Address::generate(&env);
    let verifier_b = Address::generate(&env);

    client.initialize(&protocol_admin);

    env.ledger().set_timestamp(100);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Paused Verifier Campaign"),
        &100,
        &200,
        &0,
    );

    client.add_verifier(&campaign_id, &verifier_a);
    client.pause_campaign(&campaign_id);

    env.ledger().set_timestamp(150);

    client.remove_verifier(&campaign_id, &verifier_a);
    client.add_verifier(&campaign_id, &verifier_b);

    assert!(!client.is_verifier(&campaign_id, &verifier_a));
    assert!(client.is_verifier(&campaign_id, &verifier_b));
}
