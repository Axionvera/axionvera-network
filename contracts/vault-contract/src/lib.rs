#![no_std]

mod errors;
mod events;
mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::errors::{BalanceError, StateError, ValidationError, VaultError};

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
        if storage::is_initialized(&e) {
            return Err(StateError::AlreadyInitialized.into());
        }

        validate_distinct_token_addresses(&deposit_token, &reward_token)?;
        admin.require_auth();

        storage::initialize_state(&e, &admin, &deposit_token, &reward_token);
        events::emit_initialize(&e, admin, deposit_token, reward_token);

        Ok(())
    }

    pub fn deposit(e: Env, from: Address, amount: i128) -> Result<(), VaultError> {
        validate_positive_amount(amount)?;
        from.require_auth();

        let token_id = storage::get_deposit_token(&e)?;
        let token = soroban_sdk::token::Client::new(&e, &token_id);
        ensure_balance(token.balance(&from), amount)?;

        with_non_reentrant(&e, || {
            let (_, position) = storage::store_deposit(&e, &from, amount)?;
            let token = soroban_sdk::token::Client::new(&e, &token_id);
            token.transfer(&from, &e.current_contract_address(), &amount);
            events::emit_deposit(&e, from.clone(), amount, position.balance);
            Ok(())
        })
    }

    pub fn withdraw(e: Env, to: Address, amount: i128) -> Result<(), VaultError> {
        validate_positive_amount(amount)?;
        to.require_auth();

        ensure_balance(storage::get_user_balance(&e, &to)?, amount)?;

        let token_id = storage::get_deposit_token(&e)?;
        let token = soroban_sdk::token::Client::new(&e, &token_id);
        ensure_contract_balance(token.balance(&e.current_contract_address()), amount)?;

        with_non_reentrant(&e, || {
            let (_, position) = storage::store_withdraw(&e, &to, amount)?;
            let token = soroban_sdk::token::Client::new(&e, &token_id);
            token.transfer(&e.current_contract_address(), &to, &amount);
            events::emit_withdraw(&e, to.clone(), amount, position.balance);
            Ok(())
        })
    }

    pub fn distribute_rewards(e: Env, amount: i128) -> Result<i128, VaultError> {
        validate_positive_amount(amount)?;

        let state = storage::get_state(&e)?;
        let admin = state.admin.clone();
        let reward_token_id = state.reward_token.clone();

        admin.require_auth();

        let reward_token = soroban_sdk::token::Client::new(&e, &reward_token_id);
        ensure_balance(reward_token.balance(&admin), amount)?;

        with_non_reentrant(&e, || {
            let next_index = storage::store_reward_distribution(&e, amount)?.reward_index;
            let reward_token = soroban_sdk::token::Client::new(&e, &reward_token_id);
            reward_token.transfer(&admin, &e.current_contract_address(), &amount);
            events::emit_distribute(&e, admin.clone(), amount, next_index);
            Ok(next_index)
        })
    }

    pub fn claim_rewards(e: Env, user: Address) -> Result<i128, VaultError> {
        user.require_auth();

        let claimable = storage::pending_user_rewards_view(&e, &user)?;
        if claimable <= 0 {
            return Ok(0);
        }

        let reward_token_id = storage::get_reward_token(&e)?;
        let reward_token = soroban_sdk::token::Client::new(&e, &reward_token_id);
        ensure_contract_balance(
            reward_token.balance(&e.current_contract_address()),
            claimable,
        )?;

        with_non_reentrant(&e, || {
            let amount = storage::store_claimable_rewards(&e, &user)?;
            if amount <= 0 {
                return Ok(0);
            }

            let reward_token = soroban_sdk::token::Client::new(&e, &reward_token_id);
            reward_token.transfer(&e.current_contract_address(), &user, &amount);
            events::emit_claim(&e, user.clone(), amount);
            Ok(amount)
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

fn with_non_reentrant<T, F>(e: &Env, f: F) -> Result<T, VaultError>
where
    F: FnOnce() -> Result<T, VaultError>,
{
    storage::enter_non_reentrant(e)?;
    let result = f();
    storage::exit_non_reentrant(e);
    result
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::storage::{checked_reward_index_increment, REWARD_INDEX_SCALE};
    use proptest::prelude::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::token::StellarAssetClient;

    fn setup_vault(
        e: &Env,
    ) -> (
        Address,
        Address,
        Address,
        Address,
        Address,
        VaultContractClient<'_>,
    ) {
        e.mock_all_auths();

        let admin = Address::generate(e);
        let user = Address::generate(e);
        let deposit_token_id = e.register_stellar_asset_contract_v2(admin.clone()).address();
        let reward_token_id = e.register_stellar_asset_contract_v2(admin.clone()).address();
        let vault_id = e.register(VaultContract, ());
        let vault = VaultContractClient::new(e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        (admin, user, deposit_token_id, reward_token_id, vault_id, vault)
    }

    #[test]
    fn rewards_are_proportional_and_claimable() {
        let e = Env::default();
        let (admin, user_a, deposit_token_id, reward_token_id, _vault_id, vault) =
            setup_vault(&e);
        let user_b = Address::generate(&e);

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        let reward_token = StellarAssetClient::new(&e, &reward_token_id);

        deposit_token.mint(&user_a, &1_000);
        deposit_token.mint(&user_b, &1_000);
        reward_token.mint(&admin, &600);

        vault.deposit(&user_a, &100);
        vault.deposit(&user_b, &300);

        let next_index = vault.distribute_rewards(&400);
        assert_eq!(next_index, REWARD_INDEX_SCALE);
        assert_eq!(vault.pending_rewards(&user_a), 100);
        assert_eq!(vault.pending_rewards(&user_b), 300);
        assert_eq!(vault.claim_rewards(&user_a), 100);
        assert_eq!(vault.claim_rewards(&user_b), 300);
    }

    #[test]
    fn distribute_rewards_extreme_values_fail_gracefully() {
        let e = Env::default();
        let admin = Address::generate(&e);
        let deposit_token_id = e.register_stellar_asset_contract_v2(admin.clone()).address();
        let reward_token_id = e.register_stellar_asset_contract_v2(admin.clone()).address();
        let vault_id = e.register(VaultContract, ());

        e.as_contract(&vault_id, || {
            storage::initialize_state(&e, &admin, &deposit_token_id, &reward_token_id);

            let mut state = storage::get_state(&e).unwrap();
            state.total_deposits = 1;
            storage::set_state(&e, &state);

            let result = storage::store_reward_distribution(&e, i128::MAX);
            assert_eq!(result, Err(VaultError::MathOverflow));
            assert_eq!(storage::get_reward_index(&e).unwrap(), 0);
        });
    }

    fn rewards_strategy() -> impl Strategy<Value = i128> {
        prop_oneof![
            Just(1_i128),
            Just(REWARD_INDEX_SCALE),
            Just(i128::MAX - 1),
            Just(i128::MAX),
            (1_i128..=i128::MAX),
        ]
    }

    fn deposits_strategy() -> impl Strategy<Value = i128> {
        prop_oneof![Just(1_i128), Just(2_i128), Just(3_i128), 1_i128..=1_000_000_i128,]
    }

    proptest! {
        #[test]
        fn reward_index_math_matches_checked_arithmetic(
            total_deposits in deposits_strategy(),
            rewards in rewards_strategy(),
        ) {
            let result = checked_reward_index_increment(rewards, total_deposits);

            match rewards.checked_mul(REWARD_INDEX_SCALE) {
                None => prop_assert_eq!(result, Err(VaultError::MathOverflow)),
                Some(scaled) => {
                    let expected = scaled.checked_div(total_deposits).unwrap();

                    if expected <= 0 {
                        prop_assert_eq!(result, Err(VaultError::ZeroRewardIncrement));
                    } else {
                        prop_assert_eq!(result, Ok(expected));
                    }
                }
            }
        }
    }

    #[test]
    fn pending_rewards_view_does_not_mutate_state_after_failed_overflow_path() {
        let e = Env::default();
        let admin = Address::generate(&e);
        let user = Address::generate(&e);
        let deposit_token_id = e.register_stellar_asset_contract_v2(admin.clone()).address();
        let reward_token_id = e.register_stellar_asset_contract_v2(admin.clone()).address();
        let vault_id = e.register(VaultContract, ());

        e.as_contract(&vault_id, || {
            storage::initialize_state(&e, &admin, &deposit_token_id, &reward_token_id);
            let _ = storage::store_deposit(&e, &user, 1);

            let err = storage::store_reward_distribution(&e, i128::MAX).unwrap_err();
            assert_eq!(err, VaultError::MathOverflow);
            assert_eq!(storage::pending_user_rewards_view(&e, &user).unwrap(), 0);
            assert_eq!(storage::get_reward_index(&e).unwrap(), 0);
        });
    }
}
