#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn initialize(_env: Env, _admin: Address, _deposit_token: Address, _reward_token: Address) {
        // Initial clean scaffold.
    }

    pub fn deposit(_env: Env, _from: Address, amount: i128) -> i128 {
        amount
    }

    pub fn withdraw(_env: Env, _to: Address, amount: i128) -> i128 {
        amount
    }

    pub fn claim_rewards(_env: Env, _user: Address) -> i128 {
        0
    }

    pub fn total_deposits(_env: Env) -> i128 {
        0
    }
}
