#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Env};

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
            .set(&DataKey::Balance(from), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalDeposits, &new_total);
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
        let new_balance = balance - amount;
        let new_total = Self::total(&env) - amount;
        env.storage()
            .persistent()
            .set(&DataKey::Balance(to), &new_balance);
        env.storage()
            .instance()
            .set(&DataKey::TotalDeposits, &new_total);
        Ok(new_balance)
    }

    pub fn claim_rewards(env: Env, user: Address) -> Result<i128, VaultError> {
        Self::require_initialized(&env)?;
        user.require_auth();
        let reward_balance = Self::reward_balance(&env);
        if reward_balance <= 0 {
            return Err(VaultError::InvalidRewardState);
        }
        env.storage()
            .instance()
            .set(&DataKey::RewardBalance, &0_i128);
        Ok(reward_balance)
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
    fn tracks_deposits_and_rejects_empty_reward_claim() {
        let (env, client, _) = setup();
        let user = Address::generate(&env);
        assert_eq!(client.deposit(&user, &10), 10);
        assert_eq!(client.total_deposits(), 10);
        assert_eq!(client.withdraw(&user, &4), 6);
        assert_eq!(client.total_deposits(), 6);
        assert_eq!(
            client.try_claim_rewards(&user).unwrap_err().unwrap(),
            VaultError::InvalidRewardState
        );
    }
}
