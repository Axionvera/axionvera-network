use soroban_sdk::{contracttype, Address, Env};

use crate::errors::{ArithmeticError, AuthorizationError, StateError, VaultError};

pub const PRECISION_FACTOR: i128 = 1_000_000_000;

const INSTANCE_TTL_THRESHOLD: u32 = 100;
const INSTANCE_TTL_EXTEND_TO: u32 = 1_000;

const PERSISTENT_TTL_THRESHOLD: u32 = 100;
const PERSISTENT_TTL_EXTEND_TO: u32 = 10_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Initialized,
    Admin,
    DepositToken,
    RewardToken,
    TotalDeposits,
    RewardIndex,
    ReentrancyGuard,
    UserBalance(Address),
    UserRewardIndex(Address),
    UserRewards(Address),
    IsPaused,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultState {
    pub admin: Address,
    pub deposit_token: Address,
    pub reward_token: Address,
    pub total_deposits: i128,
    pub reward_index: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserPosition {
    pub balance: i128,
    pub reward_index: i128,
    pub rewards: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserRewardSnapshot {
    pub reward_index: i128,
    pub rewards: i128,
}

// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

pub fn is_initialized(e: &Env) -> bool {
    e.storage().instance().has(&DataKey::Initialized)
}

pub fn require_initialized(e: &Env) -> Result<(), VaultError> {
    if is_initialized(e) {
        Ok(())
    } else {
        Err(StateError::NotInitialized.into())
    }
}

pub fn initialize_state(
    e: &Env,
    admin: &Address,
    deposit_token: &Address,
    reward_token: &Address,
) {
    let state = VaultState {
        admin: admin.clone(),
        deposit_token: deposit_token.clone(),
        reward_token: reward_token.clone(),
        total_deposits: 0,
        reward_index: 0,
    };
    e.storage().instance().set(&DataKey::Initialized, &true);
    e.storage().instance().set(&DataKey::Admin, &state);
    bump_instance_ttl(e);
}

// ---------------------------------------------------------------------------
// State (global)
// ---------------------------------------------------------------------------

pub fn get_state(e: &Env) -> Result<VaultState, VaultError> {
    require_initialized(e)?;
    let state: VaultState = e
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(StateError::NotInitialized)?;
    bump_instance_ttl(e);
    Ok(state)
}

fn set_state(e: &Env, state: &VaultState) {
    e.storage().instance().set(&DataKey::Admin, state);
    bump_instance_ttl(e);
}

pub fn get_admin(e: &Env) -> Result<Address, VaultError> {
    Ok(get_state(e)?.admin)
}

pub fn get_deposit_token(e: &Env) -> Result<Address, VaultError> {
    Ok(get_state(e)?.deposit_token)
}

pub fn get_reward_token(e: &Env) -> Result<Address, VaultError> {
    Ok(get_state(e)?.reward_token)
}

pub fn get_total_deposits(e: &Env) -> Result<i128, VaultError> {
    Ok(get_state(e)?.total_deposits)
}

pub fn get_reward_index(e: &Env) -> Result<i128, VaultError> {
    Ok(get_state(e)?.reward_index)
}

// ---------------------------------------------------------------------------
// Reentrancy guard
// ---------------------------------------------------------------------------

pub fn enter_non_reentrant(e: &Env) -> Result<(), VaultError> {
    if e.storage()
        .instance()
        .get::<_, bool>(&DataKey::ReentrancyGuard)
        .unwrap_or(false)
    {
        return Err(AuthorizationError::ReentrancyDetected.into());
    }
    e.storage()
        .instance()
        .set(&DataKey::ReentrancyGuard, &true);
    bump_instance_ttl(e);
    Ok(())
}

pub fn exit_non_reentrant(e: &Env) {
    e.storage()
        .instance()
        .set(&DataKey::ReentrancyGuard, &false);
}

// ---------------------------------------------------------------------------
// User position
// ---------------------------------------------------------------------------

fn get_user_position_unchecked(e: &Env, user: &Address) -> UserPosition {
    let balance: i128 = e
        .storage()
        .persistent()
        .get(&DataKey::UserBalance(user.clone()))
        .unwrap_or(0);
    let reward_index: i128 = e
        .storage()
        .persistent()
        .get(&DataKey::UserRewardIndex(user.clone()))
        .unwrap_or(0);
    let rewards: i128 = e
        .storage()
        .persistent()
        .get(&DataKey::UserRewards(user.clone()))
        .unwrap_or(0);
    UserPosition {
        balance,
        reward_index,
        rewards,
    }
}

fn set_user_position(e: &Env, user: &Address, position: &UserPosition) {
    let bal_key = DataKey::UserBalance(user.clone());
    let idx_key = DataKey::UserRewardIndex(user.clone());
    let rew_key = DataKey::UserRewards(user.clone());

    if position.balance == 0 {
        e.storage().persistent().remove(&bal_key);
    } else {
        e.storage().persistent().set(&bal_key, &position.balance);
        bump_persistent_ttl(e, &bal_key);
    }

    e.storage()
        .persistent()
        .set(&idx_key, &position.reward_index);
    bump_persistent_ttl(e, &idx_key);

    if position.rewards == 0 {
        e.storage().persistent().remove(&rew_key);
    } else {
        e.storage().persistent().set(&rew_key, &position.rewards);
        bump_persistent_ttl(e, &rew_key);
    }
}

pub fn get_user_balance(e: &Env, user: &Address) -> Result<i128, VaultError> {
    require_initialized(e)?;
    Ok(get_user_position_unchecked(e, user).balance)
}

// ---------------------------------------------------------------------------
// Mutating store operations
// ---------------------------------------------------------------------------

pub fn store_deposit(
    e: &Env,
    user: &Address,
    amount: i128,
) -> Result<(VaultState, UserPosition), VaultError> {
    let mut state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user);
    accrue_position_rewards(&state, &mut position)?;

    position.balance = position
        .balance
        .checked_add(amount)
        .ok_or(VaultError::MathOverflow)?;
    state.total_deposits = state
        .total_deposits
        .checked_add(amount)
        .ok_or(VaultError::MathOverflow)?;

    set_state(e, &state);
    set_user_position(e, user, &position);
    Ok((state, position))
}

pub fn store_withdraw(
    e: &Env,
    user: &Address,
    amount: i128,
) -> Result<(VaultState, UserPosition), VaultError> {
    let mut state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user);
    accrue_position_rewards(&state, &mut position)?;

    if position.balance < amount {
        return Err(VaultError::InsufficientBalance);
    }
    if state.total_deposits < amount {
        return Err(StateError::InvalidState.into());
    }

    position.balance = position
        .balance
        .checked_sub(amount)
        .ok_or(VaultError::MathOverflow)?;
    state.total_deposits = state
        .total_deposits
        .checked_sub(amount)
        .ok_or(VaultError::MathOverflow)?;

    set_state(e, &state);
    set_user_position(e, user, &position);
    Ok((state, position))
}

pub fn store_reward_distribution(e: &Env, amount: i128) -> Result<VaultState, VaultError> {
    let mut state = get_state(e)?;
    let increment = checked_reward_index_increment(amount, state.total_deposits)?;

    state.reward_index = state
        .reward_index
        .checked_add(increment)
        .ok_or(VaultError::MathOverflow)?;

    set_state(e, &state);
    Ok(state)
}

pub fn store_claimable_rewards(e: &Env, user: &Address) -> Result<i128, VaultError> {
    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user);
    accrue_position_rewards(&state, &mut position)?;

    let claimable = position.rewards;
    position.rewards = 0;
    set_user_position(e, user, &position);

    Ok(claimable)
}

// ---------------------------------------------------------------------------
// Read-only reward preview
// ---------------------------------------------------------------------------

pub fn preview_user_rewards(e: &Env, user: &Address) -> Result<UserRewardSnapshot, VaultError> {
    require_initialized(e)?;
    let state = get_state(e)?;
    let position = get_user_position_unchecked(e, user);

    if state.reward_index == position.reward_index || position.balance == 0 {
        return Ok(UserRewardSnapshot {
            reward_index: state.reward_index,
            rewards: position.rewards,
        });
    }

    let delta = state
        .reward_index
        .checked_sub(position.reward_index)
        .ok_or(VaultError::MathOverflow)?;
    let accrued = checked_accrued_rewards(position.balance, delta)?;
    let rewards = position
        .rewards
        .checked_add(accrued)
        .ok_or(VaultError::MathOverflow)?;

    Ok(UserRewardSnapshot {
        reward_index: state.reward_index,
        rewards,
    })
}

pub fn pending_user_rewards_view(e: &Env, user: &Address) -> Result<i128, VaultError> {
    Ok(preview_user_rewards(e, user)?.rewards)
}

// ---------------------------------------------------------------------------
// Precision math  (issue #81)
// ---------------------------------------------------------------------------

/// Computes the reward index increment for a distribution.
/// Formula: (amount * PRECISION_FACTOR) / total_deposits
pub(crate) fn checked_reward_index_increment(
    amount: i128,
    total_deposits: i128,
) -> Result<i128, VaultError> {
    if total_deposits <= 0 {
        return Err(VaultError::NoDeposits);
    }

    let scaled = amount
        .checked_mul(PRECISION_FACTOR)
        .ok_or(VaultError::MathOverflow)?;
    let increment = scaled
        .checked_div(total_deposits)
        .ok_or(VaultError::from(ArithmeticError::RewardCalculationFailed))?;

    if increment <= 0 {
        return Err(VaultError::from(ArithmeticError::ZeroRewardIncrement));
    }

    Ok(increment)
}

/// Computes a user's accrued rewards from an index delta.
/// Formula: (balance * index_delta) / PRECISION_FACTOR
pub(crate) fn checked_accrued_rewards(
    balance: i128,
    index_delta: i128,
) -> Result<i128, VaultError> {
    let raw = balance
        .checked_mul(index_delta)
        .ok_or(VaultError::MathOverflow)?;
    raw.checked_div(PRECISION_FACTOR)
        .ok_or(VaultError::from(ArithmeticError::RewardCalculationFailed))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn accrue_position_rewards(
    state: &VaultState,
    position: &mut UserPosition,
) -> Result<(), VaultError> {
    if state.reward_index == position.reward_index || position.balance == 0 {
        position.reward_index = state.reward_index;
        return Ok(());
    }

    let delta = state
        .reward_index
        .checked_sub(position.reward_index)
        .ok_or(VaultError::MathOverflow)?;
    let accrued = checked_accrued_rewards(position.balance, delta)?;

    if accrued > 0 {
        position.rewards = position
            .rewards
            .checked_add(accrued)
            .ok_or(VaultError::MathOverflow)?;
    }

    position.reward_index = state.reward_index;
    Ok(())
}

fn bump_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

fn bump_persistent_ttl(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}
