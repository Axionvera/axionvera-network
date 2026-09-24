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
