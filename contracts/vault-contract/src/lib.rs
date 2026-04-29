#![no_std]

pub mod errors;
mod events;
mod storage;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use crate::errors::{ArithmeticError, BalanceError, StateError, ValidationError, VaultError};

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn version() -> u32 {
        1
    }

    pub fn initialize(
        e: Env,
        admin: Address,
        deposit_token: Address,
        reward_token: Address,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        if storage::is_initialized(&e) {
            return Err(StateError::AlreadyInitialized.into());
        }

        validate_distinct_token_addresses(&deposit_token, &reward_token)?;
        admin.require_auth();

        storage::initialize_state(&e, &admin, &deposit_token, &reward_token);
        events::emit_initialize(&e, admin, deposit_token, reward_token);

        Ok(())
    }

    /// Deposits tokens into the vault and accrues pending rewards before updating balance.
    /// This ensures users receive rewards based on their old balance up to this point.
    pub fn deposit(e: Env, from: Address, amount: i128) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        from.require_auth();

        with_non_reentrant(&e, || {
            let state = storage::get_state(&e)?;
            let (_, position) = storage::store_deposit(&e, &from, amount)?;
            let token = soroban_sdk::token::Client::new(&e, &state.deposit_token);
            token.transfer(&from, &e.current_contract_address(), &amount);
            events::emit_deposit(&e, from.clone(), amount, position.balance);
            Ok(())
        })
    }

    /// Withdraws tokens from the vault and accrues pending rewards before updating balance.
    /// This function is isolated from reward claiming - it only handles the deposit token.
    /// If the reward token contract fails, users can still withdraw their deposits.
    pub fn withdraw(e: Env, to: Address, amount: i128) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        to.require_auth();

        with_non_reentrant(&e, || {
            let (state, position) = storage::store_withdraw(&e, &to, amount)?;
            
            events::emit_withdraw(&e, to.clone(), amount, position.balance);

            let token = soroban_sdk::token::Client::new(&e, &state.deposit_token);
            // Adhering to Check-Effects-Interactions pattern.
            token.transfer(&e.current_contract_address(), &to, &amount);
            events::emit_withdraw(&e, to, amount, position.balance);
            Ok(())
        })
    }

/// Distributes rewards to all depositors by updating the global reward index.
/// Does not immediately transfer rewards to users - they accrue lazily.
/// 
/// Security: Only admin can call this function.
/// Minimum amount: 100,000 stroops to prevent dust spam attacks.
pub fn distribute_rewards(e: Env, amount: i128) -> Result<i128, VaultError> {
    storage::require_initialized(&e)?;
    validate_positive_amount(amount)?;

    // Prevent dust spam attacks by enforcing minimum amount
    const MIN_REWARD_DISTRIBUTION: i128 = 100_000;
    if amount < MIN_REWARD_DISTRIBUTION {
        return Err(ValidationError::InsufficientRewardAmount.into());
    }

    let state = storage::get_state(&e)?;
    let admin = state.admin.clone();
    let reward_token_id = state.reward_token.clone();

    admin.require_auth();

    with_non_reentrant(&e, || {
        let next_index = storage::store_reward_distribution(&e, amount)?.reward_index;
        let reward_token = soroban_sdk::token::Client::new(&e, &reward_token_id);
        reward_token.transfer(&admin, &e.current_contract_address(), &amount);
        events::emit_distribute(&e, admin.clone(), amount, next_index);
        Ok(next_index)
    })
}

    /// Claims accrued rewards for a user.
    /// Isolated from withdraw to ensure exit liquidity is always available.
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

            events::emit_claim(&e, user, amt);
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

    pub fn pause_contract(e: Env, admin: Address) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let current_admin = storage::get_admin(&e)?;
        if current_admin != admin {
            return Err(VaultError::Unauthorized);
        }
        admin.require_auth();
        storage::set_paused(&e, true);
        Ok(())
    }

    pub fn unpause_contract(e: Env, admin: Address) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let current_admin = storage::get_admin(&e)?;
        if current_admin != admin {
            return Err(VaultError::Unauthorized);
        }
        admin.require_auth();
        storage::set_paused(&e, false);
    /// Upgrades the contract WASM to a new version.
    /// Only the admin can perform this action.
    /// The new WASM hash must reference a valid, already-uploaded WASM that
    /// is compatible with the current storage layout.
    pub fn upgrade(e: Env, admin: Address, new_wasm_hash: BytesN<32>) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        admin.require_auth();

        let stored_admin = storage::get_admin(&e)?;
        if admin != stored_admin {
            return Err(VaultError::UpgradeFailed);
        }

        e.deployer().update_current_contract_wasm(new_wasm_hash.clone());
        events::emit_upgrade(&e, admin, new_wasm_hash);

        Ok(())
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

fn ensure_contract_balance(balance: i128, requested_amount: i128) -> Result<(), VaultError> {
    if balance < requested_amount {
        return Err(BalanceError::InsufficientContractBalance.into());
    }
    Ok(())
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

// ---------------------------------------------------------------------------
// Precision math unit tests  (issue #81)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod precision_tests {
    use super::storage::{checked_accrued_rewards, checked_reward_index_increment, PRECISION_FACTOR};
    use super::errors::VaultError;

    #[test]
    fn increment_basic() {
        // 400 rewards / 400 total => index += 1 * PRECISION_FACTOR
        let inc = checked_reward_index_increment(400, 400).unwrap();
        assert_eq!(inc, PRECISION_FACTOR);
    }

    #[test]
    fn increment_small_reward_large_deposits_retains_precision() {
        // 1 reward token, 1_000_000 deposited.
        // Without scaling this would be 0; with PRECISION_FACTOR it is non-zero.
        let inc = checked_reward_index_increment(1, 1_000_000).unwrap();
        assert!(inc > 0, "precision lost: increment rounded to zero");
        assert_eq!(inc, PRECISION_FACTOR / 1_000_000);
    }

    #[test]
    fn increment_rejects_zero_deposits() {
        assert_eq!(
            checked_reward_index_increment(100, 0),
            Err(VaultError::NoDeposits)
        );
    }

    #[test]
    fn increment_rejects_negative_deposits() {
        assert_eq!(
            checked_reward_index_increment(100, -1),
            Err(VaultError::NoDeposits)
        );
    }

    #[test]
    fn accrued_proportional_equal_deposits() {
        // Both users deposited 100 each (200 total), 400 rewards distributed.
        // index increment = (400 * PRECISION_FACTOR) / 200 = 2 * PRECISION_FACTOR
        let delta = checked_reward_index_increment(400, 200).unwrap();
        let reward = checked_accrued_rewards(100, delta).unwrap();
        assert_eq!(reward, 200);
    }

    #[test]
    fn accrued_vastly_different_deposits_user_a_tiny() {
        // User A: 1 token, User B: 1_000_000 tokens. 1_000_001 rewards distributed.
        let total = 1_000_001_i128;
        let rewards = 1_000_001_i128;
        let delta = checked_reward_index_increment(rewards, total).unwrap();

        let reward_a = checked_accrued_rewards(1, delta).unwrap();
        assert_eq!(reward_a, 1);

        let reward_b = checked_accrued_rewards(1_000_000, delta).unwrap();
        assert_eq!(reward_b, 1_000_000);
    }

    #[test]
    fn accrued_zero_balance_returns_zero() {
        let delta = checked_reward_index_increment(1000, 500).unwrap();
        assert_eq!(checked_accrued_rewards(0, delta).unwrap(), 0);
    }

    #[test]
    fn accrued_zero_delta_returns_zero() {
        assert_eq!(checked_accrued_rewards(1_000_000, 0).unwrap(), 0);
    }

    #[test]
    fn precision_factor_value() {
        assert_eq!(PRECISION_FACTOR, 1_000_000_000);
    }

    #[test]
    fn round_trip_proportionality() {
        // Alice: 1 token, Bob: 999_999 tokens. 1_000_000 rewards.
        let total = 1_000_000_i128;
        let rewards = 1_000_000_i128;
        let delta = checked_reward_index_increment(rewards, total).unwrap();

        assert_eq!(checked_accrued_rewards(1, delta).unwrap(), 1);
        assert_eq!(checked_accrued_rewards(999_999, delta).unwrap(), 999_999);
    }
}

// TODO(gas): Consider merging per-user keys (balance/index/rewards) into a single struct to reduce reads.
// TODO(security): Consider adding pausability or per-user deposit caps.
// TODO(governance): Introduce admin handover / multisig patterns.

#[cfg(test)]
mod tests {
    use super::*;
    use axionvera_vault_contract_v2::VaultContractV2Client;
    use soroban_sdk::testutils::Address as _;

    /// Deploys V1, initializes it, sets up state, then upgrades to V2.
    /// Verifies that V2 functions work while maintaining V1 state.
    ///
    /// Prerequisite: Build V2 WASM before running:
    ///   cargo build --target wasm32-unknown-unknown --release -p axionvera-vault-contract-v2
    #[test]
    fn upgrade_v1_to_v2_preserves_state() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let deposit_token = Address::generate(&env);
        let reward_token = Address::generate(&env);
        let user = Address::generate(&env);

        // ── Deploy V1 ──────────────────────────────────────────────
        let contract_id = env.register_contract(None, VaultContract);
        let v1 = VaultContractClient::new(&env, &contract_id);

        // Initialize V1
        v1.initialize(&admin, &deposit_token, &reward_token);

        // Verify V1 version
        assert_eq!(v1.version(), 1);

        // Set up V1 state: write a user balance directly into storage
        // so we can verify it survives the upgrade without needing a
        // token contract for a full deposit flow.
        {
            let key = storage::DataKey::UserBalance(user.clone());
            env.as_contract(&contract_id, || {
                env.storage().persistent().set(&key, &5_000_i128);
            });
        }

        // Verify the balance is readable via V1
        assert_eq!(v1.balance(&user), Ok(5_000));
        assert_eq!(v1.admin(), Ok(admin.clone()));

        // ── Upload V2 WASM ─────────────────────────────────────────
        let v2_wasm_path = std::path::Path::new(
            "../target/wasm32-unknown-unknown/release/axionvera_vault_contract_v2.wasm",
        );
        let v2_wasm_bytes =
            std::fs::read(v2_wasm_path).expect(
                "V2 WASM not found. Build it first:\n  \
                 cargo build --target wasm32-unknown-unknown --release \
                 -p axionvera-vault-contract-v2",
            );
        let v2_hash = env.deployer().upload_contract_wasm(v2_wasm_bytes);

        // ── Upgrade to V2 ─────────────────────────────────────────
        v1.upgrade(&admin, &v2_hash);

        // ── Verify V2 behavior with preserved V1 state ─────────────
        let v2 = VaultContractV2Client::new(&env, &contract_id);

        // V2 reports version 2
        assert_eq!(v2.version(), 2);

        // V1 state is still readable
        assert_eq!(v2.balance(&user), 5_000);
        assert_eq!(v2.admin(), admin);

        // V2-only function works
        assert_eq!(v2.v2_greeting(), soroban_sdk::symbol_short!("hello"));
    }

    /// Verifies that only the stored admin can upgrade the contract.
    #[test]
    fn upgrade_rejects_non_admin() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let imposter = Address::generate(&env);
        let deposit_token = Address::generate(&env);
        let reward_token = Address::generate(&env);

        let contract_id = env.register_contract(None, VaultContract);
        let client = VaultContractClient::new(&env, &contract_id);
        client.initialize(&admin, &deposit_token, &reward_token);

        // Build V2 WASM hash (reuse the same WASM for simplicity)
        let v2_wasm_path = std::path::Path::new(
            "../target/wasm32-unknown-unknown/release/axionvera_vault_contract_v2.wasm",
        );
        let v2_wasm_bytes =
            std::fs::read(v2_wasm_path).expect("V2 WASM not found. Build it first.");
        let v2_hash = env.deployer().upload_contract_wasm(v2_wasm_bytes);

        // Non-admin should be rejected
        let result = client.try_upgrade(&imposter, &v2_hash);
        assert!(result.is_err());
    }

    /// Verifies that upgrade fails on uninitialized contract.
    #[test]
    fn upgrade_fails_on_uninitialized_contract() {
        let env = Env::default();
        env.mock_all_auths();

        let fake_admin = Address::generate(&env);

        let contract_id = env.register_contract(None, VaultContract);
        let client = VaultContractClient::new(&env, &contract_id);

        // Build V2 WASM hash
        let v2_wasm_path = std::path::Path::new(
            "../target/wasm32-unknown-unknown/release/axionvera_vault_contract_v2.wasm",
        );
        let v2_wasm_bytes =
            std::fs::read(v2_wasm_path).expect("V2 WASM not found. Build it first.");
        let v2_hash = env.deployer().upload_contract_wasm(v2_wasm_bytes);

        // Upgrade on uninitialized contract should fail
        let result = client.try_upgrade(&fake_admin, &v2_hash);
        assert!(result.is_err());
    }
}
