#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};

#[contract]
pub struct AdminContract;

#[contractimpl]
impl AdminContract {
    /// Admin entry point to trigger protocol pause
    pub fn trigger_emergency_pause(env: Env, security_contract: Address) {
        // Enforce admin auth here if needed, then call security contract
        env.invoke_contract::<()>(
            &security_contract,
            &soroban_sdk::Symbol::new(&env, "pause"),
            soroban_sdk::vec![&env]
        );
    }
    
    /// Admin entry point to recover protocol
    pub fn trigger_recovery(env: Env, security_contract: Address) {
        env.invoke_contract::<()>(
            &security_contract,
            &soroban_sdk::Symbol::new(&env, "unpause"),
            soroban_sdk::vec![&env]
        );
    }
}