use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Initialized,
    ModuleAddress(Symbol),
    ModuleStatus(Address),
    AllModules,
}

pub fn is_initialized(e: &Env) -> bool {
    e.storage()
        .instance()
        .get::<_, bool>(&DataKey::Initialized)
        .unwrap_or(false)
}

pub fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_admin(e: &Env, admin: &Address) {
    e.storage().instance().set(&DataKey::Admin, admin);
}

pub fn get_module_address(e: &Env, name: Symbol) -> Option<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::ModuleAddress(name))
}

pub fn set_module_address(e: &Env, name: Symbol, address: &Address) {
    e.storage()
        .persistent()
        .set(&DataKey::ModuleAddress(name), address);
}

pub fn get_module_status(e: &Env, address: &Address) -> Option<bool> {
    e.storage()
        .persistent()
        .get(&DataKey::ModuleStatus(address.clone()))
}

pub fn set_module_status(e: &Env, address: &Address, is_active: bool) {
    e.storage()
        .persistent()
        .set(&DataKey::ModuleStatus(address.clone()), &is_active);
}

pub fn get_all_modules(e: &Env) -> Vec<Address> {
    e.storage()
        .persistent()
        .get(&DataKey::AllModules)
        .unwrap_or_else(|| Vec::new(e))
}

pub fn add_to_all_modules(e: &Env, address: &Address) {
    let mut modules = get_all_modules(e);
    if !modules.contains(address) {
        modules.push_back(address.clone());
        e.storage()
            .persistent()
            .set(&DataKey::AllModules, &modules);
    }
}

pub fn has_module_name(e: &Env, name: Symbol) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::ModuleAddress(name))
}

pub fn has_module_address(e: &Env, address: &Address) -> bool {
    e.storage()
        .persistent()
        .has(&DataKey::ModuleStatus(address.clone()))
}
