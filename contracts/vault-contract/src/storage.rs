use soroban_sdk::{contracttype, Address, Env};

use crate::errors::{ArithmeticError, AuthorizationError, BalanceError, StateError, VaultError};

pub const PRECISION_FACTOR: i128 = 1_000_000_000;

const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

const PERSISTENT_TTL_THRESHOLD: u32 = 518_400;
const PERSISTENT_TTL_EXTEND_TO: u32 = 518_400;

/// Keys used to store data in the contract's storage.
/// Soroban supports three types of storage:
/// 1. Temporary: Cheap, but can be deleted if not extended.
/// 2. Instance: Tied to the contract instance, shared by all users.
/// 3. Persistent: Long-term storage for user-specific data.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Flag indicating if the contract has been initialized.
    /// Stores a boolean value.
    Initialized(bool),
    /// The global state of the vault.
    /// Stores a `VaultState` struct.
    State,
    /// User-specific position data (balance, rewards, etc.).
    /// Stores a `UserPosition` struct for a specific `Address`.
    UserPosition(Address),
    /// A guard to prevent reentrant calls to sensitive functions.
    /// Stores a boolean value.
    ReentrancyGuard,
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
    ReentrancyGuard,
    IsPaused,
}

/// The global state of the vault contract.
/// This struct is stored in instance storage and updated frequently.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VaultState {
    /// The address allowed to perform administrative actions like reward distribution.
    pub admin: Address,
    /// The address of the token that users deposit into the vault.
    pub deposit_token: Address,
    /// The address of the token distributed as rewards.
    pub reward_token: Address,
    /// The total amount of deposit tokens currently held by the vault.
    pub total_deposits: i128,
    /// The global reward index that tracks cumulative rewards per unit of deposit.
    /// It increases whenever `distribute_rewards` is called.
    pub reward_index: i128,
}

/// Snapshot of a user's position in the vault.
/// This is stored in persistent storage for each user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserPosition {
    /// The amount of deposit tokens the user has currently staked.
    pub balance: i128,
    /// The value of the global reward index at the time of the user's last interaction.
    /// Used to calculate newly accrued rewards since that interaction.
    pub reward_index: i128,
    /// The amount of rewards the user has earned but not yet claimed.
    pub rewards: i128,
}

/// A helper struct for returning reward information in view functions.
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
    /// The current reward index applied to the snapshot.
    pub reward_index: i128,
    /// The total rewards (accrued + pending) for the user.
    pub rewards: i128,
}

/// Checks if the contract has been initialized.
///
/// Returns `true` if initialized, `false` otherwise.
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
// ---------------------------------------------------------------------------
// Init
// ---------------------------------------------------------------------------

pub fn is_initialized(e: &Env) -> bool {
    e.storage()
        .instance()
        .get::<_, bool>(&DataKey::Initialized(true))
        .unwrap_or(false)
}

/// Ensures the contract is initialized, returning an error if not.
///
/// # Errors
/// * `VaultError::NotInitialized` - If the contract has not been initialized.
pub fn require_initialized(e: &Env) -> Result<(), VaultError> {
    if is_initialized(e) {
        Ok(())
    } else {
        Err(StateError::NotInitialized.into())
    }
}

/// Initializes the contract state and sets the initialization flag.
///
/// This should only be called by the `initialize` function in `lib.rs`.
pub fn initialize_state(e: &Env, admin: &Address, deposit_token: &Address, reward_token: &Address) {
    let state = VaultState {
        admin: admin.clone(),
        deposit_token: deposit_token.clone(),
        reward_token: reward_token.clone(),
        total_deposits: 0,
        reward_index: 0,
    };
    e.storage().instance().set(&DataKey::State, &state);
    e.storage().instance().set(&DataKey::Initialized(true), &true);
    bump_instance_ttl(e);
}

/// Retrieves the global vault state.
///
/// # Errors
/// * `VaultError::NotInitialized` - If the state is not found.
pub fn get_state(e: &Env) -> Result<VaultState, VaultError> {
    require_initialized(e)?;
    let state: VaultState = e
        bump_instance_ttl(e);
        Ok(())
    } else {
        Err(StateError::NotInitialized.into())
    }
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
    e.storage().instance().set(&DataKey::ReentrancyGuard, &false);
    bump_instance_ttl(e);
}

pub fn initialize_state(e: &Env, admin: &Address, deposit_token: &Address, reward_token: &Address) {
    e.storage().instance().set(&DataKey::Initialized, &true);
    e.storage().instance().set(&DataKey::Admin, admin);
    e.storage().instance().set(&DataKey::DepositToken, deposit_token);
    e.storage().instance().set(&DataKey::RewardToken, reward_token);
    e.storage().instance().set(&DataKey::TotalDeposits, &0_i128);
    e.storage().instance().set(&DataKey::RewardIndex, &0_i128);
    e.storage().instance().set(&DataKey::ReentrancyGuard, &false);
    bump_instance_ttl(e);
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

pub fn get_state(e: &Env) -> Result<VaultState, VaultError> {
    require_initialized(e)?;
    let state = VaultState {
        admin: e.storage().instance().get(&DataKey::Admin).ok_or(StateError::NotInitialized)?,
        deposit_token: e.storage().instance().get(&DataKey::DepositToken).ok_or(StateError::NotInitialized)?,
        reward_token: e.storage().instance().get(&DataKey::RewardToken).ok_or(StateError::NotInitialized)?,
        total_deposits: e.storage().instance().get(&DataKey::TotalDeposits).unwrap_or(0_i128),
        reward_index: e.storage().instance().get(&DataKey::RewardIndex).unwrap_or(0_i128),
    };
    let admin = e
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(StateError::InvalidState)?;
    bump_instance_ttl(e);
    Ok(admin)
}

/// Updates the global vault state.
pub fn set_state(e: &Env, state: &VaultState) {
    e.storage().instance().set(&DataKey::State, state);
    e.storage().instance().set(&DataKey::TotalDeposits, &state.total_deposits);
    e.storage().instance().set(&DataKey::RewardIndex, &state.reward_index);
// ---------------------------------------------------------------------------
// State (global)
// ---------------------------------------------------------------------------

pub fn get_pending_admin(e: &Env) -> Result<Option<Address>, VaultError> {
    require_initialized(e)?;
    let pending = e.storage().instance().get(&DataKey::PendingAdmin);
    bump_instance_ttl(e);
    Ok(pending)
}

/// Retrieves the admin address from the vault state.
pub fn get_admin(e: &Env) -> Result<Address, VaultError> {
    Ok(get_state(e)?.admin)
pub fn set_pending_admin(e: &Env, pending_admin: &Address) {
    e.storage()
        .instance()
        .set(&DataKey::PendingAdmin, pending_admin);
    bump_instance_ttl(e);
}

/// Retrieves the deposit token address from the vault state.
pub fn get_deposit_token(e: &Env) -> Result<Address, VaultError> {
    Ok(get_state(e)?.deposit_token)
pub fn clear_pending_admin(e: &Env) {
    e.storage().instance().remove(&DataKey::PendingAdmin);
    bump_instance_ttl(e);
}

pub fn get_deposit_token(e: &Env) -> Result<Address, VaultError> {
    Ok(get_state(e)?.deposit_token)
    require_initialized(e)?;
    let token = e
        .storage()
        .instance()
        .get(&DataKey::DepositToken)
        .ok_or(StateError::InvalidState)?;
    bump_instance_ttl(e);
    Ok(token)
}

/// Retrieves the reward token address from the vault state.
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

/// Retrieves the total amount of deposits from the vault state.
pub fn get_total_deposits(e: &Env) -> Result<i128, VaultError> {
    Ok(get_state(e)?.total_deposits)
}

/// Retrieves the current reward index from the vault state.
pub fn get_reward_index(e: &Env) -> Result<i128, VaultError> {
    Ok(get_state(e)?.reward_index)
}

/// Retrieves a user's position data from persistent storage.
///
/// If no position exists, returns a default position with 0 balance
/// and the current global reward index.
pub fn get_user_position(e: &Env, user: &Address) -> Result<UserPosition, VaultError> {
    require_initialized(e)?;
    let key = DataKey::UserPosition(user.clone());
    let position = e
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or(UserPosition {
            balance: 0,
            reward_index: get_reward_index(e)?,
            rewards: 0,
        });
    bump_persistent_ttl(e, &key);
    Ok(position)
}

/// Retrieves a user's position, ignoring errors (useful for internal calculations).
pub fn get_user_position_unchecked(e: &Env, user: &Address) -> UserPosition {
    get_user_position(e, user).unwrap_or(UserPosition {
        balance: 0,
        reward_index: get_reward_index(e).unwrap_or(0),
        rewards: 0,
    })
}

/// Persists a user's position data to persistent storage.
pub fn set_user_position(e: &Env, user: &Address, position: &UserPosition) {
    let key = DataKey::UserPosition(user.clone());
    e.storage().persistent().set(&key, position);
    bump_persistent_ttl(e, &key);
}

/// Retrieves just the balance part of a user's position.
pub fn get_user_balance(e: &Env, user: &Address) -> Result<i128, VaultError> {
    Ok(get_user_position(e, user)?.balance)
pub fn get_reward_index(e: &Env) -> Result<i128, VaultError> {
    Ok(get_state(e)?.reward_index)
}

pub fn get_user_balance_and_extend(e: &Env, user: &Address) -> i128 {
    let key = DataKey::UserBalance(user.clone());
    if let Some(bal) = e.storage().persistent().get(&key) {
        bump_persistent_ttl(e, &key);
        bal
    } else {
        0_i128
    }
}

pub fn set_user_balance_and_extend(e: &Env, user: &Address, balance: i128) {
    let key = DataKey::UserBalance(user.clone());
    if balance == 0 {
        e.storage().persistent().remove(&key);
    } else {
        e.storage().persistent().set(&key, &balance);
        bump_persistent_ttl(e, &key);
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

pub fn get_user_reward_index_and_extend(e: &Env, user: &Address) -> i128 {
    let key = DataKey::UserRewardIndex(user.clone());
    if let Some(idx) = e.storage().persistent().get(&key) {
        bump_persistent_ttl(e, &key);
        idx
    } else {
        0_i128
    }
}

pub fn set_user_reward_index_and_extend(e: &Env, user: &Address, index: i128) {
    let key = DataKey::UserRewardIndex(user.clone());
    e.storage().persistent().set(&key, &index);
    bump_persistent_ttl(e, &key);
}

pub fn get_user_rewards_and_extend(e: &Env, user: &Address) -> i128 {
    let key = DataKey::UserRewards(user.clone());
    if let Some(rewards) = e.storage().persistent().get(&key) {
        bump_persistent_ttl(e, &key);
        rewards
    } else {
        0_i128
    }
}

pub fn set_user_rewards_and_extend(e: &Env, user: &Address, rewards: i128) {
    let key = DataKey::UserRewards(user.clone());
    e.storage().persistent().set(&key, &rewards);
    bump_persistent_ttl(e, &key);
}

/// Sets the reentrancy guard flag to prevent recursive calls.
///
/// # Errors
/// * `VaultError::ReentrancyDetected` - If the guard is already set.
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

/// Clears the reentrancy guard flag.
pub fn exit_non_reentrant(e: &Env) {
    e.storage().instance().set(&DataKey::ReentrancyGuard, &false);
pub fn get_user_position_unchecked(e: &Env, user: &Address) -> UserPosition {
    UserPosition {
        balance: get_user_balance_and_extend(e, user),
        reward_index: get_user_reward_index_and_extend(e, user),
        rewards: get_user_rewards_and_extend(e, user),
    }
}

pub fn get_user_position(e: &Env, user: &Address) -> Result<UserPosition, VaultError> {
    require_initialized(e)?;
    Ok(get_user_position_unchecked(e, user))
}

pub fn set_user_position(e: &Env, user: &Address, position: &UserPosition) {
    set_user_balance_and_extend(e, user, position.balance);
    set_user_reward_index_and_extend(e, user, position.reward_index);
    set_user_rewards_and_extend(e, user, position.rewards);
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

/// Logic for recording a deposit in the contract state.
///
/// This function updates the user's balance and the global total deposits.
/// It also triggers reward accrual for the user based on their previous balance.
pub fn store_deposit(
    e: &Env,
    user: &Address,
    amount: i128,
) -> Result<(VaultState, UserPosition), VaultError> {
    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user);
    
    // Accrue rewards earned up to this point using the old balance.
    accrue_position_rewards(&state, &mut position)?;

    // Update balance and total deposits.
    position.balance = position
        .balance
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;
    let next_total = state
        .total_deposits
        .checked_add(amount)
        .ok_or(ArithmeticError::Overflow)?;

    // Persist changes.
    set_state(e, &state);
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

/// Logic for recording a withdrawal in the contract state.
///
/// Ensures the user has enough balance and updates both user and global state.
pub fn store_withdraw(
    e: &Env,
    user: &Address,
    amount: i128,
) -> Result<(VaultState, UserPosition), VaultError> {
    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user);
    
    // Accrue rewards earned up to this point using the old balance.
    accrue_position_rewards(&state, &mut position)?;

    if position.balance < amount {
        return Err(BalanceError::InsufficientBalance.into());
    }
    
    // Update balance and total deposits.
    position.balance = position
        .balance
        .checked_sub(amount)
        .ok_or(VaultError::MathOverflow)?;
    state.total_deposits = state
        .total_deposits
        .checked_sub(amount)
        .ok_or(VaultError::MathOverflow)?;

    // Persist changes.
    set_state(e, &state);
    set_user_position(e, user, &position);
    Ok((state, position))
    if state.total_deposits < amount {
        return Err(StateError::InvalidState.into());
    }

    position.balance = position.balance.checked_sub(amount).unwrap();
    state.total_deposits = state.total_deposits.checked_sub(amount).unwrap();

    set_state(e, &state);
    set_user_position(e, user, &position);
    Ok((state, position))
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

/// Logic for distributing rewards and updating the global reward index.
///
/// Calculates the index increment based on the distributed amount and total deposits.
pub fn store_reward_distribution(e: &Env, amount: i128) -> Result<VaultState, VaultError> {
    let state = get_state(e)?;
    let increment = checked_reward_index_increment(amount, state.total_deposits)?;

    // Increment the global index. All subsequent interactions will use this new index.
    state.reward_index = state
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

/// Logic for claiming accrued rewards for a user.
///
/// Accrues all pending rewards and resets the user's rewards counter to zero.
pub fn store_claimable_rewards(e: &Env, user: &Address) -> Result<i128, VaultError> {
    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user);
    
    // Accrue all rewards earned up to the current global index.
    accrue_position_rewards(&state, &mut position)?;

    let claimable = position.rewards;
    position.rewards = 0;
    
    // Reset the counter but keep the balance and current reward index.
    set_user_position(e, user, &position);

    Ok(claimable)
}

/// Provides a read-only view of a user's pending rewards without modifying state.
pub fn preview_user_rewards(e: &Env, user: &Address) -> Result<UserRewardSnapshot, VaultError> {
    require_initialized(e)?;

    if !is_initialized(e) {
        return Err(StateError::NotInitialized.into());
    }

// ---------------------------------------------------------------------------
// Read-only reward preview
// ---------------------------------------------------------------------------

pub fn preview_user_rewards(e: &Env, user: &Address) -> Result<UserRewardSnapshot, VaultError> {
    require_initialized(e)?;
    let state = get_state(e)?;
    let position = get_user_position_unchecked(e, user);

    // If global index hasn't moved or user has no balance, no new rewards.
    if state.reward_index == position.reward_index || position.balance == 0 {
        return Ok(UserRewardSnapshot {
            reward_index: state.reward_index,
            rewards: position.rewards,
        });
    }

    // Calculate the hypothetical accrual.
    let delta = state
        .reward_index
        .checked_sub(position.reward_index)
        .ok_or(VaultError::MathOverflow)?;
    let accrued = checked_accrued_rewards(position.balance, delta)?;
    let rewards = position
        .rewards
        .checked_add(accrued)
        .ok_or(VaultError::MathOverflow)?;
    let state = get_state(e)?;
    let mut position = get_user_position_unchecked(e, user);
    accrue_position_rewards(&state, &mut position)?;

    Ok(UserRewardSnapshot {
        reward_index: position.reward_index,
        rewards: position.rewards,
    })
}

/// View function for pending rewards.
pub fn pending_user_rewards_view(e: &Env, user: &Address) -> Result<i128, VaultError> {
    Ok(preview_user_rewards(e, user)?.rewards)
}

/// Internal helper to calculate how much the reward index should increase.
///
/// Formula: `(amount * SCALE) / total_deposits`
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

/// Internal helper to accrue rewards for a specific user position.
///
/// Updates `position.rewards` and syncs `position.reward_index` with `state.reward_index`.
pub(crate) fn checked_accrued_rewards(balance: i128, delta: i128) -> Result<i128, VaultError> {
    let scaled = balance
        .checked_mul(delta)
        .ok_or(VaultError::MathOverflow)?;
    Ok(scaled / REWARD_INDEX_SCALE)
}

pub fn pending_user_rewards_view(e: &Env, user: &Address) -> Result<i128, VaultError> {
    Ok(preview_user_rewards(e, user)?.rewards)
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

    // Mark that the user is now synced with the current global reward index.
    position.reward_index = state.reward_index;
    Ok(())
}

/// Internal helper to calculate rewards based on balance and index delta.
///
/// Formula: `(balance * index_delta) / SCALE`
fn checked_accrued_rewards(balance: i128, index_delta: i128) -> Result<i128, VaultError> {
    balance
        .checked_mul(index_delta)
        .and_then(|v| v.checked_div(REWARD_INDEX_SCALE))
        .ok_or(VaultError::MathOverflow)
fn checked_accrued_rewards(balance: i128, reward_delta: i128) -> Result<i128, VaultError> {
    balance
        .checked_mul(reward_delta)
        .ok_or(ArithmeticError::Overflow)?
        .checked_div(REWARD_INDEX_SCALE)
        .ok_or(ArithmeticError::RewardCalculationFailed.into())
}

/// Bumps the TTL for instance storage to keep the contract active.
fn bump_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

/// Bumps the TTL for a specific persistent storage key.
fn bump_persistent_ttl(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}
