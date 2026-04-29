use soroban_sdk::{contracttype, Address, Env};

use crate::errors::{ArithmeticError, AuthorizationError, BalanceError, StateError, VaultError};

pub const PRECISION_FACTOR: i128 = 1_000_000_000;

const INSTANCE_TTL_THRESHOLD: u32 = 100;
const INSTANCE_TTL_EXTEND_TO: u32 = 1_000;

const PERSISTENT_TTL_THRESHOLD: u32 = 100;
const PERSISTENT_TTL_EXTEND_TO: u32 = 10_000;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Initialized,
    ReentrancyGuard,
    Admin,
    PendingAdmin,
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
pub struct VaultState {
    pub admin: Address,
    pub deposit_token: Address,
    pub reward_token: Address,
    pub total_deposits: i128,
    pub reward_index: i128,
}

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
        bump_instance_ttl(e);
        Ok(())
    } else {
        Err(StateError::NotInitialized.into())
    }
}

pub fn initialize_state(e: &Env, admin: &Address, deposit_token: &Address, reward_token: &Address) {
    e.storage().instance().set(&DataKey::Initialized, &true);
    e.storage().instance().set(&DataKey::Admin, admin);
    e.storage().instance().remove(&DataKey::PendingAdmin);
    e.storage()
        .instance()
        .set(&DataKey::DepositToken, deposit_token);
    e.storage()
        .instance()
        .set(&DataKey::RewardToken, reward_token);
    e.storage().instance().set(&DataKey::TotalDeposits, &0_i128);
    e.storage().instance().set(&DataKey::RewardIndex, &0_i128);
    e.storage()
        .instance()
        .set(&DataKey::ReentrancyGuard, &false);
    bump_instance_ttl(e);
}

pub fn enter_non_reentrant(e: &Env) -> Result<(), VaultError> {
    if e.storage()
        .instance()
        .get::<_, bool>(&DataKey::ReentrancyGuard)
        .unwrap_or(false)
    {
        return Err(AuthorizationError::ReentrancyDetected.into());
    }

    e.storage().instance().set(&DataKey::ReentrancyGuard, &true);
    bump_instance_ttl(e);
    Ok(())
}

pub fn exit_non_reentrant(e: &Env) {
    e.storage()
        .instance()
        .set(&DataKey::ReentrancyGuard, &false);
    bump_instance_ttl(e);
}

pub fn get_state(e: &Env) -> Result<VaultState, VaultError> {
    Ok(VaultState {
        admin: get_admin(e)?,
        deposit_token: get_deposit_token(e)?,
        reward_token: get_reward_token(e)?,
        total_deposits: get_total_deposits(e)?,
        reward_index: get_reward_index(e)?,
    })
}

pub fn get_admin(e: &Env) -> Result<Address, VaultError> {
    require_initialized(e)?;
    let admin = e
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(StateError::InvalidState)?;
    bump_instance_ttl(e);
    Ok(admin)
}

// ---------------------------------------------------------------------------
// State (global)
// ---------------------------------------------------------------------------

pub fn get_pending_admin(e: &Env) -> Result<Option<Address>, VaultError> {
    require_initialized(e)?;
    let pending = e.storage().instance().get(&DataKey::PendingAdmin);
    bump_instance_ttl(e);
    Ok(pending)
}

pub fn set_pending_admin(e: &Env, pending_admin: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::PendingAdmin, pending_admin);
    bump_instance_ttl(e);
}

pub fn clear_pending_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::PendingAdmin);
    bump_instance_ttl(e);
}

pub fn get_deposit_token(e: &Env) -> Result<Address, VaultError> {
    require_initialized(e)?;
    let token = e
        .storage()
        .instance()
        .get(&DataKey::DepositToken)
        .ok_or(StateError::InvalidState)?;
    bump_instance_ttl(e);
    Ok(token)
}

pub fn get_reward_token(e: &Env) -> Result<Address, VaultError> {
    require_initialized(e)?;
    let token = e
        .storage()
        .instance()
        .get(&DataKey::RewardToken)
        .ok_or(StateError::InvalidState)?;
    bump_instance_ttl(e);
    Ok(token)
}

pub fn get_total_deposits(e: &Env) -> Result<i128, VaultError> {
    require_initialized(e)?;
    let total = e
        .storage()
        .instance()
        .get(&DataKey::TotalDeposits)
        .ok_or(StateError::InvalidState)?;
    bump_instance_ttl(e);
    Ok(total)
}

pub fn get_reward_index(e: &Env) -> Result<i128, VaultError> {
    Ok(get_state(e)?.reward_index)
}

pub fn get_reward_index(e: &Env) -> Result<i128, VaultError> {
    require_initialized(e)?;
    let reward_index = e
        .storage()
        .instance()
        .get(&DataKey::RewardIndex)
        .ok_or(StateError::InvalidState)?;
    bump_instance_ttl(e);
    Ok(reward_index)
}

pub fn set_reward_index(e: &Env, reward_index: i128) {
    e.storage()
        .instance()
        .set(&DataKey::RewardIndex, &reward_index);
    bump_instance_ttl(e);
}

pub fn get_user_position(e: &Env, user: &Address) -> Result<UserPosition, VaultError> {
    require_initialized(e)?;
    Ok(get_user_position_unchecked(e, user))
}

fn get_user_position_unchecked(e: &Env, user: &Address) -> UserPosition {
    let balance_key = DataKey::UserBalance(user.clone());
    let reward_index_key = DataKey::UserRewardIndex(user.clone());
    let rewards_key = DataKey::UserRewards(user.clone());

    let balance = e.storage().persistent().get(&balance_key).unwrap_or(0_i128);
    let reward_index = e
        .storage()
        .persistent()
        .get(&reward_index_key)
        .unwrap_or(0_i128);
    let rewards = e.storage().persistent().get(&rewards_key).unwrap_or(0_i128);

    if balance != 0 {
        bump_persistent_ttl(e, &balance_key);
    }
    if reward_index != 0 {
        bump_persistent_ttl(e, &reward_index_key);
    }
    if rewards != 0 {
        bump_persistent_ttl(e, &rewards_key);
    }

    UserPosition {
        balance,
        reward_index,
        rewards,
    }
}

fn set_user_position(e: &Env, user: &Address, position: &UserPosition) {
    let balance_key = DataKey::UserBalance(user.clone());
    let reward_index_key = DataKey::UserRewardIndex(user.clone());
    let rewards_key = DataKey::UserRewards(user.clone());

    if position.balance == 0 {
        e.storage().persistent().remove(&balance_key);
    } else {
        e.storage()
            .persistent()
            .set(&balance_key, &position.balance);
        bump_persistent_ttl(e, &balance_key);
    }

    if position.reward_index == 0 {
        e.storage().persistent().remove(&reward_index_key);
    } else {
        e.storage()
            .persistent()
            .set(&reward_index_key, &position.reward_index);
        bump_persistent_ttl(e, &reward_index_key);
    }

    if position.rewards == 0 {
        e.storage().persistent().remove(&rewards_key);
    } else {
        e.storage()
            .persistent()
            .set(&rewards_key, &position.rewards);
        bump_persistent_ttl(e, &rewards_key);
    }
}

pub fn get_user_balance(e: &Env, user: &Address) -> Result<i128, VaultError> {
    Ok(get_user_position(e, user)?.balance)
}

pub fn store_deposit(
    e: &Env,
    user: &Address,
    amount: i128,
) -> Result<(VaultState, UserPosition), VaultError> {
    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user);
    accrue_position_rewards(&state, &mut position)?;

    position.balance = position
        .balance
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;
    let next_total = state
        .total_deposits
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;

    set_total_deposits(e, next_total);
    set_user_position(e, user, &position);

    Ok((
        VaultState {
            total_deposits: next_total,
            ..state
        },
        position,
    ))
}

pub fn store_withdraw(
    e: &Env,
    user: &Address,
    amount: i128,
) -> Result<(VaultState, UserPosition), VaultError> {
    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user);
    accrue_position_rewards(&state, &mut position)?;

    if position.balance < amount {
        return Err(BalanceError::InsufficientBalance.into());
    }
    if state.total_deposits < amount {
        return Err(StateError::InvalidState.into());
    }

    position.balance = position
        .balance
        .checked_sub(amount)
        .ok_or(ArithmeticError::Overflow)?;
    let next_total = state
        .total_deposits
        .checked_sub(amount)
        .ok_or(ArithmeticError::Overflow)?;

    set_total_deposits(e, next_total);
    set_user_position(e, user, &position);

    Ok((
        VaultState {
            total_deposits: next_total,
            ..state
        },
        position,
    ))
}

pub fn store_reward_distribution(e: &Env, amount: i128) -> Result<VaultState, VaultError> {
    let state = get_state(e)?;
    let increment = checked_reward_index_increment(amount, state.total_deposits)?;
    let next_reward_index = state
        .reward_index
        .checked_add(increment)
        .ok_or(ArithmeticError::Overflow)?;

    set_reward_index(e, next_reward_index);

    Ok(VaultState {
        reward_index: next_reward_index,
        ..state
    })
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

    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user);
    accrue_position_rewards(&state, &mut position)?;

    Ok(UserRewardSnapshot {
        reward_index: position.reward_index,
        rewards: position.rewards,
    })
}

pub fn pending_user_rewards_view(e: &Env, user: &Address) -> Result<i128, VaultError> {
    Ok(preview_user_rewards(e, user)?.rewards)
}

pub(crate) fn checked_reward_index_increment(
    amount: i128,
    total_deposits: i128,
) -> Result<i128, VaultError> {
    if total_deposits <= 0 {
        return Err(BalanceError::NoDeposits.into());
    }

    let scaled = amount
        .checked_mul(REWARD_INDEX_SCALE)
        .ok_or(ArithmeticError::Overflow)?;
    let increment = scaled
        .checked_div(total_deposits)
        .ok_or(ArithmeticError::RewardCalculationFailed)?;

    if increment <= 0 {
        return Err(ArithmeticError::ZeroRewardIncrement.into());
    }

    Ok(increment)
}

fn accrue_position_rewards(
    state: &VaultState,
    position: &mut UserPosition,
) -> Result<(), VaultError> {
    if state.reward_index == position.reward_index || position.balance == 0 {
        position.reward_index = state.reward_index;
        return Ok(());
    }

    if position.balance > 0 {
        let delta = state
            .reward_index
            .checked_sub(position.reward_index)
            .ok_or(ArithmeticError::Overflow)?;
        let accrued = checked_accrued_rewards(position.balance, delta)?;

        if accrued > 0 {
            position.rewards = position
                .rewards
                .checked_add(accrued)
                .ok_or(ArithmeticError::Overflow)?;
        }
    }

    position.reward_index = state.reward_index;
    Ok(())
}

fn checked_accrued_rewards(balance: i128, reward_delta: i128) -> Result<i128, VaultError> {
    balance
        .checked_mul(reward_delta)
        .ok_or(ArithmeticError::Overflow)?
        .checked_div(REWARD_INDEX_SCALE)
        .ok_or(ArithmeticError::RewardCalculationFailed.into())
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
