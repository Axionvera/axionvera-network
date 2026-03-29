#![no_std]

mod errors;
mod events;
mod storage;

use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::errors::VaultError;

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
            return Err(VaultError::AlreadyInitialized);
        }

        admin.require_auth();
        validate_init_config(&e, &admin, &deposit_token, &reward_token)?;

        storage::initialize_state(&e, &admin, &deposit_token, &reward_token);

        events::emit_initialize(&e, admin, deposit_token, reward_token);
        Ok(())
    }

    pub fn deposit(e: Env, from: Address, amount: i128) -> Result<(), VaultError> {
        validate_positive_amount(amount)?;
        from.require_auth();
        with_non_reentrant(&e, || {
            let (state, position) = storage::store_deposit(&e, &from, amount)?;
            let token = soroban_sdk::token::Client::new(&e, &state.deposit_token);
            token.transfer(&from, &e.current_contract_address(), &amount);
            events::emit_deposit(&e, from, amount, position.balance);
            Ok(())
        })
    }

    pub fn withdraw(e: Env, to: Address, amount: i128) -> Result<(), VaultError> {
        validate_positive_amount(amount)?;
        to.require_auth();
        with_non_reentrant(&e, || {
            let (state, position) = storage::store_withdraw(&e, &to, amount)?;
            let next_balance = position.balance;
            let token = soroban_sdk::token::Client::new(&e, &state.deposit_token);
            token.transfer(&e.current_contract_address(), &to, &amount);

            events::emit_withdraw(&e, to, amount, next_balance);
            Ok(())
        })
    }

    pub fn distribute_rewards(e: Env, amount: i128) -> Result<i128, VaultError> {
        validate_positive_amount(amount)?;

        let state = storage::get_state(&e)?;
        let admin = state.admin;
        let reward_token = state.reward_token;
        admin.require_auth();
        with_non_reentrant(&e, || {
            let next_idx = storage::store_reward_distribution(&e, amount)?.reward_index;
            let reward_token_client = soroban_sdk::token::Client::new(&e, &reward_token);
            reward_token_client.transfer(&admin, &e.current_contract_address(), &amount);
            events::emit_distribute(&e, admin, amount, next_idx);
            Ok(next_idx)
        })
    }

    pub fn claim_rewards(e: Env, user: Address) -> Result<i128, VaultError> {
        user.require_auth();
        with_non_reentrant(&e, || {
            let amt = storage::store_claimable_rewards(&e, &user)?;
            if amt <= 0 {
                return Ok(0);
            }

            let reward_token_id = storage::get_reward_token(&e)?;
            let reward_token = soroban_sdk::token::Client::new(&e, &reward_token_id);
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
}

fn validate_positive_amount(amount: i128) -> Result<(), VaultError> {
    if amount <= 0 {
        return Err(VaultError::InvalidAmount);
    }
    Ok(())
}

fn validate_init_config(
    e: &Env,
    admin: &Address,
    deposit_token: &Address,
    reward_token: &Address,
) -> Result<(), VaultError> {
    let contract = e.current_contract_address();
    if admin == &contract || deposit_token == &contract || reward_token == &contract {
        return Err(VaultError::InvalidConfiguration);
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
// TODO(security): Consider adding pausability or per-user deposit caps.
// TODO(governance): Introduce admin handover / multisig patterns.
// TODO(upgradeability): Evaluate upgrade patterns compatible with Soroban best practices.

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::token::StellarAssetClient;

    // ===== Deposit Logic Tests =====

    #[test]
    fn test_deposit_logic() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        // Test single deposit
        vault.deposit(&user, &400);
        assert_eq!(vault.balance(&user), 400);
        assert_eq!(vault.total_deposits(), 400);

        // Test multiple deposits accumulate
        vault.deposit(&user, &200);
        assert_eq!(vault.balance(&user), 600);
        assert_eq!(vault.total_deposits(), 600);
    }

    #[test]
    fn test_multiple_deposits_accumulate() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &5_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        vault.deposit(&user, &100);
        assert_eq!(vault.balance(&user), 100);
        assert_eq!(vault.total_deposits(), 100);

        vault.deposit(&user, &200);
        assert_eq!(vault.balance(&user), 300);
        assert_eq!(vault.total_deposits(), 300);

        vault.deposit(&user, &700);
        assert_eq!(vault.balance(&user), 1_000);
        assert_eq!(vault.total_deposits(), 1_000);
    }

    #[test]
    fn test_deposit_increases_total() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user1 = Address::generate(&e);
        let user2 = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user1, &1_000);
        deposit_token.mint(&user2, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        let amount1 = 100_i128;
        let amount2 = 250_i128;

        vault.deposit(&user1, &amount1);
        assert_eq!(vault.total_deposits(), amount1);

        vault.deposit(&user2, &amount2);
        assert_eq!(vault.total_deposits(), amount1 + amount2);
    }

    // ===== Withdraw Logic Tests =====

    #[test]
    fn test_withdraw_logic() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        // Test withdraw logic
        vault.deposit(&user, &500);
        assert_eq!(vault.balance(&user), 500);

        vault.withdraw(&user, &200);
        assert_eq!(vault.balance(&user), 300);
        assert_eq!(vault.total_deposits(), 300);
    }

    #[test]
    fn test_multiple_withdrawals_work_correctly() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        vault.deposit(&user, &1_000);
        assert_eq!(vault.balance(&user), 1_000);

        vault.withdraw(&user, &100);
        assert_eq!(vault.balance(&user), 900);
        assert_eq!(vault.total_deposits(), 900);

        vault.withdraw(&user, &250);
        assert_eq!(vault.balance(&user), 650);
        assert_eq!(vault.total_deposits(), 650);

        vault.withdraw(&user, &650);
        assert_eq!(vault.balance(&user), 0);
        assert_eq!(vault.total_deposits(), 0);
    }

    #[test]
    fn test_withdraw_entire_balance() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        let amount = 500_i128;
        vault.deposit(&user, &amount);
        vault.withdraw(&user, &amount);

        assert_eq!(vault.balance(&user), 0);
        assert_eq!(vault.total_deposits(), 0);
    }

    #[test]
    fn test_deposit_after_reward_distribution() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user1 = Address::generate(&e);
        let user2 = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        let reward_token = StellarAssetClient::new(&e, &reward_token_id);

        deposit_token.mint(&user1, &1_000);
        deposit_token.mint(&user2, &1_000);
        reward_token.mint(&admin, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        let deposit1 = 500_i128;
        let reward_amount = 100_i128;
        let deposit2 = 200_i128;

        vault.deposit(&user1, &deposit1);
        vault.distribute_rewards(&reward_amount);
        vault.deposit(&user2, &deposit2);

        let total = vault.total_deposits();
        assert_eq!(total, deposit1 + deposit2);
    }

    // ===== Reward Distribution Tests =====

    #[test]
    fn test_reward_distribution() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        let reward_token = StellarAssetClient::new(&e, &reward_token_id);

        deposit_token.mint(&user, &1_000);
        reward_token.mint(&admin, &100);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        let deposit = 1_000_i128;
        let reward = 100_i128;

        vault.deposit(&user, &deposit);

        let reward_index_before = vault.reward_index();
        assert_eq!(reward_index_before, 0);

        vault.distribute_rewards(&reward);

        let reward_index_after = vault.reward_index();
        assert!(reward_index_after > 0);
    }

    #[test]
    fn test_rewards_are_proportional_and_claimable() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let alice = Address::generate(&e);
        let bob = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        let reward_token = StellarAssetClient::new(&e, &reward_token_id);

        deposit_token.mint(&alice, &1_000);
        deposit_token.mint(&bob, &1_000);
        reward_token.mint(&admin, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);
        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        // User1 deposits 2x more than User2
        let deposit1 = 200_i128;
        let deposit2 = 100_i128;
        let reward_amount = 300_i128;

        vault.deposit(&alice, &deposit1);
        vault.deposit(&bob, &deposit2);

        vault.distribute_rewards(&reward_amount);

        let pending_alice = vault.pending_rewards(&alice);
        let pending_bob = vault.pending_rewards(&bob);

        assert!(pending_alice > 0);
        assert!(pending_bob > 0);
        assert!(pending_alice > pending_bob);

        let claimed_alice = vault.claim_rewards(&alice);
        let claimed_bob = vault.claim_rewards(&bob);

        assert_eq!(claimed_alice, pending_alice);
        assert_eq!(claimed_bob, pending_bob);
    }

    #[test]
    fn test_multiple_reward_distributions_accumulate() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        let reward_token = StellarAssetClient::new(&e, &reward_token_id);

        deposit_token.mint(&user, &1_000);
        reward_token.mint(&admin, &3_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        let deposit = 1_000_i128;
        let reward1 = 100_i128;
        let reward2 = 50_i128;

        vault.deposit(&user, &deposit);
        vault.distribute_rewards(&reward1);

        let pending_after_first = vault.pending_rewards(&user);

        vault.distribute_rewards(&reward2);

        let pending_after_second = vault.pending_rewards(&user);

        assert!(pending_after_second > pending_after_first);
    }

    #[test]
    fn test_reward_proportionality_with_unequal_deposits() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user1 = Address::generate(&e);
        let user2 = Address::generate(&e);
        let user3 = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        let reward_token = StellarAssetClient::new(&e, &reward_token_id);

        deposit_token.mint(&user1, &1_000);
        deposit_token.mint(&user2, &2_000);
        deposit_token.mint(&user3, &3_000);
        reward_token.mint(&admin, &6_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        // Deposits in ratio 1:2:3
        vault.deposit(&user1, &1_000);
        vault.deposit(&user2, &2_000);
        vault.deposit(&user3, &3_000);

        // Distribute 600 rewards
        vault.distribute_rewards(&600);

        let pending1 = vault.pending_rewards(&user1);
        let pending2 = vault.pending_rewards(&user2);
        let pending3 = vault.pending_rewards(&user3);

        // Rewards should be proportional to deposits
        assert_eq!(pending1, 100);
        assert_eq!(pending2, 200);
        assert_eq!(pending3, 300);

        let claimed1 = vault.claim_rewards(&user1);
        let claimed2 = vault.claim_rewards(&user2);
        let claimed3 = vault.claim_rewards(&user3);

        assert_eq!(claimed1, 100);
        assert_eq!(claimed2, 200);
        assert_eq!(claimed3, 300);
    }

    #[test]
    fn test_claim_with_zero_rewards() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        // Try to claim without any rewards
        let claimed = vault.claim_rewards(&user);
        assert_eq!(claimed, 0);
    }

    // ===== Edge Cases Tests =====

    #[test]
    fn test_rejects_invalid_amounts() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);
        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        // Test zero deposit
        assert!(vault.try_deposit(&user, &0).is_err());

        // Test negative deposit
        assert!(vault.try_deposit(&user, &-1_000).is_err());

        // Test zero withdraw
        assert!(vault.try_withdraw(&user, &0).is_err());

        // Test negative withdraw
        assert!(vault.try_withdraw(&user, &-500).is_err());
    }

    #[test]
    fn test_negative_deposits_rejected() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        assert!(vault.try_deposit(&user, &-100).is_err());
    }

    #[test]
    fn test_negative_withdrawals_rejected() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        vault.deposit(&user, &500);

        assert!(vault.try_withdraw(&user, &-100).is_err());
    }

    #[test]
    fn test_cannot_withdraw_more_than_balance() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &500);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);
        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        vault.deposit(&user, &200);

        assert!(vault.try_withdraw(&user, &201).is_err());
    }

    #[test]
    fn test_large_deposit_and_withdraw() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        let large_amount = 9_223_372_036_854_775_000i128; // Near i128 max
        deposit_token.mint(&user, &large_amount);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        vault.deposit(&user, &large_amount);
        assert_eq!(vault.balance(&user), large_amount);
        assert_eq!(vault.total_deposits(), large_amount);

        vault.withdraw(&user, &large_amount);
        assert_eq!(vault.balance(&user), 0);
        assert_eq!(vault.total_deposits(), 0);
    }

    #[test]
    fn test_distribute_requires_deposits() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let reward_token = StellarAssetClient::new(&e, &reward_token_id);
        reward_token.mint(&admin, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);
        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        // Try to distribute rewards without any deposits
        assert!(vault.try_distribute_rewards(&100).is_err());
    }

    #[test]
    fn test_initialization_is_one_time() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);
        assert!(vault
            .try_initialize(&admin, &deposit_token_id, &reward_token_id)
            .is_err());
    }

    // ===== Query Tests =====

    #[test]
    fn test_query_functions_return_correct_values() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        // Test version
        assert_eq!(VaultContract::version(), 1);

        // Test initial state
        assert_eq!(vault.total_deposits(), 0);
        assert_eq!(vault.reward_index(), 0);
        assert_eq!(vault.balance(&user), 0);
        assert_eq!(vault.pending_rewards(&user), 0);
    }

    // ===== Round-Trip Tests =====

    #[test]
    fn test_deposit_withdraw_round_trip() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        let deposit = 750_i128;
        let withdraw = 500_i128;

        vault.deposit(&user, &deposit);
        let balance_after_deposit = vault.balance(&user);
        assert_eq!(balance_after_deposit, deposit);

        vault.withdraw(&user, &withdraw);
        let balance_after_withdraw = vault.balance(&user);
        assert_eq!(balance_after_withdraw, deposit - withdraw);

        let total = vault.total_deposits();
        assert_eq!(total, deposit - withdraw);
    }

    #[test]
    fn test_multiple_users_deposits_and_withdrawals() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user1 = Address::generate(&e);
        let user2 = Address::generate(&e);
        let user3 = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user1, &2_000);
        deposit_token.mint(&user2, &3_000);
        deposit_token.mint(&user3, &1_500);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        let deposit1 = 800_i128;
        let deposit2 = 1_200_i128;
        let deposit3 = 500_i128;

        vault.deposit(&user1, &deposit1);
        assert_eq!(vault.balance(&user1), deposit1);
        assert_eq!(vault.total_deposits(), deposit1);

        vault.deposit(&user2, &deposit2);
        assert_eq!(vault.balance(&user2), deposit2);
        assert_eq!(vault.total_deposits(), deposit1 + deposit2);

        vault.deposit(&user3, &deposit3);
        assert_eq!(vault.balance(&user3), deposit3);
        assert_eq!(vault.total_deposits(), deposit1 + deposit2 + deposit3);

        vault.withdraw(&user1, &300);
        assert_eq!(vault.balance(&user1), deposit1 - 300);
        assert_eq!(vault.total_deposits(), deposit1 - 300 + deposit2 + deposit3);

        vault.withdraw(&user2, &400);
        assert_eq!(vault.balance(&user2), deposit2 - 400);

        let final_total =
            (deposit1 - 300) + (deposit2 - 400) + deposit3;
        assert_eq!(vault.total_deposits(), final_total);
    }

    #[test]
    fn test_reward_accrual_on_deposit_withdrawal_sequence() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        let reward_token = StellarAssetClient::new(&e, &reward_token_id);

        deposit_token.mint(&user, &5_000);
        reward_token.mint(&admin, &2_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);

        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        let deposit = 1_000_i128;
        let reward = 200_i128;

        vault.deposit(&user, &deposit);
        vault.distribute_rewards(&reward);

        let pending_before_withdraw = vault.pending_rewards(&user);
        assert!(pending_before_withdraw > 0);

        vault.withdraw(&user, &(deposit / 2));

        let pending_after_withdraw = vault.pending_rewards(&user);
        assert_eq!(pending_before_withdraw, pending_after_withdraw);
    }

    #[test]
    fn test_edge_cases() {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let deposit_token_id = e.register_stellar_asset_contract(admin.clone());
        let reward_token_id = e.register_stellar_asset_contract(admin.clone());

        let deposit_token = StellarAssetClient::new(&e, &deposit_token_id);
        deposit_token.mint(&user, &1_000);

        let vault_id = e.register_contract(None, VaultContract);
        let vault = VaultContractClient::new(&e, &vault_id);
        vault.initialize(&admin, &deposit_token_id, &reward_token_id);

        // Test zero values are rejected
        assert!(vault.try_deposit(&user, &0).is_err());
        assert!(vault.try_deposit(&user, &-50).is_err());

        // Test insufficient balance on withdraw
        vault.deposit(&user, &200);
        assert!(vault.try_withdraw(&user, &201).is_err());

        // Test invalid amounts
        assert!(vault.try_withdraw(&user, &0).is_err());
    }
}
