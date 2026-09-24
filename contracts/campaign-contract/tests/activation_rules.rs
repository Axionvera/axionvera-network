use axionvera_campaign_contract::{CampaignContract, CampaignContractClient, CampaignError};
use soroban_sdk::testutils::Ledger;
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
fn adds_and_reads_activation_rule() {
    let (env, client, campaign_id, _) = setup_campaign();

    let milestone = String::from_str(&env, "KYC_VERIFIED");

    client.add_activation_rule(&campaign_id, &milestone, &2);

    let rule = client.get_activation_rule(&campaign_id, &milestone);

    assert_eq!(rule.campaign_id, campaign_id);
    assert_eq!(rule.milestone, milestone);
    assert_eq!(rule.reward_amount, 2);
    assert!(rule.enabled);
}

#[test]
fn supports_multiple_rules_in_same_campaign() {
    let (env, client, campaign_id, _) = setup_campaign();

    let kyc = String::from_str(&env, "KYC_VERIFIED");
    let first_tx = String::from_str(&env, "FIRST_TRANSACTION");

    client.add_activation_rule(&campaign_id, &kyc, &2);

    client.add_activation_rule(&campaign_id, &first_tx, &3);

    assert_eq!(
        client.get_activation_rule(&campaign_id, &kyc).reward_amount,
        2
    );

    assert_eq!(
        client
            .get_activation_rule(&campaign_id, &first_tx)
            .reward_amount,
        3
    );
}

#[test]
fn rejects_duplicate_rule_in_same_campaign() {
    let (env, client, campaign_id, _) = setup_campaign();

    let milestone = String::from_str(&env, "KYC_VERIFIED");

    client.add_activation_rule(&campaign_id, &milestone, &2);

    let result = client.try_add_activation_rule(&campaign_id, &milestone, &5);

    assert_eq!(result, Err(Ok(CampaignError::RuleAlreadyExists)));
}

#[test]
fn rejects_zero_or_negative_reward_amount() {
    let (env, client, campaign_id, _) = setup_campaign();

    let zero = String::from_str(&env, "ZERO_REWARD");

    assert_eq!(
        client.try_add_activation_rule(&campaign_id, &zero, &0,),
        Err(Ok(CampaignError::InvalidRewardAmount))
    );

    let negative = String::from_str(&env, "NEGATIVE_REWARD");

    assert_eq!(
        client.try_add_activation_rule(&campaign_id, &negative, &-1,),
        Err(Ok(CampaignError::InvalidRewardAmount))
    );
}

#[test]
fn same_milestone_can_exist_in_different_campaigns() {
    let (env, client, campaign_id_1, campaign_admin) = setup_campaign();

    let reward_token = Address::generate(&env);

    let campaign_id_2 = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Lagos Merchant Activation"),
        &100,
        &1_000,
        &0,
    );

    let milestone = String::from_str(&env, "KYC_VERIFIED");

    client.add_activation_rule(&campaign_id_1, &milestone, &2);

    client.add_activation_rule(&campaign_id_2, &milestone, &5);

    assert_eq!(
        client
            .get_activation_rule(&campaign_id_1, &milestone)
            .reward_amount,
        2
    );

    assert_eq!(
        client
            .get_activation_rule(&campaign_id_2, &milestone)
            .reward_amount,
        5
    );
}

#[test]
fn returns_rule_not_found_for_missing_rule() {
    let (env, client, campaign_id, _) = setup_campaign();

    let result =
        client.try_get_activation_rule(&campaign_id, &String::from_str(&env, "DOES_NOT_EXIST"));

    assert_eq!(result, Err(Ok(CampaignError::RuleNotFound)));
}

#[test]
fn campaign_admin_must_authorize_adding_activation_rule() {
    let (env, client, campaign_id, _) = setup_campaign();

    let milestone = String::from_str(&env, "AUTH_TEST");

    // No campaign-admin authorisation is provided.
    env.set_auths(&[]);

    let result = client.try_add_activation_rule(&campaign_id, &milestone, &5);

    assert!(result.is_err());

    // Failed authentication must not create the rule.
    env.mock_all_auths();

    assert_eq!(
        client.try_get_activation_rule(&campaign_id, &milestone),
        Err(Ok(CampaignError::RuleNotFound))
    );

    // A properly authorised call must still work afterwards.
    client.add_activation_rule(&campaign_id, &milestone, &5);

    let rule = client.get_activation_rule(&campaign_id, &milestone);

    assert_eq!(rule.reward_amount, 5);
    assert!(rule.enabled);
}

#[test]
fn ended_campaign_cannot_add_activation_rule() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    env.ledger().set_timestamp(100);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Ended Rule Campaign"),
        &100,
        &200,
        &0,
    );

    env.ledger().set_timestamp(200);

    let milestone = String::from_str(&env, "POST_END_RULE");

    let result = client.try_add_activation_rule(&campaign_id, &milestone, &2);

    assert_eq!(
        result,
        Err(Ok(
            axionvera_campaign_contract::CampaignError::CampaignEnded
        ))
    );

    assert_eq!(
        client.try_get_activation_rule(&campaign_id, &milestone),
        Err(Ok(axionvera_campaign_contract::CampaignError::RuleNotFound))
    );
}

#[test]
fn activation_rule_can_be_added_before_start_time() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CampaignContract, ());
    let client = CampaignContractClient::new(&env, &contract_id);

    let protocol_admin = Address::generate(&env);
    let campaign_admin = Address::generate(&env);
    let reward_token = Address::generate(&env);

    client.initialize(&protocol_admin);

    env.ledger().set_timestamp(100);

    let campaign_id = client.create_campaign(
        &campaign_admin,
        &reward_token,
        &String::from_str(&env, "Preconfigured Campaign"),
        &200,
        &1_000,
        &0,
    );

    let milestone = String::from_str(&env, "PRE_START_RULE");

    client.add_activation_rule(&campaign_id, &milestone, &2);

    let rule = client.get_activation_rule(&campaign_id, &milestone);

    assert_eq!(rule.reward_amount, 2);
    assert!(rule.enabled);
}

#[test]
fn paused_campaign_cannot_add_activation_rule() {
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
        &String::from_str(&env, "Paused Rule Campaign"),
        &100,
        &1_000,
        &0,
    );

    client.pause_campaign(&campaign_id);

    let milestone = String::from_str(&env, "PAUSED_RULE");

    let result = client.try_add_activation_rule(&campaign_id, &milestone, &2);

    assert_eq!(
        result,
        Err(Ok(
            axionvera_campaign_contract::CampaignError::CampaignNotActive
        ))
    );

    assert_eq!(
        client.try_get_activation_rule(&campaign_id, &milestone),
        Err(Ok(axionvera_campaign_contract::CampaignError::RuleNotFound))
    );
}
