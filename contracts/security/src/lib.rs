#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, symbol_short};

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Admin,
    Paused,
}

const INSTANCE_TTL: u32 = 518400;

#[contract]
pub struct EmergencyPause;

#[contractimpl]
impl EmergencyPause {
    pub fn init(e: Env, admin: Address) {
        if e.storage().instance().has(&DataKey::Admin) { panic!("Already initialized"); }
        e.storage().instance().set(&DataKey::Admin, &admin);
        e.storage().instance().set(&DataKey::Paused, &false);
        e.storage().instance().extend_ttl(INSTANCE_TTL, INSTANCE_TTL);
    }

    pub fn pause(e: Env, caller: Address) {
        caller.require_auth();
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != admin { panic!("Not authorized"); }
        e.storage().instance().set(&DataKey::Paused, &true);
        e.events().publish((symbol_short!("pause"),), symbol_short!("paused"));
    }

    pub fn unpause(e: Env, caller: Address) {
        caller.require_auth();
        let admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        if caller != admin { panic!("Not authorized"); }
        e.storage().instance().set(&DataKey::Paused, &false);
        e.events().publish((symbol_short!("pause"),), symbol_short!("unpaused"));
    }

    pub fn is_paused(e: Env) -> bool {
        e.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }

    pub fn admin(e: Env) -> Address {
        e.storage().instance().get(&DataKey::Admin).unwrap()
    }
}

#[contract]
pub struct SecurityContract;

#[contractimpl]
impl SecurityContract {
    /// Initializes the security contract with an admin address.
    pub fn init(env: Env, admin: Address) {
        assert!(!env.storage().instance().has(&DataKey::Admin), "Already initialized");
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::IsPaused, &false);
    }

    /// Pauses all critical protocol functions. Only accessible by Admin.
    pub fn pause(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Not initialized");
        admin.require_auth();
        
        env.storage().instance().set(&DataKey::IsPaused, &true);
        env.events().publish((symbol_short!("security"), symbol_short!("pause")), true);
    }

    /// Unpauses protocol functions. Only accessible by Admin.
    pub fn unpause(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Not initialized");
        admin.require_auth();
        
        env.storage().instance().set(&DataKey::IsPaused, &false);
        env.events().publish((symbol_short!("security"), symbol_short!("unpause")), false);
    }

    /// Read-only check for the current pause state.
    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get(&DataKey::IsPaused).unwrap_or(false)
    }
}

mod test;