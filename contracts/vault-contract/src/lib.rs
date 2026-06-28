#![no_std]

pub mod cross_contract;
pub mod errors;
mod events;
mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env};

use axionvera_accounting as accounting;

use crate::cross_contract::CrossContractClient;
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
        vesting_period: u64,
        target_deposits: i128,
        utilization_multipliers: soroban_sdk::Vec<storage::MultiplierPoint>,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        if storage::is_initialized(&e) {
            return Err(StateError::AlreadyInitialized.into());
        }

        validate_distinct_token_addresses(&deposit_token, &reward_token)?;
        validate_utilization_multipliers(&utilization_multipliers)?;

        admin.require_auth();

        storage::initialize_state(
            &e,
            &admin,
            &deposit_token,
            &reward_token,
            vesting_period,
            target_deposits,
            &utilization_multipliers,
        );
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::Initialize,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(1, 10, 2, 0),
        )?;
        events::emit_initialize(&e, admin, deposit_token, reward_token);

        Ok(())
    }

    pub fn propose_new_admin(e: Env, new_admin: Address) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;

        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        storage::set_pending_admin(&e, &new_admin);
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernanceAdminPropose,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(2, 1, 2, 0),
        )?;
        events::emit_admin_transfer_proposed(&e, admin, new_admin);

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
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernanceAdminAccept,
            Some(new_admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(3, 2, 2, 0),
        )?;
        events::emit_admin_transfer_accepted(&e, previous_admin, new_admin);

        Ok(())
    }

    pub fn deposit(e: Env, from: Address, amount: i128) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        from.require_auth();

        with_non_reentrant(&e, || {
            let state = storage::get_state(&e)?;
            CrossContractClient::token_transfer(
                &e,
                &state.deposit_token,
                &from,
                &e.current_contract_address(),
                amount,
            )?;

            let (_state, _position) = storage::store_deposit(&e, &from, amount)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::VaultDeposit,
                Some(from.clone()),
                Some(state.deposit_token.clone()),
                amount,
                0,
                amount,
                accounting::OperationResources::new(5, 5, 2, 1),
            )?;
            events::emit_deposit(&e, from.clone(), amount);
            Ok(())
        })
    }

    pub fn withdraw(e: Env, to: Address, amount: i128) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        to.require_auth();

        with_non_reentrant(&e, || {
            let (state, position) = storage::store_withdraw(&e, &to, amount)?;

            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::VaultWithdraw,
                Some(to.clone()),
                Some(state.deposit_token.clone()),
                0,
                amount,
                amount,
                accounting::OperationResources::new(6, 5, 2, 1),
            )?;
            events::emit_withdraw(&e, to.clone(), amount, position.balance);

            CrossContractClient::token_transfer(
                &e,
                &state.deposit_token,
                &e.current_contract_address(),
                &to,
                amount,
            )?;

            Ok(())
        })
    }

    pub fn distribute_rewards(e: Env, amount: i128) -> Result<i128, VaultError> {
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;

        const MIN_REWARD_DISTRIBUTION: i128 = 100_000;
        if amount < MIN_REWARD_DISTRIBUTION {
            return Err(ValidationError::InsufficientRewardAmount.into());
        }

        let state = storage::get_state(&e)?;
        let admin = state.admin.clone();
        let reward_token_id = state.reward_token.clone();

        admin.require_auth();

        with_non_reentrant(&e, || {
            CrossContractClient::token_transfer(
                &e,
                &reward_token_id,
                &admin,
                &e.current_contract_address(),
                amount,
            )?;

            let next_state = storage::store_reward_distribution(&e, amount)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Rewards,
                accounting::AccountingOperation::RewardDistribute,
                Some(admin.clone()),
                Some(reward_token_id.clone()),
                amount,
                0,
                amount,
                accounting::OperationResources::new(4, 2, 2, 1),
            )?;
            events::emit_distribute(&e, admin.clone(), amount);
            Ok(next_state.reward_index)
        })
    }

    pub fn lock(
        e: Env,
        from: Address,
        amount: i128,
        duration_seconds: u64,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        if duration_seconds == 0 {
            return Err(ValidationError::InvalidLockDuration.into());
        }
        from.require_auth();

        with_non_reentrant(&e, || {
            let unlock_timestamp = e
                .ledger()
                .timestamp()
                .checked_add(duration_seconds)
                .ok_or(VaultError::MathOverflow)?;
            storage::store_lock(&e, &from, amount, duration_seconds)?;
            let deposit_token = storage::get_deposit_token(&e)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::VaultLock,
                Some(from.clone()),
                Some(deposit_token),
                0,
                0,
                amount,
                accounting::OperationResources::new(4, 3, 2, 0),
            )?;
            events::emit_lock(&e, from, amount, unlock_timestamp);
            Ok(())
        })
    }

    pub fn unlock_expired(e: Env, user: Address, limit: u32) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        user.require_auth();

        // Enforce a maximum limit to prevent budget exhaustion in a single call.
        const MAX_UNLOCK_LIMIT: u32 = 50;
        if limit > MAX_UNLOCK_LIMIT {
            return Err(VaultError::OperationLimitExceeded);
        }

        with_non_reentrant(&e, || {
            let unlocked_amount = storage::unlock_expired_locks(&e, &user, limit)?;
            if unlocked_amount > 0 {
                let deposit_token = storage::get_deposit_token(&e)?;
                account_operation(
                    &e,
                    accounting::AccountingCategory::Vault,
                    accounting::AccountingOperation::VaultUnlock,
                    Some(user.clone()),
                    Some(deposit_token),
                    0,
                    0,
                    unlocked_amount,
                    accounting::OperationResources::new(3, 2, 2, 0),
                )?;
                events::emit_unlock(&e, user, unlocked_amount);
            }
            Ok(unlocked_amount)
        })
    }

    pub fn claim_rewards(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        user.require_auth();

        with_non_reentrant(&e, || {
            let amt = storage::store_claimable_rewards(&e, &user)?;
            if amt <= 0 {
                return Ok(0);
            }

            let reward_token_id = storage::get_reward_token(&e)?;
            let contract_balance = CrossContractClient::token_balance(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
            )?;
            ensure_contract_balance(contract_balance, amt)?;
            CrossContractClient::token_transfer(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
                &user,
                amt,
            )?;

            account_operation(
                &e,
                accounting::AccountingCategory::Rewards,
                accounting::AccountingOperation::RewardClaim,
                Some(user.clone()),
                Some(reward_token_id),
                0,
                amt,
                amt,
                accounting::OperationResources::new(5, 3, 2, 1),
            )?;
            events::emit_claim_rewards(&e, user, amt);
            Ok(amt)
        })
    }

    pub fn balance(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::get_user_balance(&e, &user)
    }

    pub fn liquid_balance(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::get_liquid_balance(&e, &user)
    }

    pub fn locked_balance(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::get_locked_balance(&e, &user)
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

    pub fn vested_rewards(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::vested_user_rewards_view(&e, &user)
    }

    pub fn vesting_period(e: Env) -> Result<u64, VaultError> {
        storage::get_vesting_period(&e)
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

    pub fn pause_contract(e: Env) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_paused(&e, true);
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernancePause,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(2, 1, 2, 0),
        )?;
        events::emit_pause(&e, admin);
        Ok(())
    }

    pub fn unpause_contract(e: Env) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();
        storage::set_paused(&e, false);
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernanceUnpause,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(2, 1, 2, 0),
        )?;
        events::emit_unpause(&e, admin);
        Ok(())
    }

    pub fn set_penalty_rate(e: Env, admin: Address, rate_bps: u32) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let stored_admin = storage::get_admin(&e)?;
        if admin != stored_admin {
            return Err(AuthorizationError::Unauthorized.into());
        }
        admin.require_auth();
        if rate_bps > 10000 {
            return Err(ValidationError::InvalidPenaltyRate.into());
        }
        storage::set_penalty_rate_bps(&e, rate_bps);
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernanceSetParameter,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(2, 1, 1, 0),
        )?;
        Ok(())
    }

    pub fn penalty_rate(e: Env) -> Result<u32, VaultError> {
        storage::get_penalty_rate_bps(&e)
    }

    pub fn total_penalties(e: Env) -> Result<i128, VaultError> {
        storage::get_total_penalties(&e)
    }

    pub fn user_penalties(e: Env, user: Address) -> Result<i128, VaultError> {
        storage::get_user_penalty_paid(&e, &user)
    }

    pub fn withdraw_locked_early(e: Env, to: Address, amount: i128) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        to.require_auth();

        with_non_reentrant(&e, || {
            let (state, position, net_amount, penalty) =
                storage::store_early_withdraw_locked(&e, &to, amount)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::VaultEarlyWithdraw,
                Some(to.clone()),
                Some(state.deposit_token.clone()),
                0,
                net_amount,
                amount,
                accounting::OperationResources::new(7, 6, if penalty > 0 { 3 } else { 2 }, 1),
            )?;
            if penalty > 0 {
                account_operation(
                    &e,
                    accounting::AccountingCategory::Treasury,
                    accounting::AccountingOperation::TreasuryPenalty,
                    Some(to.clone()),
                    Some(state.deposit_token.clone()),
                    penalty,
                    0,
                    penalty,
                    accounting::OperationResources::new(2, 2, 1, 0),
                )?;
            }
            events::emit_withdraw(&e, to.clone(), net_amount, position.balance);
            CrossContractClient::token_transfer(
                &e,
                &state.deposit_token,
                &e.current_contract_address(),
                &to,
                net_amount,
            )?;
            Ok(net_amount)
        })
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let admin = storage::get_admin(&e)?;
        admin.require_auth();

        e.deployer()
            .update_current_contract_wasm(new_wasm_hash.clone());
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::GovernanceUpgrade,
            Some(admin.clone()),
            None,
            0,
            0,
            0,
            accounting::OperationResources::new(2, 1, 2, 0),
        )?;
        events::emit_upgrade(&e, admin, new_wasm_hash);

        Ok(())
    }

    // ---------------------------------------------------------------------------
    // Multi-Asset Functions
    // ---------------------------------------------------------------------------

    pub fn add_asset(e: Env, admin: Address, asset: Address) -> Result<(), VaultError> {
        storage::require_initialized(&e)?;
        let stored_admin = storage::get_admin(&e)?;
        if admin != stored_admin {
            return Err(AuthorizationError::Unauthorized.into());
        }
        admin.require_auth();

        storage::add_supported_asset(&e, &asset)?;
        account_operation(
            &e,
            accounting::AccountingCategory::Governance,
            accounting::AccountingOperation::AssetAdded,
            Some(admin.clone()),
            Some(asset.clone()),
            0,
            0,
            0,
            accounting::OperationResources::new(3, 2, 2, 0),
        )?;
        events::emit_asset_added(&e, asset);
        Ok(())
    }

    pub fn deposit_asset(
        e: Env,
        from: Address,
        asset: Address,
        amount: i128,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        from.require_auth();

        if !storage::is_asset_supported(&e, &asset) {
            return Err(ValidationError::InvalidAddress.into());
        }

        with_non_reentrant(&e, || {
            CrossContractClient::token_transfer(
                &e,
                &asset,
                &from,
                &e.current_contract_address(),
                amount,
            )?;

            let _position = storage::store_asset_deposit(&e, &from, &asset, amount)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::AssetDeposit,
                Some(from.clone()),
                Some(asset.clone()),
                amount,
                0,
                amount,
                accounting::OperationResources::new(5, 4, 2, 1),
            )?;
            events::emit_asset_deposit(&e, from.clone(), asset.clone(), amount);
            Ok(())
        })
    }

    pub fn withdraw_asset(
        e: Env,
        to: Address,
        asset: Address,
        amount: i128,
    ) -> Result<(), VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;
        to.require_auth();

        if !storage::is_asset_supported(&e, &asset) {
            return Err(ValidationError::InvalidAddress.into());
        }

        with_non_reentrant(&e, || {
            let position = storage::store_asset_withdraw(&e, &to, &asset, amount)?;

            account_operation(
                &e,
                accounting::AccountingCategory::Vault,
                accounting::AccountingOperation::AssetWithdraw,
                Some(to.clone()),
                Some(asset.clone()),
                0,
                amount,
                amount,
                accounting::OperationResources::new(5, 4, 2, 1),
            )?;
            events::emit_asset_withdraw(&e, to.clone(), asset.clone(), amount, position.balance);

            CrossContractClient::token_transfer(
                &e,
                &asset,
                &e.current_contract_address(),
                &to,
                amount,
            )?;

            Ok(())
        })
    }

    pub fn distribute_rewards_for_asset(
        e: Env,
        admin: Address,
        asset: Address,
        amount: i128,
    ) -> Result<i128, VaultError> {
        storage::require_initialized(&e)?;
        validate_positive_amount(amount)?;

        const MIN_REWARD_DISTRIBUTION: i128 = 100_000;
        if amount < MIN_REWARD_DISTRIBUTION {
            return Err(ValidationError::InsufficientRewardAmount.into());
        }

        if !storage::is_asset_supported(&e, &asset) {
            return Err(ValidationError::InvalidAddress.into());
        }

        let state = storage::get_state(&e)?;
        let stored_admin = state.admin.clone();
        if admin != stored_admin {
            return Err(AuthorizationError::Unauthorized.into());
        }

        let reward_token_id = state.reward_token.clone();
        admin.require_auth();

        with_non_reentrant(&e, || {
            CrossContractClient::token_transfer(
                &e,
                &reward_token_id,
                &admin,
                &e.current_contract_address(),
                amount,
            )?;

            let next_reward_index = storage::store_asset_reward_distribution(&e, &asset, amount)?;
            account_operation(
                &e,
                accounting::AccountingCategory::Rewards,
                accounting::AccountingOperation::AssetRewardDistribute,
                Some(admin.clone()),
                Some(asset.clone()),
                amount,
                0,
                amount,
                accounting::OperationResources::new(5, 3, 2, 1),
            )?;
            events::emit_asset_distribute(&e, admin.clone(), asset.clone(), amount);
            Ok(next_reward_index)
        })
    }

    pub fn claim_rewards_for_asset(
        e: Env,
        user: Address,
        asset: Address,
    ) -> Result<i128, VaultError> {
        storage::require_not_paused(&e)?;
        storage::require_initialized(&e)?;
        user.require_auth();

        if !storage::is_asset_supported(&e, &asset) {
            return Err(ValidationError::InvalidAddress.into());
        }

        with_non_reentrant(&e, || {
            let amt = storage::store_asset_claimable_rewards(&e, &user, &asset)?;
            if amt <= 0 {
                return Ok(0);
            }

            let reward_token_id = storage::get_reward_token(&e)?;
            let contract_balance = CrossContractClient::token_balance(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
            )?;
            ensure_contract_balance(contract_balance, amt)?;
            CrossContractClient::token_transfer(
                &e,
                &reward_token_id,
                &e.current_contract_address(),
                &user,
                amt,
            )?;

            account_operation(
                &e,
                accounting::AccountingCategory::Rewards,
                accounting::AccountingOperation::AssetRewardClaim,
                Some(user.clone()),
                Some(asset.clone()),
                0,
                amt,
                amt,
                accounting::OperationResources::new(5, 3, 2, 1),
            )?;
            events::emit_asset_claim_rewards(&e, user, asset, amt);
            Ok(amt)
        })
    }

    pub fn balance_of_asset(e: Env, user: Address, asset: Address) -> Result<i128, VaultError> {
        storage::get_user_asset_balance(&e, &user, &asset)
    }

    pub fn total_deposits_of_asset(e: Env, asset: Address) -> Result<i128, VaultError> {
        storage::get_asset_total_deposits(&e, &asset)
    }

    pub fn reward_index_of_asset(e: Env, asset: Address) -> Result<i128, VaultError> {
        storage::get_asset_reward_index(&e, &asset)
    }

    pub fn pending_rewards_for_asset(
        e: Env,
        user: Address,
        asset: Address,
    ) -> Result<i128, VaultError> {
        storage::pending_user_asset_rewards_view(&e, &user, &asset)
    }

    pub fn vested_rewards_for_asset(
        e: Env,
        user: Address,
        asset: Address,
    ) -> Result<i128, VaultError> {
        storage::vested_user_asset_rewards_view(&e, &user, &asset)
    }

    pub fn is_asset_supported(e: Env, asset: Address) -> bool {
        storage::is_asset_supported(&e, &asset)
    }

    // ---------------------------------------------------------------------------
    // Accounting Views
    // ---------------------------------------------------------------------------

    pub fn accounting_report(e: Env) -> accounting::AccountingReport {
        accounting::accounting_report(&e)
    }

    pub fn accounting_totals(e: Env) -> accounting::ResourceTotals {
        accounting::get_total_usage(&e)
    }

    pub fn accounting_category(
        e: Env,
        category: accounting::AccountingCategory,
    ) -> accounting::ResourceTotals {
        accounting::get_category_usage(&e, category)
    }

    pub fn accounting_operation(
        e: Env,
        operation: accounting::AccountingOperation,
    ) -> accounting::ResourceTotals {
        accounting::get_operation_usage(&e, operation)
    }

    pub fn accounting_asset(e: Env, asset: Address) -> accounting::ResourceTotals {
        accounting::get_asset_usage(&e, &asset)
    }

    pub fn accounting_is_consistent(e: Env) -> bool {
        accounting::validate_accounting(&e)
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

fn validate_utilization_multipliers(
    multipliers: &soroban_sdk::Vec<storage::MultiplierPoint>,
) -> Result<(), VaultError> {
    if multipliers.is_empty() {
        return Ok(()); // An empty list is valid, which causes rewards to default to 1.0x.
    }

    let mut last_util_bps = 0;
    for point in multipliers.iter() {
        if point.utilization_bps < last_util_bps {
            // The list must be sorted by utilization_bps in ascending order.
            return Err(ValidationError::InvalidUtilizationParameters.into());
        }
        last_util_bps = point.utilization_bps;
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

fn account_operation(
    e: &Env,
    category: accounting::AccountingCategory,
    operation: accounting::AccountingOperation,
    actor: Option<Address>,
    asset: Option<Address>,
    amount_in: i128,
    amount_out: i128,
    amount_processed: i128,
    resources: accounting::OperationResources,
) -> Result<(), VaultError> {
    accounting::record_operation(
        e,
        accounting::AccountingEntry {
            category,
            operation,
            actor,
            asset,
            amount_in,
            amount_out,
            amount_processed,
            resources,
        },
    )
    .map_err(accounting_error_to_vault_error)
}

fn accounting_error_to_vault_error(error: accounting::AccountingError) -> VaultError {
    match error {
        accounting::AccountingError::NegativeAmount => VaultError::NegativeAmount,
        accounting::AccountingError::Overflow => VaultError::MathOverflow,
        accounting::AccountingError::InconsistentTotals => VaultError::InvalidState,
    }
}

#[cfg(test)]
mod precision_tests {
    use super::errors::VaultError;
    use super::storage::{
        checked_accrued_rewards, checked_reward_index_increment, PRECISION_FACTOR,
    };

    #[test]
    fn increment_basic() {
        let inc = checked_reward_index_increment(400, 400).unwrap();
        assert_eq!(inc, PRECISION_FACTOR);
    }

    #[test]
    fn increment_small_reward_large_deposits_retains_precision() {
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
        let delta = checked_reward_index_increment(400, 200).unwrap();
        let reward = checked_accrued_rewards(100, delta).unwrap();
        assert_eq!(reward, 200);
    }

    #[test]
    fn accrued_vastly_different_deposits_user_a_tiny() {
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
        let total = 1_000_000_i128;
        let rewards = 1_000_000_i128;
        let delta = checked_reward_index_increment(rewards, total).unwrap();

        assert_eq!(checked_accrued_rewards(1, delta).unwrap(), 1);
        assert_eq!(checked_accrued_rewards(999_999, delta).unwrap(), 999_999);
    }
}
