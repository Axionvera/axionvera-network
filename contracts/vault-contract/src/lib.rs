#![no_std]

mod errors;
mod events;
mod storage;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::errors::{ArithmeticError, BalanceError, StateError, ValidationError, VaultError};

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn version() -> u32 {
        1
    }

    /// Initializes the vault with the specified admin and token addresses.
    ///
    /// # Security Considerations
    /// This function can only be called once. It checks the initialization state
    /// using a dedicated storage flag. The provided `admin` address must authorize
    /// the call to ensure that the person initializing the contract is indeed
    /// the intended administrator.
    ///
    /// # Arguments
    /// * `e` - The environment.
    /// * `admin` - The address that will have administrative privileges (e.g., distributing rewards).
    /// * `deposit_token` - The address of the token that users will deposit.
    /// * `reward_token` - The address of the token that will be distributed as rewards.
    ///
    /// # Errors
    /// * `VaultError::AlreadyInitialized` - If the vault has already been initialized.
    /// * `VaultError::InvalidTokenConfiguration` - If the deposit and reward tokens are the same.
    pub fn initialize(
        e: Env,
        admin: Address,
        deposit_token: Address,
        reward_token: Address,
    ) -> Result<(), VaultError> {
        // --- STEP 1: INITIALIZATION GUARD ---
        // We must ensure this function is only callable once to prevent an attacker
        // from re-initializing the contract and taking control.
        if storage::is_initialized(&e) {
            return Err(StateError::AlreadyInitialized.into());
        }

        // --- STEP 2: VALIDATION ---
        // Basic sanity checks for the provided addresses.
        validate_distinct_token_addresses(&deposit_token, &reward_token)?;
        
        // --- STEP 3: AUTHENTICATION ---
        // We require the admin to sign this transaction. This prevents front-running
        // by an attacker who might try to initialize the contract with their own address
        // if the deployer doesn't include the initialization in the deployment transaction.
        admin.require_auth();

        // --- STEP 4: STATE INITIALIZATION ---
        // Persist the initial state to the contract's instance storage.
        storage::initialize_state(&e, &admin, &deposit_token, &reward_token);
        
        // --- STEP 5: EVENT EMISSION ---
        // Emit an event for indexers and off-chain monitoring.
        events::emit_initialize(&e, admin, deposit_token, reward_token);

        Ok(())
    }

    /// Deposits tokens into the vault and accrues pending rewards before updating balance.
    ///
    /// This function handles the transfer of tokens from the user to the contract.
    /// It ensures that rewards are accrued for the user's previous balance before
    /// the new deposit increases their stake.
    ///
    /// # Arguments
    /// * `e` - The environment.
    /// * `from` - The address of the user depositing tokens.
    /// * `amount` - The amount of tokens to deposit.
    ///
    /// # Errors
    /// * `VaultError::NotInitialized` - If the vault hasn't been initialized.
    /// * `VaultError::InvalidAmount` - If the amount is zero or negative.
    /// * `VaultError::ReentrancyDetected` - If called recursively.
    pub fn deposit(e: Env, from: Address, amount: i128) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        from.require_auth();

        with_non_reentrant(&e, || {
            let (state, position) = storage::store_deposit(&e, &from, amount)?;
            let token = soroban_sdk::token::Client::new(&e, &state.deposit_token);
            token.transfer(&from, &e.current_contract_address(), &amount);
            events::emit_deposit(&e, from.clone(), amount, position.balance);
            Ok(())
        })
    }

    /// Withdraws tokens from the vault and accrues pending rewards before updating balance.
    ///
    /// This function is isolated from reward claiming - it only handles the deposit token.
    /// This "separation of concerns" ensures that even if the reward token contract is
    /// malfunctioning, users can still recover their initial deposits.
    ///
    /// # Arguments
    /// * `e` - The environment.
    /// * `to` - The address of the user withdrawing tokens.
    /// * `amount` - The amount of tokens to withdraw.
    pub fn withdraw(e: Env, to: Address, amount: i128) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        to.require_auth();

        with_non_reentrant(&e, || {
            let (state, position) = storage::store_withdraw(&e, &to, amount)?;
            let token = soroban_sdk::token::Client::new(&e, &state.deposit_token);
            token.transfer(&e.current_contract_address(), &to, &amount);

            events::emit_withdraw(&e, to, amount, position.balance);
            Ok(())
        })
    }

    /// Distributes rewards to all depositors by updating the global reward index.
    ///
    /// This is an administrative function that increases the cumulative rewards
    /// per unit of deposit. It does not immediately transfer tokens to users;
    /// instead, users accrue rewards lazily whenever they interact with the vault.
    ///
    /// # Arguments
    /// * `e` - The environment.
    /// * `amount` - The total amount of reward tokens to distribute.
    pub fn distribute_rewards(e: Env, amount: i128) -> Result<i128, VaultError> {
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;

        let state = storage::get_state(&e)?;
        let admin = state.admin.clone();
        let reward_token_id = state.reward_token.clone();

        admin.require_auth();

        with_non_reentrant(&e, || {
            let next_index = storage::store_reward_distribution(&e, amount)?.reward_index;
            let reward_token = soroban_sdk::token::Client::new(&e, &reward_token_id);
            reward_token.transfer(&admin, &e.current_contract_address(), &amount);
            events::emit_distribute_rewards(&e, amount, next_index);
            Ok(next_index)
        })
    }

    /// Claims all accrued rewards for the calling user.
    ///
    /// This function calculates all pending rewards since the user's last interaction
    /// and transfers them from the contract's balance to the user.
    ///
    /// # Arguments
    /// * `e` - The environment.
    /// * `user` - The address of the user claiming rewards.
    pub fn claim_rewards(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::require_initialized(&e)?;
        user.require_auth();

        with_non_reentrant(&e, || {
            let amt = storage::store_claimable_rewards(&e, &user)?;
            if amt <= 0 {
                return Ok(0);
            }

            let reward_token_id = storage::get_reward_token(&e)?;
            let reward_token = soroban_sdk::token::Client::new(&e, &reward_token_id);
            ensure_contract_balance(reward_token.balance(&e.current_contract_address()), amt)?;
            reward_token.transfer(&e.current_contract_address(), &user, &amt);

            events::emit_claim_rewards(&e, user, amt);
            Ok(amt)
        })
    }

    pub fn balance(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::get_user_balance(&e, &user)
    }

    pub fn total_deposits(e: Env) -> Result<i128, VaultError> {
        storage::get_total_deposits(&e)
    }

    pub fn reward_index(e: Env) -> Result<i128, VaultError> {
        storage::get_reward_index(&e)
    }

    pub fn pending_rewards(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::pending_user_rewards_view(&e, &user)
    }

    pub fn admin(e: Env) -> Result<Address, VaultError> {
        storage::get_admin(&e)
    }

    pub fn deposit_token(e: Env) -> Result<Address, VaultError> {
        storage::get_deposit_token(&e)
    }

    pub fn reward_token(e: Env) -> Result<Address, VaultError> {
        storage::get_reward_token(&e)
    }
}

fn validate_positive_amount(amount: i128) -> Result<(), VaultError> {
    if amount < 0 {
        return Err(ValidationError::NegativeAmount.into());
    }
    if amount == 0 {
        return Err(ValidationError::InvalidAmount.into());
    }
    Ok(())
}

fn validate_distinct_token_addresses(
    deposit_token: &Address,
    reward_token: &Address,
) -> Result<(), VaultError> {
    if deposit_token == reward_token {
        return Err(ValidationError::InvalidTokenConfiguration.into());
    }

    Ok(())
}

fn ensure_balance(balance: i128, requested_amount: i128) -> Result<(), VaultError> {
    if balance < requested_amount {
        return Err(BalanceError::InsufficientBalance.into());
    }

    Ok(())
}

fn ensure_contract_balance(balance: i128, requested_amount: i128) -> Result<(), VaultError> {
    if balance < requested_amount {
        return Err(BalanceError::InsufficientContractBalance.into());
    }

    Ok(())
}

fn overflow() -> VaultError {
    ArithmeticError::Overflow.into()
}

fn with_non_reentrant<T, F>(e: &Env, f: F) -> Result<T, VaultError>
where
    F: FnOnce() -> Result<T, VaultError>,
{
    storage::enter_non_reentrant(e)?;
    let result = f();
    storage::exit_non_reentrant(e);
    result
}

// TODO(reward-optimization): Consider a higher precision / rounding strategy for small totals.
// TODO(gas): Consider merging per-user keys (balance/index/rewards) into a single struct to reduce reads.
// TODO(security): Consider adding pausability or per-user deposit caps.
// TODO(governance): Introduce admin handover / multisig patterns.
// TODO(upgradeability): Evaluate upgrade patterns compatible with Soroban best practices.
