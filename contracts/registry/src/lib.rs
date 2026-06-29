#![no_std]

mod storage;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec};
use axionvera_events as events;

#[contract]
pub struct RegistryContract;

#[contractimpl]
impl RegistryContract {
    pub fn initialize(e: Env, admin: Address) {
        if storage::is_initialized(&e) {
            panic!("already initialized");
        }
        admin.require_auth();
        storage::set_admin(&e, &admin);
        e.storage().instance().set(&storage::DataKey::Initialized, &true);
    }

    pub fn register_module(e: Env, name: Symbol, module_address: Address) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        if storage::has_module_name(&e, name.clone()) {
            panic!("module name already registered");
        }

        if storage::has_module_address(&e, &module_address) {
            panic!("module address already registered");
        }

        storage::set_module_address(&e, name.clone(), &module_address);
        storage::set_module_status(&e, &module_address, true);
        storage::add_to_all_modules(&e, &module_address);

        // Emit event
        let timestamp = e.ledger().timestamp();
        e.events().publish(
            (events::PROTOCOL, events::ACT_MOD_REGISTER),
            events::ModuleRegisteredEvent {
                admin,
                name,
                module_address,
                timestamp,
            },
        );
    }

    pub fn set_module_status(e: Env, module_address: Address, is_active: bool) {
        let admin = storage::get_admin(&e);
        admin.require_auth();

        storage::set_module_status(&e, &module_address, is_active);

        let timestamp = e.ledger().timestamp();
        e.events().publish(
            (events::PROTOCOL, events::ACT_MOD_STATUS_UPDATE),
            events::ModuleStatusChangedEvent {
                admin,
                module_address,
                is_active,
                timestamp,
            },
        );
    }

    pub fn get_module_address(e: Env, name: Symbol) -> Option<Address> {
        storage::get_module_address(&e, name)
    }

    pub fn get_module_status(e: Env, module_address: Address) -> Option<bool> {
        storage::get_module_status(&e, &module_address)
    }

    pub fn is_module_active(e: Env, module_address: Address) -> bool {
        storage::get_module_status(&e, &module_address).unwrap_or(false)
    }

    pub fn list_modules(e: Env) -> Vec<Address> {
        storage::get_all_modules(&e)
    }
}
