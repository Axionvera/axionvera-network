#![no_std]

#[cfg(test)]
extern crate std;

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, token, Address, Env, String,
    Symbol,
};

const TOPIC_CAMPAIGN: Symbol = symbol_short!("campaign");
const TOPIC_INIT: Symbol = symbol_short!("init");
const TOPIC_CREATED: Symbol = symbol_short!("created");
const TOPIC_FUNDED: Symbol = symbol_short!("funded");
const TOPIC_RULE: Symbol = symbol_short!("rule");
const TOPIC_ADDED: Symbol = symbol_short!("added");
const TOPIC_VERIFIER: Symbol = symbol_short!("verifyr");
const TOPIC_REMOVED: Symbol = symbol_short!("removed");
const TOPIC_ACTIVATION: Symbol = symbol_short!("activate");
const TOPIC_REWARD: Symbol = symbol_short!("reward");
const TOPIC_CLAIM: Symbol = symbol_short!("claim");
const TOPIC_PAUSED: Symbol = symbol_short!("paused");
const TOPIC_RESUMED: Symbol = symbol_short!("resumed");
const TOPIC_CLOSED: Symbol = symbol_short!("closed");
const TOPIC_UNUSED: Symbol = symbol_short!("unused");
const TOPIC_WITHDRAW: Symbol = symbol_short!("withdraw");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CampaignError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidTimeRange = 3,
    InvalidCap = 4,
    CampaignNotFound = 5,
    CampaignIdOverflow = 6,
    InvalidAmount = 7,
    CampaignNotActive = 8,
    ArithmeticOverflow = 9,
    InvalidRewardAmount = 10,
    RuleAlreadyExists = 11,
    RuleNotFound = 12,
    VerifierAlreadyExists = 13,
    VerifierNotFound = 14,
    UnauthorizedVerifier = 15,
    RuleDisabled = 16,
    DuplicateActivation = 17,
    InsufficientCampaignFunds = 18,
    AgentCapExceeded = 19,
    NothingToClaim = 20,
    InvalidAccountingState = 21,
    CampaignNotPaused = 22,
    CampaignAlreadyClosed = 23,
    CampaignNotClosed = 24,
    InsufficientUnusedFunds = 25,
    CampaignNotStarted = 26,
    CampaignEnded = 27,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CampaignStatus {
    Active,
    Paused,
    Closed,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Campaign {
    pub id: u64,
    pub admin: Address,
    pub reward_token: Address,
    pub name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub status: CampaignStatus,

    // Treasury accounting.
    pub funded_amount: i128,
    pub allocated_amount: i128,
    pub claimed_amount: i128,
    pub withdrawn_amount: i128,

    // 0 means no per-agent cap.
    pub per_agent_cap: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivationRule {
    pub campaign_id: u64,
    pub milestone: String,
    pub reward_amount: i128,
    pub enabled: bool,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Initialized,
    ProtocolAdmin,
    NextCampaignId,
    Campaign(u64),
    ActivationRule(u64, String),
    Verifier(u64, Address),
    ActivationProcessed(u64, String, String),
    ClaimableReward(u64, Address),
    AgentTotalEarned(u64, Address),
}

#[contract]
pub struct CampaignContract;

#[contractimpl]
impl CampaignContract {
    /// Initializes the Axionvera campaign protocol.
    pub fn initialize(env: Env, protocol_admin: Address) -> Result<(), CampaignError> {
        protocol_admin.require_auth();

        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(CampaignError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage()
            .instance()
            .set(&DataKey::ProtocolAdmin, &protocol_admin);

        // Campaign IDs begin at 1.
        env.storage()
            .instance()
            .set(&DataKey::NextCampaignId, &1_u64);

        env.events()
            .publish((TOPIC_CAMPAIGN, TOPIC_INIT), protocol_admin.clone());

        Self::extend_instance_ttl(&env);

        Ok(())
    }

    /// Creates a new reward campaign.
    ///
    /// Any authenticated address may create a campaign and becomes that
    /// campaign's admin.
    ///
    /// `per_agent_cap == 0` means that the campaign has no per-agent cap.
    pub fn create_campaign(
        env: Env,
        admin: Address,
        reward_token: Address,
        name: String,
        start_time: u64,
        end_time: u64,
        per_agent_cap: i128,
    ) -> Result<u64, CampaignError> {
        Self::require_initialized(&env)?;
        admin.require_auth();

        if end_time <= start_time {
            return Err(CampaignError::InvalidTimeRange);
        }

        if per_agent_cap < 0 {
            return Err(CampaignError::InvalidCap);
        }

        let campaign_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::NextCampaignId)
            .ok_or(CampaignError::NotInitialized)?;

        let next_campaign_id = campaign_id
            .checked_add(1)
            .ok_or(CampaignError::CampaignIdOverflow)?;

        let campaign = Campaign {
            id: campaign_id,
            admin: admin.clone(),
            reward_token: reward_token.clone(),
            name,
            start_time,
            end_time,
            status: CampaignStatus::Active,
            funded_amount: 0,
            allocated_amount: 0,
            claimed_amount: 0,
            withdrawn_amount: 0,
            per_agent_cap,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        Self::extend_persistent_ttl(&env, &DataKey::Campaign(campaign_id));

        env.storage()
            .instance()
            .set(&DataKey::NextCampaignId, &next_campaign_id);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_CREATED),
            (campaign_id, admin, reward_token),
        );

        Ok(campaign_id)
    }

    /// Funds a campaign treasury with its configured reward token.
    ///
    /// Only the campaign admin may fund the campaign in v1.
    /// Tokens are transferred from the admin into this contract.
    pub fn fund_campaign(env: Env, campaign_id: u64, amount: i128) -> Result<i128, CampaignError> {
        Self::require_initialized(&env)?;

        if amount <= 0 {
            return Err(CampaignError::InvalidAmount);
        }

        let mut campaign = Self::load_campaign(&env, campaign_id)?;

        if campaign.status != CampaignStatus::Active {
            return Err(CampaignError::CampaignNotActive);
        }

        campaign.admin.require_auth();

        if env.ledger().timestamp() >= campaign.end_time {
            return Err(CampaignError::CampaignEnded);
        }

        let new_funded_amount = campaign
            .funded_amount
            .checked_add(amount)
            .ok_or(CampaignError::ArithmeticOverflow)?;

        let token_client = token::Client::new(&env, &campaign.reward_token);

        token_client.transfer(&campaign.admin, &env.current_contract_address(), &amount);

        campaign.funded_amount = new_funded_amount;

        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_FUNDED),
            (campaign_id, campaign.admin.clone(), amount),
        );

        Ok(new_funded_amount)
    }

    /// Adds a fixed-reward activation rule to a campaign.
    ///
    /// Example:
    /// `KYC_VERIFIED -> 2 USDC`
    ///
    /// Only the campaign admin may add activation rules.
    pub fn add_activation_rule(
        env: Env,
        campaign_id: u64,
        milestone: String,
        reward_amount: i128,
    ) -> Result<(), CampaignError> {
        Self::require_initialized(&env)?;

        if reward_amount <= 0 {
            return Err(CampaignError::InvalidRewardAmount);
        }

        let campaign = Self::load_campaign(&env, campaign_id)?;

        if campaign.status != CampaignStatus::Active {
            return Err(CampaignError::CampaignNotActive);
        }

        campaign.admin.require_auth();

        if env.ledger().timestamp() >= campaign.end_time {
            return Err(CampaignError::CampaignEnded);
        }

        let key = DataKey::ActivationRule(campaign_id, milestone.clone());

        if env.storage().persistent().has(&key) {
            return Err(CampaignError::RuleAlreadyExists);
        }

        let rule = ActivationRule {
            campaign_id,
            milestone: milestone.clone(),
            reward_amount,
            enabled: true,
        };

        env.storage().persistent().set(&key, &rule);

        Self::extend_persistent_ttl(&env, &key);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_RULE, TOPIC_ADDED),
            (campaign_id, milestone, reward_amount),
        );

        Ok(())
    }

    /// Pauses an active campaign.
    ///
    /// New reward allocations stop while paused, but agents may still
    /// claim rewards that were allocated before the pause.
    pub fn pause_campaign(env: Env, campaign_id: u64) -> Result<(), CampaignError> {
        Self::require_initialized(&env)?;

        let mut campaign = Self::load_campaign(&env, campaign_id)?;

        campaign.admin.require_auth();

        if campaign.status != CampaignStatus::Active {
            return Err(CampaignError::CampaignNotActive);
        }

        if env.ledger().timestamp() >= campaign.end_time {
            return Err(CampaignError::CampaignEnded);
        }

        campaign.status = CampaignStatus::Paused;

        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_PAUSED),
            (campaign_id, campaign.admin.clone()),
        );

        Ok(())
    }

    /// Resumes a paused campaign.
    pub fn resume_campaign(env: Env, campaign_id: u64) -> Result<(), CampaignError> {
        Self::require_initialized(&env)?;

        let mut campaign = Self::load_campaign(&env, campaign_id)?;

        campaign.admin.require_auth();

        if campaign.status != CampaignStatus::Paused {
            return Err(CampaignError::CampaignNotPaused);
        }

        if env.ledger().timestamp() >= campaign.end_time {
            return Err(CampaignError::CampaignEnded);
        }

        campaign.status = CampaignStatus::Active;

        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_RESUMED),
            (campaign_id, campaign.admin.clone()),
        );

        Ok(())
    }

    /// Permanently closes a campaign.
    ///
    /// A closed campaign cannot be resumed and cannot receive new
    /// reward allocations. Previously allocated rewards remain claimable.
    pub fn close_campaign(env: Env, campaign_id: u64) -> Result<(), CampaignError> {
        Self::require_initialized(&env)?;

        let mut campaign = Self::load_campaign(&env, campaign_id)?;

        campaign.admin.require_auth();

        if campaign.status == CampaignStatus::Closed {
            return Err(CampaignError::CampaignAlreadyClosed);
        }

        campaign.status = CampaignStatus::Closed;

        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_CLOSED),
            (campaign_id, campaign.admin.clone()),
        );

        Ok(())
    }

    /// Returns campaign funds that have not been allocated to agents.
    pub fn available_unused_funds(env: Env, campaign_id: u64) -> Result<i128, CampaignError> {
        Self::require_initialized(&env)?;

        let campaign = Self::load_campaign(&env, campaign_id)?;

        campaign
            .funded_amount
            .checked_sub(campaign.allocated_amount)
            .and_then(|value| value.checked_sub(campaign.withdrawn_amount))
            .ok_or(CampaignError::InvalidAccountingState)
    }

    /// Withdraws unused campaign funds after the campaign has been closed.
    ///
    /// Rewards already allocated to agents remain fully reserved and
    /// cannot be withdrawn by the campaign admin.
    pub fn withdraw_unused_funds(
        env: Env,
        campaign_id: u64,
        amount: i128,
    ) -> Result<i128, CampaignError> {
        Self::require_initialized(&env)?;

        if amount <= 0 {
            return Err(CampaignError::InvalidAmount);
        }

        let mut campaign = Self::load_campaign(&env, campaign_id)?;

        campaign.admin.require_auth();

        if campaign.status != CampaignStatus::Closed {
            return Err(CampaignError::CampaignNotClosed);
        }

        let available = campaign
            .funded_amount
            .checked_sub(campaign.allocated_amount)
            .and_then(|value| value.checked_sub(campaign.withdrawn_amount))
            .ok_or(CampaignError::InvalidAccountingState)?;

        if amount > available {
            return Err(CampaignError::InsufficientUnusedFunds);
        }

        let new_withdrawn = campaign
            .withdrawn_amount
            .checked_add(amount)
            .ok_or(CampaignError::ArithmeticOverflow)?;

        campaign.withdrawn_amount = new_withdrawn;

        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        let token_client = token::Client::new(&env, &campaign.reward_token);

        token_client.transfer(&env.current_contract_address(), &campaign.admin, &amount);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_UNUSED, TOPIC_WITHDRAW),
            (campaign_id, campaign.admin.clone(), amount),
        );

        Ok(available - amount)
    }

    /// Adds an authorised verifier to a campaign.
    ///
    /// Only the campaign admin may manage verifier permissions.
    pub fn add_verifier(
        env: Env,
        campaign_id: u64,
        verifier: Address,
    ) -> Result<(), CampaignError> {
        Self::require_initialized(&env)?;

        let campaign = Self::load_campaign(&env, campaign_id)?;

        campaign.admin.require_auth();

        if campaign.status == CampaignStatus::Closed {
            return Err(CampaignError::CampaignNotActive);
        }

        if env.ledger().timestamp() >= campaign.end_time {
            return Err(CampaignError::CampaignEnded);
        }

        let key = DataKey::Verifier(campaign_id, verifier.clone());

        if env.storage().persistent().has(&key) {
            return Err(CampaignError::VerifierAlreadyExists);
        }

        env.storage().persistent().set(&key, &true);

        Self::extend_persistent_ttl(&env, &key);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_VERIFIER, TOPIC_ADDED),
            (campaign_id, verifier),
        );

        Ok(())
    }

    /// Removes an authorised verifier from a campaign.
    pub fn remove_verifier(
        env: Env,
        campaign_id: u64,
        verifier: Address,
    ) -> Result<(), CampaignError> {
        Self::require_initialized(&env)?;

        let campaign = Self::load_campaign(&env, campaign_id)?;

        campaign.admin.require_auth();

        if campaign.status == CampaignStatus::Closed {
            return Err(CampaignError::CampaignNotActive);
        }

        if env.ledger().timestamp() >= campaign.end_time {
            return Err(CampaignError::CampaignEnded);
        }

        let key = DataKey::Verifier(campaign_id, verifier.clone());

        if !env.storage().persistent().has(&key) {
            return Err(CampaignError::VerifierNotFound);
        }

        env.storage().persistent().remove(&key);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_VERIFIER, TOPIC_REMOVED),
            (campaign_id, verifier),
        );

        Ok(())
    }

    /// Returns true when the address is authorised to verify
    /// activations for the specified campaign.
    pub fn is_verifier(
        env: Env,
        campaign_id: u64,
        verifier: Address,
    ) -> Result<bool, CampaignError> {
        Self::require_initialized(&env)?;

        Self::load_campaign(&env, campaign_id)?;

        let key = DataKey::Verifier(campaign_id, verifier);
        let exists = env.storage().persistent().has(&key);

        if exists {
            Self::extend_persistent_ttl(&env, &key);
        }

        Ok(exists)
    }

    /// Verifies a merchant activation milestone and allocates
    /// the configured campaign reward to an agent.
    pub fn verify_and_allocate_reward(
        env: Env,
        campaign_id: u64,
        verifier: Address,
        agent: Address,
        merchant_ref: String,
        milestone: String,
    ) -> Result<i128, CampaignError> {
        Self::require_initialized(&env)?;

        let mut campaign = Self::load_campaign(&env, campaign_id)?;

        if campaign.status != CampaignStatus::Active {
            return Err(CampaignError::CampaignNotActive);
        }

        let now = env.ledger().timestamp();

        if now < campaign.start_time {
            return Err(CampaignError::CampaignNotStarted);
        }

        if now >= campaign.end_time {
            return Err(CampaignError::CampaignEnded);
        }

        let verifier_key = DataKey::Verifier(campaign_id, verifier.clone());

        if !env.storage().persistent().has(&verifier_key) {
            return Err(CampaignError::UnauthorizedVerifier);
        }

        Self::extend_persistent_ttl(&env, &verifier_key);

        verifier.require_auth();

        let rule_key = DataKey::ActivationRule(campaign_id, milestone.clone());

        let rule: ActivationRule = env
            .storage()
            .persistent()
            .get(&rule_key)
            .ok_or(CampaignError::RuleNotFound)?;

        Self::extend_persistent_ttl(&env, &rule_key);

        if !rule.enabled {
            return Err(CampaignError::RuleDisabled);
        }

        let activation_key =
            DataKey::ActivationProcessed(campaign_id, merchant_ref.clone(), milestone.clone());

        if env.storage().persistent().has(&activation_key) {
            return Err(CampaignError::DuplicateActivation);
        }

        let available = campaign
            .funded_amount
            .checked_sub(campaign.allocated_amount)
            .ok_or(CampaignError::ArithmeticOverflow)?;

        if available < rule.reward_amount {
            return Err(CampaignError::InsufficientCampaignFunds);
        }

        let claimable_key = DataKey::ClaimableReward(campaign_id, agent.clone());

        let current_claimable: i128 = env.storage().persistent().get(&claimable_key).unwrap_or(0);

        let earned_key = DataKey::AgentTotalEarned(campaign_id, agent.clone());

        let current_total_earned: i128 = env.storage().persistent().get(&earned_key).unwrap_or(0);

        let new_total_earned = current_total_earned
            .checked_add(rule.reward_amount)
            .ok_or(CampaignError::ArithmeticOverflow)?;

        if campaign.per_agent_cap > 0 && new_total_earned > campaign.per_agent_cap {
            return Err(CampaignError::AgentCapExceeded);
        }

        let new_claimable = current_claimable
            .checked_add(rule.reward_amount)
            .ok_or(CampaignError::ArithmeticOverflow)?;

        let new_allocated = campaign
            .allocated_amount
            .checked_add(rule.reward_amount)
            .ok_or(CampaignError::ArithmeticOverflow)?;

        campaign.allocated_amount = new_allocated;

        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        env.storage()
            .persistent()
            .set(&claimable_key, &new_claimable);

        Self::extend_persistent_ttl(&env, &claimable_key);

        env.storage()
            .persistent()
            .set(&earned_key, &new_total_earned);

        Self::extend_persistent_ttl(&env, &earned_key);

        env.storage().persistent().set(&activation_key, &true);

        Self::extend_persistent_ttl(&env, &activation_key);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_ACTIVATION, TOPIC_REWARD),
            (
                campaign_id,
                verifier,
                agent,
                merchant_ref,
                milestone,
                rule.reward_amount,
            ),
        );

        Ok(rule.reward_amount)
    }

    /// Claims all currently allocated rewards for an agent
    /// within a single campaign.
    ///
    /// The agent must authorize the claim. The configured reward
    /// token is transferred from the campaign contract to the agent.
    pub fn claim_reward(env: Env, campaign_id: u64, agent: Address) -> Result<i128, CampaignError> {
        Self::require_initialized(&env)?;

        let mut campaign = Self::load_campaign(&env, campaign_id)?;

        agent.require_auth();

        let claimable_key = DataKey::ClaimableReward(campaign_id, agent.clone());

        let claimable: i128 = env.storage().persistent().get(&claimable_key).unwrap_or(0);

        if claimable <= 0 {
            return Err(CampaignError::NothingToClaim);
        }

        let new_claimed_amount = campaign
            .claimed_amount
            .checked_add(claimable)
            .ok_or(CampaignError::ArithmeticOverflow)?;

        // Claimed rewards must never exceed rewards previously allocated.
        if new_claimed_amount > campaign.allocated_amount {
            return Err(CampaignError::InvalidAccountingState);
        }

        // Effects first. If the token transfer fails, Soroban transaction
        // atomicity reverts these storage changes as well.
        campaign.claimed_amount = new_claimed_amount;

        env.storage()
            .persistent()
            .set(&DataKey::Campaign(campaign_id), &campaign);

        env.storage().persistent().set(&claimable_key, &0_i128);

        let token_client = token::Client::new(&env, &campaign.reward_token);

        token_client.transfer(&env.current_contract_address(), &agent, &claimable);

        env.events().publish(
            (TOPIC_CAMPAIGN, TOPIC_REWARD, TOPIC_CLAIM),
            (campaign_id, agent, claimable),
        );

        Ok(claimable)
    }

    /// Returns the agent's currently claimable reward for a campaign.
    pub fn claimable_reward(
        env: Env,
        campaign_id: u64,
        agent: Address,
    ) -> Result<i128, CampaignError> {
        Self::require_initialized(&env)?;

        Self::load_campaign(&env, campaign_id)?;

        let key = DataKey::ClaimableReward(campaign_id, agent);

        match env.storage().persistent().get(&key) {
            Some(amount) => {
                Self::extend_persistent_ttl(&env, &key);
                Ok(amount)
            }
            None => Ok(0),
        }
    }

    /// Returns the total amount an agent has earned in a campaign,
    /// including amounts already claimed.
    pub fn agent_total_earned(
        env: Env,
        campaign_id: u64,
        agent: Address,
    ) -> Result<i128, CampaignError> {
        Self::require_initialized(&env)?;

        Self::load_campaign(&env, campaign_id)?;

        let key = DataKey::AgentTotalEarned(campaign_id, agent);

        match env.storage().persistent().get(&key) {
            Some(amount) => {
                Self::extend_persistent_ttl(&env, &key);
                Ok(amount)
            }
            None => Ok(0),
        }
    }

    /// Returns an activation rule for a campaign milestone.
    pub fn get_activation_rule(
        env: Env,
        campaign_id: u64,
        milestone: String,
    ) -> Result<ActivationRule, CampaignError> {
        Self::require_initialized(&env)?;

        let key = DataKey::ActivationRule(campaign_id, milestone);

        let rule = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(CampaignError::RuleNotFound)?;

        Self::extend_persistent_ttl(&env, &key);

        Ok(rule)
    }

    /// Returns a campaign by ID.
    pub fn get_campaign(env: Env, campaign_id: u64) -> Result<Campaign, CampaignError> {
        Self::require_initialized(&env)?;

        Self::load_campaign(&env, campaign_id)
    }

    pub fn is_initialized(env: Env) -> bool {
        let initialized = env.storage().instance().has(&DataKey::Initialized);

        if initialized {
            Self::extend_instance_ttl(&env);
        }

        initialized
    }

    pub fn protocol_admin(env: Env) -> Result<Address, CampaignError> {
        Self::require_initialized(&env)?;

        env.storage()
            .instance()
            .get(&DataKey::ProtocolAdmin)
            .ok_or(CampaignError::NotInitialized)
    }

    pub fn next_campaign_id(env: Env) -> Result<u64, CampaignError> {
        Self::require_initialized(&env)?;

        env.storage()
            .instance()
            .get(&DataKey::NextCampaignId)
            .ok_or(CampaignError::NotInitialized)
    }

    fn extend_persistent_ttl(env: &Env, key: &DataKey) {
        let max_ttl = env.storage().max_ttl();
        let threshold = max_ttl / 2;

        env.storage()
            .persistent()
            .extend_ttl(key, threshold, max_ttl);
    }

    fn load_campaign(env: &Env, campaign_id: u64) -> Result<Campaign, CampaignError> {
        let key = DataKey::Campaign(campaign_id);

        let campaign = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(CampaignError::CampaignNotFound)?;

        Self::extend_persistent_ttl(env, &key);

        Ok(campaign)
    }

    fn extend_instance_ttl(env: &Env) {
        let max_ttl = env.storage().max_ttl();
        let threshold = max_ttl / 2;

        env.storage().instance().extend_ttl(threshold, max_ttl);
    }

    fn require_initialized(env: &Env) -> Result<(), CampaignError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(CampaignError::NotInitialized);
        }

        Self::extend_instance_ttl(env);

        Ok(())
    }
}

#[cfg(test)]
mod interface_tests {
    use super::*;
    use serde_json::Value;
    use soroban_sdk::xdr::{Limits, ReadXdr, ScSpecEntry, ScSpecFunctionV0, ScSpecTypeDef};
    use std::string::ToString;

    const CAMPAIGN_INTERFACE_SCHEMA: &str =
        include_str!("../../../schemas/campaign-interface.schema.json");
    const CAMPAIGN_INTERFACE_V0_1: &str =
        include_str!("../../../schemas/campaign-interface-v0.1.json");

    const PUBLIC_METHOD_NAMES: [&str; 21] = [
        "initialize",
        "create_campaign",
        "fund_campaign",
        "add_activation_rule",
        "pause_campaign",
        "resume_campaign",
        "close_campaign",
        "available_unused_funds",
        "withdraw_unused_funds",
        "add_verifier",
        "remove_verifier",
        "is_verifier",
        "verify_and_allocate_reward",
        "claim_reward",
        "claimable_reward",
        "agent_total_earned",
        "get_activation_rule",
        "get_campaign",
        "is_initialized",
        "protocol_admin",
        "next_campaign_id",
    ];

    const PUBLIC_TYPE_NAMES: [&str; 10] = [
        "Address",
        "String",
        "u64",
        "i128",
        "bool",
        "void",
        "CampaignError",
        "CampaignStatus",
        "Campaign",
        "ActivationRule",
    ];

    fn interface_item<'a>(interface: &'a Value, collection: &str, name: &str) -> &'a Value {
        interface[collection]
            .as_array()
            .expect("interface collection must be an array")
            .iter()
            .find(|item| item["name"] == name)
            .unwrap_or_else(|| panic!("interface {collection} must include {name}"))
    }

    fn assert_string_array(actual: &Value, expected: &[&str]) {
        let actual = actual.as_array().expect("value must be a string array");

        assert_eq!(actual.len(), expected.len());

        for (actual, expected) in actual.iter().zip(expected) {
            assert_eq!(actual.as_str(), Some(*expected));
        }
    }

    fn decode_function_spec<const N: usize>(encoded: [u8; N]) -> ScSpecFunctionV0 {
        match ScSpecEntry::from_xdr(encoded, Limits::none())
            .expect("generated campaign contract function spec must be valid XDR")
        {
            ScSpecEntry::FunctionV0(function) => function,
            _ => panic!("generated campaign function spec must contain a function"),
        }
    }

    fn contract_function_specs() -> std::vec::Vec<ScSpecFunctionV0> {
        std::vec![
            decode_function_spec(CampaignContract::spec_xdr_initialize()),
            decode_function_spec(CampaignContract::spec_xdr_create_campaign()),
            decode_function_spec(CampaignContract::spec_xdr_fund_campaign()),
            decode_function_spec(CampaignContract::spec_xdr_add_activation_rule()),
            decode_function_spec(CampaignContract::spec_xdr_pause_campaign()),
            decode_function_spec(CampaignContract::spec_xdr_resume_campaign()),
            decode_function_spec(CampaignContract::spec_xdr_close_campaign()),
            decode_function_spec(CampaignContract::spec_xdr_available_unused_funds(),),
            decode_function_spec(CampaignContract::spec_xdr_withdraw_unused_funds(),),
            decode_function_spec(CampaignContract::spec_xdr_add_verifier()),
            decode_function_spec(CampaignContract::spec_xdr_remove_verifier()),
            decode_function_spec(CampaignContract::spec_xdr_is_verifier()),
            decode_function_spec(CampaignContract::spec_xdr_verify_and_allocate_reward(),),
            decode_function_spec(CampaignContract::spec_xdr_claim_reward()),
            decode_function_spec(CampaignContract::spec_xdr_claimable_reward()),
            decode_function_spec(CampaignContract::spec_xdr_agent_total_earned(),),
            decode_function_spec(CampaignContract::spec_xdr_get_activation_rule(),),
            decode_function_spec(CampaignContract::spec_xdr_get_campaign()),
            decode_function_spec(CampaignContract::spec_xdr_is_initialized()),
            decode_function_spec(CampaignContract::spec_xdr_protocol_admin()),
            decode_function_spec(CampaignContract::spec_xdr_next_campaign_id()),
        ]
    }

    fn spec_type_name(spec_type: &ScSpecTypeDef) -> std::string::String {
        match spec_type {
            ScSpecTypeDef::Address => "Address".into(),
            ScSpecTypeDef::String => "String".into(),
            ScSpecTypeDef::U64 => "u64".into(),
            ScSpecTypeDef::I128 => "i128".into(),
            ScSpecTypeDef::Bool => "bool".into(),
            ScSpecTypeDef::Void => "void".into(),
            ScSpecTypeDef::Tuple(tuple) if tuple.value_types.is_empty() => "void".into(),
            ScSpecTypeDef::Udt(udt) => udt.name.to_string(),
            other => {
                panic!("unexpected campaign public interface type: {other:?}")
            }
        }
    }

    #[test]
    fn campaign_interface_schema_is_versioned_and_pins_current_names() {
        let schema: Value = serde_json::from_str(CAMPAIGN_INTERFACE_SCHEMA)
            .expect("campaign interface schema must be valid JSON");

        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(schema["properties"]["schema_version"]["const"], "1");
        assert_eq!(schema["properties"]["interface_version"]["const"], "0.1");

        assert_eq!(schema["properties"]["methods"]["minItems"], 21);
        assert_eq!(schema["properties"]["methods"]["maxItems"], 21);

        assert_string_array(
            &schema["$defs"]["method_name"]["enum"],
            &PUBLIC_METHOD_NAMES,
        );

        assert_string_array(&schema["$defs"]["type_name"]["enum"], &PUBLIC_TYPE_NAMES);
    }

    #[test]
    fn campaign_interface_document_matches_generated_contract_spec() {
        let interface: Value = serde_json::from_str(CAMPAIGN_INTERFACE_V0_1)
            .expect("campaign interface document must be valid JSON");

        assert_eq!(
            interface["schema"],
            "schemas/campaign-interface.schema.json"
        );
        assert_eq!(interface["schema_version"], "1");
        assert_eq!(interface["interface_version"], "0.1");
        assert_eq!(interface["contract"], "axionvera-campaign-contract");
        assert_eq!(interface["contract_version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(
            interface["source"],
            "contracts/campaign-contract/src/lib.rs"
        );

        let specs = contract_function_specs();
        let methods = interface["methods"]
            .as_array()
            .expect("interface methods must be an array");

        assert_eq!(specs.len(), PUBLIC_METHOD_NAMES.len());
        assert_eq!(methods.len(), specs.len());

        for (spec, expected_name) in specs.iter().zip(PUBLIC_METHOD_NAMES) {
            let spec_name = spec.name.to_string();

            assert_eq!(spec_name, expected_name);

            let method = interface_item(&interface, "methods", &spec_name);

            let arguments = method["arguments"]
                .as_array()
                .expect("method arguments must be an array");

            assert_eq!(arguments.len(), spec.inputs.len());

            for (position, (argument, input)) in
                arguments.iter().zip(spec.inputs.iter()).enumerate()
            {
                assert_eq!(argument["position"], position);
                assert_eq!(argument["name"], input.name.to_string());
                assert_eq!(argument["type"], spec_type_name(&input.type_));
            }

            assert_eq!(spec.outputs.len(), 1);

            let returns = &method["returns"];

            match &spec.outputs[0] {
                ScSpecTypeDef::Result(result) => {
                    assert_eq!(returns["kind"], "result");
                    assert_eq!(returns["ok"], spec_type_name(&result.ok_type));
                    assert_eq!(returns["error"], spec_type_name(&result.error_type));
                }
                output => {
                    assert_eq!(returns["kind"], "value");
                    assert_eq!(returns["type"], spec_type_name(output));
                }
            }
        }
    }

    struct ExpectedSemantics<'a> {
        name: &'a str,
        auth_source: &'a str,
        auth_argument: Option<&'a str>,
        initialization: &'a str,
        mutability: &'a str,
        emits: &'a [&'a str],
    }

    #[test]
    fn campaign_interface_document_covers_current_method_semantics() {
        let interface: Value = serde_json::from_str(CAMPAIGN_INTERFACE_V0_1)
            .expect("campaign interface document must be valid JSON");

        let expected = [
            ExpectedSemantics {
                name: "initialize",
                auth_source: "argument",
                auth_argument: Some("protocol_admin"),
                initialization: "must_be_uninitialized",
                mutability: "write",
                emits: &["init"],
            },
            ExpectedSemantics {
                name: "create_campaign",
                auth_source: "argument",
                auth_argument: Some("admin"),
                initialization: "required",
                mutability: "write",
                emits: &["created"],
            },
            ExpectedSemantics {
                name: "fund_campaign",
                auth_source: "stored_campaign_admin",
                auth_argument: None,
                initialization: "required",
                mutability: "write",
                emits: &["funded"],
            },
            ExpectedSemantics {
                name: "add_activation_rule",
                auth_source: "stored_campaign_admin",
                auth_argument: None,
                initialization: "required",
                mutability: "write",
                emits: &["rule_added"],
            },
            ExpectedSemantics {
                name: "pause_campaign",
                auth_source: "stored_campaign_admin",
                auth_argument: None,
                initialization: "required",
                mutability: "write",
                emits: &["paused"],
            },
            ExpectedSemantics {
                name: "resume_campaign",
                auth_source: "stored_campaign_admin",
                auth_argument: None,
                initialization: "required",
                mutability: "write",
                emits: &["resumed"],
            },
            ExpectedSemantics {
                name: "close_campaign",
                auth_source: "stored_campaign_admin",
                auth_argument: None,
                initialization: "required",
                mutability: "write",
                emits: &["closed"],
            },
            ExpectedSemantics {
                name: "available_unused_funds",
                auth_source: "none",
                auth_argument: None,
                initialization: "required",
                mutability: "read",
                emits: &[],
            },
            ExpectedSemantics {
                name: "withdraw_unused_funds",
                auth_source: "stored_campaign_admin",
                auth_argument: None,
                initialization: "required",
                mutability: "write",
                emits: &["unused_withdraw"],
            },
            ExpectedSemantics {
                name: "add_verifier",
                auth_source: "stored_campaign_admin",
                auth_argument: None,
                initialization: "required",
                mutability: "write",
                emits: &["verifier_added"],
            },
            ExpectedSemantics {
                name: "remove_verifier",
                auth_source: "stored_campaign_admin",
                auth_argument: None,
                initialization: "required",
                mutability: "write",
                emits: &["verifier_removed"],
            },
            ExpectedSemantics {
                name: "is_verifier",
                auth_source: "none",
                auth_argument: None,
                initialization: "required",
                mutability: "read",
                emits: &[],
            },
            ExpectedSemantics {
                name: "verify_and_allocate_reward",
                auth_source: "argument",
                auth_argument: Some("verifier"),
                initialization: "required",
                mutability: "write",
                emits: &["reward_allocated"],
            },
            ExpectedSemantics {
                name: "claim_reward",
                auth_source: "argument",
                auth_argument: Some("agent"),
                initialization: "required",
                mutability: "write",
                emits: &["reward_claimed"],
            },
            ExpectedSemantics {
                name: "claimable_reward",
                auth_source: "none",
                auth_argument: None,
                initialization: "required",
                mutability: "read",
                emits: &[],
            },
            ExpectedSemantics {
                name: "agent_total_earned",
                auth_source: "none",
                auth_argument: None,
                initialization: "required",
                mutability: "read",
                emits: &[],
            },
            ExpectedSemantics {
                name: "get_activation_rule",
                auth_source: "none",
                auth_argument: None,
                initialization: "required",
                mutability: "read",
                emits: &[],
            },
            ExpectedSemantics {
                name: "get_campaign",
                auth_source: "none",
                auth_argument: None,
                initialization: "required",
                mutability: "read",
                emits: &[],
            },
            ExpectedSemantics {
                name: "is_initialized",
                auth_source: "none",
                auth_argument: None,
                initialization: "not_required",
                mutability: "read",
                emits: &[],
            },
            ExpectedSemantics {
                name: "protocol_admin",
                auth_source: "none",
                auth_argument: None,
                initialization: "required",
                mutability: "read",
                emits: &[],
            },
            ExpectedSemantics {
                name: "next_campaign_id",
                auth_source: "none",
                auth_argument: None,
                initialization: "required",
                mutability: "read",
                emits: &[],
            },
        ];

        for expected in &expected {
            let method = interface_item(&interface, "methods", expected.name);

            let authorization = &method["authorization"];

            assert_eq!(
                authorization["required"],
                Value::Bool(expected.auth_source != "none")
            );

            assert_eq!(authorization["source"], expected.auth_source);

            match expected.auth_argument {
                Some(argument) => {
                    assert_eq!(authorization["address_argument"], argument);
                    assert_eq!(authorization["mechanism"], "Address.require_auth");
                }
                None if expected.auth_source == "none" => {
                    assert!(authorization["address_argument"].is_null());
                    assert_eq!(authorization["mechanism"], "none");
                }
                None => {
                    assert!(authorization["address_argument"].is_null());
                    assert_eq!(authorization["mechanism"], "Address.require_auth");
                }
            }

            assert_eq!(method["initialization"], expected.initialization);
            assert_eq!(method["mutability"], expected.mutability);
            assert_string_array(&method["emits"], expected.emits);
        }
    }

    #[test]
    fn campaign_interface_document_matches_current_errors() {
        let interface: Value = serde_json::from_str(CAMPAIGN_INTERFACE_V0_1)
            .expect("campaign interface document must be valid JSON");

        let schema: Value = serde_json::from_str(CAMPAIGN_INTERFACE_SCHEMA)
            .expect("campaign interface schema must be valid JSON");

        let expected: [(&str, u64, &[&str]); 27] = [
            ("AlreadyInitialized", 1, &["initialize"]),
            (
                "NotInitialized",
                2,
                &[
                    "create_campaign",
                    "fund_campaign",
                    "add_activation_rule",
                    "pause_campaign",
                    "resume_campaign",
                    "close_campaign",
                    "available_unused_funds",
                    "withdraw_unused_funds",
                    "add_verifier",
                    "remove_verifier",
                    "is_verifier",
                    "verify_and_allocate_reward",
                    "claim_reward",
                    "claimable_reward",
                    "agent_total_earned",
                    "get_activation_rule",
                    "get_campaign",
                    "protocol_admin",
                    "next_campaign_id",
                ],
            ),
            ("InvalidTimeRange", 3, &["create_campaign"]),
            ("InvalidCap", 4, &["create_campaign"]),
            (
                "CampaignNotFound",
                5,
                &[
                    "fund_campaign",
                    "add_activation_rule",
                    "pause_campaign",
                    "resume_campaign",
                    "close_campaign",
                    "available_unused_funds",
                    "withdraw_unused_funds",
                    "add_verifier",
                    "remove_verifier",
                    "is_verifier",
                    "verify_and_allocate_reward",
                    "claim_reward",
                    "claimable_reward",
                    "agent_total_earned",
                    "get_campaign",
                ],
            ),
            ("CampaignIdOverflow", 6, &["create_campaign"]),
            (
                "InvalidAmount",
                7,
                &["fund_campaign", "withdraw_unused_funds"],
            ),
            (
                "CampaignNotActive",
                8,
                &[
                    "fund_campaign",
                    "add_activation_rule",
                    "pause_campaign",
                    "add_verifier",
                    "remove_verifier",
                    "verify_and_allocate_reward",
                ],
            ),
            (
                "ArithmeticOverflow",
                9,
                &[
                    "fund_campaign",
                    "withdraw_unused_funds",
                    "verify_and_allocate_reward",
                    "claim_reward",
                ],
            ),
            ("InvalidRewardAmount", 10, &["add_activation_rule"]),
            ("RuleAlreadyExists", 11, &["add_activation_rule"]),
            (
                "RuleNotFound",
                12,
                &["verify_and_allocate_reward", "get_activation_rule"],
            ),
            ("VerifierAlreadyExists", 13, &["add_verifier"]),
            ("VerifierNotFound", 14, &["remove_verifier"]),
            ("UnauthorizedVerifier", 15, &["verify_and_allocate_reward"]),
            ("RuleDisabled", 16, &["verify_and_allocate_reward"]),
            ("DuplicateActivation", 17, &["verify_and_allocate_reward"]),
            (
                "InsufficientCampaignFunds",
                18,
                &["verify_and_allocate_reward"],
            ),
            ("AgentCapExceeded", 19, &["verify_and_allocate_reward"]),
            ("NothingToClaim", 20, &["claim_reward"]),
            (
                "InvalidAccountingState",
                21,
                &[
                    "available_unused_funds",
                    "withdraw_unused_funds",
                    "claim_reward",
                ],
            ),
            ("CampaignNotPaused", 22, &["resume_campaign"]),
            ("CampaignAlreadyClosed", 23, &["close_campaign"]),
            ("CampaignNotClosed", 24, &["withdraw_unused_funds"]),
            ("InsufficientUnusedFunds", 25, &["withdraw_unused_funds"]),
            ("CampaignNotStarted", 26, &["verify_and_allocate_reward"]),
            (
                "CampaignEnded",
                27,
                &[
                    "fund_campaign",
                    "add_activation_rule",
                    "pause_campaign",
                    "resume_campaign",
                    "add_verifier",
                    "remove_verifier",
                    "verify_and_allocate_reward",
                ],
            ),
        ];

        assert_eq!(schema["properties"]["errors"]["minItems"], 27);
        assert_eq!(schema["properties"]["errors"]["maxItems"], 27);

        let expected_names: std::vec::Vec<&str> =
            expected.iter().map(|(name, _, _)| *name).collect();

        let schema_names = schema["$defs"]["error_name"]["enum"]
            .as_array()
            .expect("error-name enum must be an array");

        assert_eq!(schema_names.len(), expected_names.len());

        for (actual, expected_name) in schema_names.iter().zip(expected_names.iter()) {
            assert_eq!(actual.as_str(), Some(*expected_name));
        }

        let errors = interface["errors"]
            .as_array()
            .expect("interface errors must be an array");

        assert_eq!(errors.len(), expected.len());

        for (error, (name, code, returned_by)) in errors.iter().zip(expected) {
            assert_eq!(error["name"], name);
            assert_eq!(error["code"].as_u64(), Some(code));

            assert_string_array(&error["returned_by"], returned_by);

            assert!(!error["description"]
                .as_str()
                .expect("error description must be a string")
                .is_empty());
        }
    }

    fn decode_spec_entry<const N: usize>(encoded: [u8; N]) -> ScSpecEntry {
        ScSpecEntry::from_xdr(encoded, Limits::none())
            .expect("generated campaign type spec must be valid XDR")
    }

    fn assert_generated_struct_type<const N: usize>(
        interface: &Value,
        expected_name: &str,
        encoded: [u8; N],
    ) {
        let spec = match decode_spec_entry(encoded) {
            ScSpecEntry::UdtStructV0(spec) => spec,
            other => panic!("expected {expected_name} to generate UdtStructV0, got {other:?}"),
        };

        assert_eq!(spec.name.to_string(), expected_name);

        let documented = &interface["types"][expected_name];

        assert_eq!(documented["kind"], "struct");

        let fields = documented["fields"]
            .as_array()
            .expect("documented struct fields must be an array");

        assert_eq!(fields.len(), spec.fields.len());

        for (position, (documented_field, generated_field)) in
            fields.iter().zip(spec.fields.iter()).enumerate()
        {
            assert_eq!(documented_field["position"], position);
            assert_eq!(documented_field["name"], generated_field.name.to_string());
            assert_eq!(
                documented_field["type"],
                spec_type_name(&generated_field.type_)
            );
        }
    }

    #[test]
    fn campaign_interface_document_matches_generated_public_types() {
        let interface: Value = serde_json::from_str(CAMPAIGN_INTERFACE_V0_1)
            .expect("campaign interface document must be valid JSON");

        assert_eq!(interface["types"]["Address"]["kind"], "primitive");
        assert_eq!(interface["types"]["Address"]["soroban_type"], "Address");
        assert_eq!(interface["types"]["String"]["soroban_type"], "String");
        assert_eq!(interface["types"]["u64"]["soroban_type"], "u64");
        assert_eq!(interface["types"]["i128"]["soroban_type"], "i128");
        assert_eq!(interface["types"]["bool"]["soroban_type"], "bool");
        assert_eq!(interface["types"]["void"]["soroban_type"], "()");
        assert_eq!(
            interface["types"]["CampaignError"]["kind"],
            "contract_error"
        );
        assert_eq!(
            interface["types"]["CampaignError"]["soroban_type"],
            "contracterror(u32)"
        );

        let status_spec = match decode_spec_entry(CampaignStatus::spec_xdr()) {
            ScSpecEntry::UdtUnionV0(spec) => spec,
            other => panic!("expected CampaignStatus to generate UdtUnionV0, got {other:?}"),
        };

        assert_eq!(status_spec.name.to_string(), "CampaignStatus");

        let documented_status = &interface["types"]["CampaignStatus"];

        assert_eq!(documented_status["kind"], "union");

        let documented_cases = documented_status["cases"]
            .as_array()
            .expect("CampaignStatus cases must be an array");

        assert_eq!(documented_cases.len(), status_spec.cases.len());

        for (position, (documented_case, generated_case)) in documented_cases
            .iter()
            .zip(status_spec.cases.iter())
            .enumerate()
        {
            let generated_name = match generated_case {
                soroban_sdk::xdr::ScSpecUdtUnionCaseV0::VoidV0(case) => case.name.to_string(),
                other => panic!("CampaignStatus cases must be void variants, got {other:?}"),
            };

            assert_eq!(documented_case["position"], position);
            assert_eq!(documented_case["name"], generated_name);
            assert_eq!(documented_case["kind"], "void");
        }

        assert_generated_struct_type(&interface, "Campaign", Campaign::spec_xdr());

        assert_generated_struct_type(&interface, "ActivationRule", ActivationRule::spec_xdr());
    }
    #[test]
    fn campaign_interface_document_matches_current_events() {
        let interface: Value = serde_json::from_str(CAMPAIGN_INTERFACE_V0_1)
            .expect("campaign interface document must be valid JSON");

        struct ExpectedEvent<'a> {
            name: &'a str,
            emitted_by: &'a str,
            topics: &'a [&'a str],
            data_kind: &'a str,
            fields: &'a [(&'a str, &'a str)],
        }

        let expected_events = [
            ExpectedEvent {
                name: "init",
                emitted_by: "initialize",
                topics: &["campaign", "init"],
                data_kind: "value",
                fields: &[("protocol_admin", "Address")],
            },
            ExpectedEvent {
                name: "created",
                emitted_by: "create_campaign",
                topics: &["campaign", "created"],
                data_kind: "tuple",
                fields: &[
                    ("campaign_id", "u64"),
                    ("admin", "Address"),
                    ("reward_token", "Address"),
                ],
            },
            ExpectedEvent {
                name: "funded",
                emitted_by: "fund_campaign",
                topics: &["campaign", "funded"],
                data_kind: "tuple",
                fields: &[
                    ("campaign_id", "u64"),
                    ("admin", "Address"),
                    ("amount", "i128"),
                ],
            },
            ExpectedEvent {
                name: "rule_added",
                emitted_by: "add_activation_rule",
                topics: &["campaign", "rule", "added"],
                data_kind: "tuple",
                fields: &[
                    ("campaign_id", "u64"),
                    ("milestone", "String"),
                    ("reward_amount", "i128"),
                ],
            },
            ExpectedEvent {
                name: "paused",
                emitted_by: "pause_campaign",
                topics: &["campaign", "paused"],
                data_kind: "tuple",
                fields: &[("campaign_id", "u64"), ("admin", "Address")],
            },
            ExpectedEvent {
                name: "resumed",
                emitted_by: "resume_campaign",
                topics: &["campaign", "resumed"],
                data_kind: "tuple",
                fields: &[("campaign_id", "u64"), ("admin", "Address")],
            },
            ExpectedEvent {
                name: "closed",
                emitted_by: "close_campaign",
                topics: &["campaign", "closed"],
                data_kind: "tuple",
                fields: &[("campaign_id", "u64"), ("admin", "Address")],
            },
            ExpectedEvent {
                name: "unused_withdraw",
                emitted_by: "withdraw_unused_funds",
                topics: &["campaign", "unused", "withdraw"],
                data_kind: "tuple",
                fields: &[
                    ("campaign_id", "u64"),
                    ("admin", "Address"),
                    ("amount", "i128"),
                ],
            },
            ExpectedEvent {
                name: "verifier_added",
                emitted_by: "add_verifier",
                topics: &["campaign", "verifyr", "added"],
                data_kind: "tuple",
                fields: &[("campaign_id", "u64"), ("verifier", "Address")],
            },
            ExpectedEvent {
                name: "verifier_removed",
                emitted_by: "remove_verifier",
                topics: &["campaign", "verifyr", "removed"],
                data_kind: "tuple",
                fields: &[("campaign_id", "u64"), ("verifier", "Address")],
            },
            ExpectedEvent {
                name: "reward_allocated",
                emitted_by: "verify_and_allocate_reward",
                topics: &["campaign", "activate", "reward"],
                data_kind: "tuple",
                fields: &[
                    ("campaign_id", "u64"),
                    ("verifier", "Address"),
                    ("agent", "Address"),
                    ("merchant_ref", "String"),
                    ("milestone", "String"),
                    ("reward_amount", "i128"),
                ],
            },
            ExpectedEvent {
                name: "reward_claimed",
                emitted_by: "claim_reward",
                topics: &["campaign", "reward", "claim"],
                data_kind: "tuple",
                fields: &[
                    ("campaign_id", "u64"),
                    ("agent", "Address"),
                    ("claimable", "i128"),
                ],
            },
        ];

        let events = interface["events"]
            .as_array()
            .expect("events must be an array");

        assert_eq!(events.len(), expected_events.len());

        for expected in expected_events {
            let event = interface_item(&interface, "events", expected.name);

            assert_eq!(event["emitted_by"].as_str(), Some(expected.emitted_by));
            assert_eq!(event["type"].as_str(), Some("contract"));
            assert_eq!(event["emission"].as_str(), Some("on_success"));
            assert_eq!(event["failed_calls_emit"].as_bool(), Some(false));

            let topics = event["topics"]
                .as_array()
                .expect("event topics must be an array");

            assert_eq!(topics.len(), expected.topics.len());

            for (position, (topic, expected_value)) in
                topics.iter().zip(expected.topics.iter()).enumerate()
            {
                assert_eq!(topic["position"].as_u64(), Some(position as u64));
                assert_eq!(topic["type"].as_str(), Some("Symbol"));
                assert_eq!(topic["value"].as_str(), Some(*expected_value));
            }

            let data = &event["data"];

            assert_eq!(data["kind"].as_str(), Some(expected.data_kind));

            let fields = data["fields"]
                .as_array()
                .expect("event data fields must be an array");

            assert_eq!(fields.len(), expected.fields.len());

            for (position, (field, (expected_name, expected_type))) in
                fields.iter().zip(expected.fields.iter()).enumerate()
            {
                assert_eq!(field["position"].as_u64(), Some(position as u64));
                assert_eq!(field["name"].as_str(), Some(*expected_name));
                assert_eq!(field["type"].as_str(), Some(*expected_type));
            }
        }

        assert_eq!(
            interface["event_policy"]["failed_calls_emit"].as_bool(),
            Some(false)
        );
    }
}
