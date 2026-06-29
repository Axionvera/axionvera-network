#![cfg(test)]

use crate::{RegistryContract, RegistryContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, Symbol};

#[test]
fn test_register_and_list() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.initialize(&admin);
    
    let module_addr = Address::generate(&env);
    let name = Symbol::new(&env, "VaultV1");
    
    client.register_module(&name, &module_addr);
    
    let listed_addr = client.get_module_address(&name);
    assert_eq!(listed_addr, Some(module_addr.clone()));
    
    let is_active = client.is_module_active(&module_addr);
    assert_eq!(is_active, true);
    
    let modules = client.list_modules();
    assert_eq!(modules.len(), 1);
    assert_eq!(modules.get(0).unwrap(), module_addr);
}

#[test]
fn test_set_status() {
    let env = Env::default();
    env.mock_all_auths();
    
    let contract_id = env.register_contract(None, RegistryContract);
    let client = RegistryContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.initialize(&admin);
    
    let module_addr = Address::generate(&env);
    let name = Symbol::new(&env, "VaultV1");
    
    client.register_module(&name, &module_addr);
    
    // Pause module
    client.set_module_status(&module_addr, &false);
    let is_active = client.is_module_active(&module_addr);
    assert_eq!(is_active, false);
    
    let status = client.get_module_status(&module_addr);
    assert_eq!(status, Some(false));
}
