#![no_std]

mod errors;
mod events;
mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::errors::{AuthorizationError, BalanceError, StateError, ValidationError, VaultError};

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

    pub fn propose_new_admin(
        e: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;

        let configured_admin = storage::get_admin(&e)?;
        if current_admin != configured_admin {
            return Err(AuthorizationError::Unauthorized.into());
        }

        current_admin.require_auth();
        storage::set_pending_admin(&e, &new_admin);
        events::emit_admin_transfer_proposed(&e, current_admin, new_admin);

        Ok(())
    }

    pub fn accept_admin(e: Env, new_admin: Address) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        new_admin.require_auth();

        let previous_admin = storage::get_admin(&e)?;
        let pending_admin = storage::get_pending_admin(&e)?.ok_or(StateError::NoPendingAdmin)?;

        if pending_admin != new_admin {
            return Err(AuthorizationError::Unauthorized.into());
        }

        storage::set_admin(&e, &new_admin);
        storage::clear_pending_admin(&e);
        events::emit_admin_transfer_accepted(&e, previous_admin, new_admin);

        Ok(())
    }

    /// Deposits tokens into the vault and accrues pending rewards before updating balance.
    /// This ensures users receive rewards based on their old balance up to this point.
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
    /// This function is isolated from reward claiming - it only handles the deposit token.
    /// If the reward token contract fails, users can still withdraw their deposits.
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
    /// Does not immediately transfer rewards to users - they accrue lazily.
    pub fn distribute_rewards(e: Env, amount: i128) -> Result<i128, VaultError> {
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;

        let state = storage::get_state(&e)?;
        let admin = state.admin.clone();
        let reward_token_id = state.reward_token.clone();

        admin.require_auth();

        with_non_reentrant(&e, || {
            let next_state = storage::store_reward_distribution(&e, amount)?;
            let reward_token = soroban_sdk::token::Client::new(&e, &reward_token_id);
            reward_token.transfer(&admin, &e.current_contract_address(), &amount);
            events::emit_distribute(&e, admin.clone(), amount, next_state.reward_index);
            Ok(next_state.reward_index)
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

    pub fn pending_admin(e: Env) -> Result<Option<Address>, VaultError> {
        storage::get_pending_admin(&e)
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

// TODO(reward-optimization): Consider a higher precision / rounding strategy for small totals.
// TODO(gas): Consider merging per-user keys (balance/index/rewards) into a single struct to reduce reads.
// TODO(security): Consider adding pausability or per-user deposit caps.
// TODO(upgradeability): Evaluate upgrade patterns compatible with Soroban best practices.

#[cfg(test)]
mod tests {
    extern crate std;

    use soroban_sdk::{
        symbol_short,
        testutils::{Address as _, Events, Register},
        Address, Env, IntoVal, TryFromVal, Val, Vec,
    };

    use super::{
        events::{AdminTransferAcceptedEvent, AdminTransferProposedEvent},
        VaultContract, VaultContractClient,
    };
    use crate::errors::VaultError;

    #[test]
    fn admin_transfer_is_two_step() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = VaultContract.register(&env, None, ());
        let client = VaultContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);
        let deposit_token = Address::generate(&env);
        let reward_token = Address::generate(&env);

        client.initialize(&admin, &deposit_token, &reward_token);
        client.propose_new_admin(&admin, &new_admin);

        assert_eq!(client.admin(), admin);
        assert_eq!(client.pending_admin(), Some(new_admin.clone()));

        client.accept_admin(&new_admin);

        assert_eq!(client.admin(), new_admin);
        assert_eq!(client.pending_admin(), None);
    }

    #[test]
    fn accept_admin_requires_pending_admin_match() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = VaultContract.register(&env, None, ());
        let client = VaultContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let proposed_admin = Address::generate(&env);
        let wrong_admin = Address::generate(&env);
        let deposit_token = Address::generate(&env);
        let reward_token = Address::generate(&env);

        client.initialize(&admin, &deposit_token, &reward_token);
        client.propose_new_admin(&admin, &proposed_admin);

        let err = client.try_accept_admin(&wrong_admin).unwrap_err();
        assert_eq!(err, Ok(VaultError::Unauthorized));
        assert_eq!(client.admin(), admin);
        assert_eq!(client.pending_admin(), Some(proposed_admin));
    }

    #[test]
    fn proposing_and_accepting_emit_specific_events() {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = VaultContract.register(&env, None, ());
        let client = VaultContractClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let new_admin = Address::generate(&env);
        let deposit_token = Address::generate(&env);
        let reward_token = Address::generate(&env);

        client.initialize(&admin, &deposit_token, &reward_token);
        client.propose_new_admin(&admin, &new_admin);
        let proposed_events = env.events().all();
        let proposed = proposed_events.last().unwrap();

        assert_eq!(proposed.0, contract_id.clone());
        assert_eq!(proposed.1, topics(&env, symbol_short!("adm_prop")));
        assert_eq!(
            AdminTransferProposedEvent::try_from_val(&env, &proposed.2).unwrap(),
            AdminTransferProposedEvent {
                current_admin: admin.clone(),
                pending_admin: new_admin.clone(),
                timestamp: env.ledger().timestamp(),
            }
        );

        client.accept_admin(&new_admin);
        let accepted_events = env.events().all();
        let accepted = accepted_events.last().unwrap();

        assert_eq!(accepted.0, contract_id);
        assert_eq!(accepted.1, topics(&env, symbol_short!("adm_acpt")));
        assert_eq!(
            AdminTransferAcceptedEvent::try_from_val(&env, &accepted.2).unwrap(),
            AdminTransferAcceptedEvent {
                previous_admin: admin,
                new_admin,
                timestamp: env.ledger().timestamp(),
            }
        );
    }

    fn topics(env: &Env, topic: soroban_sdk::Symbol) -> Vec<Val> {
        let mut values = Vec::new(env);
        values.push_back(topic.into_val(env));
        values
    }
}
