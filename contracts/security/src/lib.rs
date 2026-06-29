#![no_std]

use core::marker::PhantomData;
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, symbol_short};
use axionvera_auth::{AccessPolicy, PolicyViolation};

// --- Access Policy Framework ---

pub struct Authenticated<F, Context> {
    get_actor: F,
    _context: PhantomData<Context>,
}

impl<F, Context> Authenticated<F, Context>
where
    F: Fn(&Context) -> Address,
{
    pub fn new(get_actor: F) -> Self {
        Self { get_actor, _context: PhantomData }
    }
}

impl<F, Context> AccessPolicy<Context> for Authenticated<F, Context>
where
    F: Fn(&Context) -> Address,
{
    fn enforce(&self, context: &Context) -> Result<(), PolicyViolation> {
        (self.get_actor)(context).require_auth();
        Ok(())
    }
}

pub struct MatchAddress<F1, F2, Context> {
    get_a: F1,
    get_b: F2,
    violation: PolicyViolation,
    _context: PhantomData<Context>,
}

impl<F1, F2, Context> MatchAddress<F1, F2, Context>
where
    F1: Fn(&Context) -> Address,
    F2: Fn(&Context) -> Address,
{
    pub fn new(get_a: F1, get_b: F2, violation: PolicyViolation) -> Self {
        Self { get_a, get_b, violation, _context: PhantomData }
    }
}

impl<F1, F2, Context> AccessPolicy<Context> for MatchAddress<F1, F2, Context>
where
    F1: Fn(&Context) -> Address,
    F2: Fn(&Context) -> Address,
{
    fn enforce(&self, context: &Context) -> Result<(), PolicyViolation> {
        if (self.get_a)(context) == (self.get_b)(context) {
            Ok(())
        } else {
            Err(self.violation)
        }
    }
}

pub struct PredicatePolicy<F, Context> {
    predicate: F,
    violation: PolicyViolation,
    _context: PhantomData<Context>,
}

impl<F, Context> PredicatePolicy<F, Context>
where
    F: Fn(&Context) -> bool,
{
    pub fn new(predicate: F, violation: PolicyViolation) -> Self {
        Self { predicate, violation, _context: PhantomData }
    }
}

impl<F, Context> AccessPolicy<Context> for PredicatePolicy<F, Context>
where
    F: Fn(&Context) -> bool,
{
    fn enforce(&self, context: &Context) -> Result<(), PolicyViolation> {
        if (self.predicate)(context) {
            Ok(())
        } else {
            Err(self.violation)
        }
    }
}

// --- Emergency Pause Contract ---

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
