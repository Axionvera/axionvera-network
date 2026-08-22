#![no_std]

use axionvera_rewards::calculate_pending_rewards;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VaultError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InvalidAmount = 3,
    InsufficientBalance = 4,
    InvalidRewardState = 5,
}

#[contracttype]
enum DataKey {
    Initialized,
    Admin,
    DepositToken,
    RewardToken,
    Balance(Address),
    TotalDeposits,
    RewardBalance,
    ClaimableReward(Address),
}

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        deposit_token: Address,
        reward_token: Address,
    ) -> Result<(), VaultError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(VaultError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::DepositToken, &deposit_token);
        env.storage()
            .instance()
            .set(&DataKey::RewardToken, &reward_token);
        env.storage()
            .instance()
            .set(&DataKey::TotalDeposits, &0_i128);
        env.storage()
            .instance()
            .set(&DataKey::RewardBalance, &0_i128);
        env.events().publish(
            (symbol_short!("vault"), symbol_short!("init")),
            admin.clone(),
        );
        Ok(())
    }

    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<i128, VaultError> {
        Self::require_initialized(&env)?;
        from.require_auth();
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let balance = Self::balance(&env, &from);
        let total = Self::total(&env);
        let new_balance = balance
            .checked_add(amount)
            .ok_or(VaultError::InvalidAmount)?;
        let new_total = total.checked_add(amount).ok_or(VaultError::InvalidAmount)?;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(from.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalDeposits, &new_total);
        env.events().publish(
            (symbol_short!("vault"), symbol_short!("deposit")),
            (from.clone(), amount),
        );
        Ok(new_balance)
    }

    pub fn withdraw(env: Env, to: Address, amount: i128) -> Result<i128, VaultError> {
        Self::require_initialized(&env)?;
        to.require_auth();
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let balance = Self::balance(&env, &to);
        if amount > balance {
            return Err(VaultError::InsufficientBalance);
        }
        let new_balance = balance
            .checked_sub(amount)
            .ok_or(VaultError::InvalidAmount)?;
        let new_total = Self::total(&env)
            .checked_sub(amount)
            .ok_or(VaultError::InvalidAmount)?;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to.clone()), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalDeposits, &new_total);
        env.events().publish(
            (symbol_short!("vault"), symbol_short!("withdraw")),
            (to.clone(), amount),
        );
        Ok(new_balance)
    }

    pub fn claim_rewards(env: Env, user: Address) -> Result<i128, VaultError> {
        Self::require_initialized(&env)?;
        user.require_auth();
        let claimable = Self::claimable_reward(&env, &user);
        if claimable == 0 {
            return Ok(0);
        }
        env.storage()
            .persistent()
            .set(&DataKey::ClaimableReward(user.clone()), &0_i128);
        env.events().publish(
            (symbol_short!("vault"), symbol_short!("claim")),
            (user, claimable),
        );
        Ok(claimable)
    }

    pub fn set_claimable_reward(env: Env, user: Address, amount: i128) -> Result<(), VaultError> {
        Self::require_initialized(&env)?;
        env.storage()
            .persistent()
            .set(&DataKey::ClaimableReward(user), &amount);
        Ok(())
    }

    pub fn set_reward_balance(env: Env, amount: i128) -> Result<(), VaultError> {
        Self::require_initialized(&env)?;
        env.storage()
            .instance()
            .set(&DataKey::RewardBalance, &amount);
        Ok(())
    }

    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    pub fn admin(env: Env) -> Result<Address, VaultError> {
        Self::require_initialized(&env)?;
        Ok(env.storage().instance().get(&DataKey::Admin).unwrap())
    }

    pub fn deposit_token(env: Env) -> Result<Address, VaultError> {
        Self::require_initialized(&env)?;
        Ok(env
            .storage()
            .instance()
            .get(&DataKey::DepositToken)
            .unwrap())
    }

    pub fn reward_token(env: Env) -> Result<Address, VaultError> {
        Self::require_initialized(&env)?;
        Ok(env.storage().instance().get(&DataKey::RewardToken).unwrap())
    }

    pub fn total_deposits(env: Env) -> Result<i128, VaultError> {
        Self::require_initialized(&env)?;
        Ok(Self::total(&env))
    }

    pub fn user_balance(env: Env, user: Address) -> Result<i128, VaultError> {
        Self::require_initialized(&env)?;
        Ok(Self::balance(&env, &user))
    }

    pub fn pending_rewards(env: Env, user: Address) -> Result<i128, VaultError> {
        Self::require_initialized(&env)?;
        Ok(calculate_pending_rewards(
            Self::balance(&env, &user),
            Self::total(&env),
            Self::reward_balance(&env),
        ))
    }

    fn require_initialized(env: &Env) -> Result<(), VaultError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            return Err(VaultError::NotInitialized);
        }
        Ok(())
    }

    fn balance(env: &Env, user: &Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(user.clone()))
            .unwrap_or(0)
    }

    fn total(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalDeposits)
            .unwrap_or(0)
    }

    fn reward_balance(env: &Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::RewardBalance)
            .unwrap_or(0)
    }

    fn claimable_reward(env: &Env, user: &Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::ClaimableReward(user.clone()))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup() -> (Env, VaultContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(VaultContract, ());
        let client = VaultContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin, &Address::generate(&env), &Address::generate(&env));
        (env, client, admin)
    }

    #[test]
    fn rejects_repeated_initialization() {
        let (env, client, admin) = setup();
        let result =
            client.try_initialize(&admin, &Address::generate(&env), &Address::generate(&env));
        assert_eq!(result.unwrap_err().unwrap(), VaultError::AlreadyInitialized);
    }

    #[test]
    fn rejects_uninitialized_usage() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(VaultContract, ());
        let client = VaultContractClient::new(&env, &contract_id);
        let user = Address::generate(&env);
        assert_eq!(
            client.try_deposit(&user, &0).unwrap_err().unwrap(),
            VaultError::NotInitialized
        );
    }

    #[test]
    fn rejects_invalid_amounts_and_insufficient_balance() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        assert_eq!(
            client.try_deposit(&user, &0).unwrap_err().unwrap(),
            VaultError::InvalidAmount
        );
        assert_eq!(
            client.try_withdraw(&user, &1).unwrap_err().unwrap(),
            VaultError::InsufficientBalance
        );
        client.deposit(&user, &10);
        assert_eq!(
            client.try_withdraw(&user, &11).unwrap_err().unwrap(),
            VaultError::InsufficientBalance
        );
        assert_eq!(
            client.try_withdraw(&user, &0).unwrap_err().unwrap(),
            VaultError::InvalidAmount
        );
    }

    #[test]
    fn tracks_deposits_and_claims_rewards() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        assert_eq!(client.deposit(&user, &10), 10);
        assert_eq!(client.total_deposits(), 10);
        assert_eq!(client.withdraw(&user, &4), 6);
        assert_eq!(client.total_deposits(), 6);

        // Claiming with no rewards returns zero
        assert_eq!(client.claim_rewards(&user), 0);

        // Set rewards and claim
        client.set_claimable_reward(&user, &100);
        assert_eq!(client.claim_rewards(&user), 100);

        // Reward balance should be cleared
        assert_eq!(client.claim_rewards(&user), 0);
    }

    #[test]
    fn pending_rewards_are_proportional_to_deposit_share() {
        let (env, client, _) = setup();
        let user_a = Address::generate(&env);
        let user_b = Address::generate(&env);

        client.deposit(&user_a, &500);
        client.deposit(&user_b, &500);
        client.set_reward_balance(&200);

        assert_eq!(client.pending_rewards(&user_a), 100);
        assert_eq!(client.pending_rewards(&user_b), 100);
    }

    #[test]
    fn pending_rewards_are_zero_when_total_deposits_are_zero() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        client.set_reward_balance(&200);
        assert_eq!(client.pending_rewards(&user), 0);
    }
}
