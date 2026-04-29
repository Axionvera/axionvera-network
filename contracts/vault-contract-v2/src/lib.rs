#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env};

/// Same DataKey layout as V1 so storage is compatible after upgrade.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Initialized,
    Admin,
    DepositToken,
    RewardToken,
    TotalDeposits,
    RewardIndex,
    UserBalance(Address),
    UserRewardIndex(Address),
    UserRewards(Address),
}

const INSTANCE_TTL_THRESHOLD: u32 = 100;
const INSTANCE_TTL_EXTEND_TO: u32 = 1_000;
const PERSISTENT_TTL_THRESHOLD: u32 = 100;
const PERSISTENT_TTL_EXTEND_TO: u32 = 10_000;

#[contract]
pub struct VaultContractV2;

#[contractimpl]
impl VaultContractV2 {
    /// V2 returns version 2 to distinguish itself from V1.
    pub fn version(_e: Env) -> u32 {
        2
    }

    /// Reads the admin from storage (same key as V1).
    pub fn admin(e: Env) -> Address {
        e.storage()
            .instance()
            .get::<_, Address>(&DataKey::Admin)
            .unwrap()
    }

    /// Reads a user balance from storage (same key as V1).
    pub fn balance(e: Env, user: Address) -> i128 {
        let key = DataKey::UserBalance(user);
        e.storage().persistent().get(&key).unwrap_or(0_i128)
    }

    /// Reads total deposits from storage (same key as V1).
    pub fn total_deposits(e: Env) -> i128 {
        e.storage()
            .instance()
            .get(&DataKey::TotalDeposits)
            .unwrap_or(0_i128)
    }

    /// V2-only function that was not available in V1.
    /// Demonstrates that new functionality can be added after upgrade.
    pub fn v2_greeting(_e: Env) -> soroban_sdk::Symbol {
        soroban_sdk::symbol_short!("hello")
    }

    /// V2 also supports upgrade so the contract can be upgraded again.
    pub fn upgrade(e: Env, admin: Address, new_wasm_hash: BytesN<32>) {
        admin.require_auth();
        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}

fn _bump_instance_ttl(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}

fn _bump_persistent_ttl(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
}
