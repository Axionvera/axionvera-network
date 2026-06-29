#![no_std]

use soroban_sdk::{contracttype, Address, Bytes, Env, Symbol, Vec};

use axionvera_resources::{
    emit_resource_created_event, emit_resource_retired_event, emit_resource_transition_event,
    DataKey as ResourceDataKey, ResourceError, ResourceInfo, ResourceState,
};
use axionvera_state::{
    emit_state_transition, GovernanceState, RewardState, StateError, StakingState, TreasuryState,
    VaultState,
};

// ===========================================================================
// Event — module symbols for state machine events
// ===========================================================================

const MOD_VAULT: Symbol = Symbol::short!("vault");
const MOD_STAKING: Symbol = Symbol::short!("staking");
const MOD_REWARDS: Symbol = Symbol::short!("rewards");
const MOD_TREASURY: Symbol = Symbol::short!("treasury");
const MOD_GOV: Symbol = Symbol::short!("gov");

// ===========================================================================
// Storage Keys for Protocol State Machines
// ===========================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StateDataKey {
    VaultState,
    StakingState,
    RewardState,
    TreasuryState,
    GovernanceState(Symbol),
}

// ===========================================================================
// Vault State
// ===========================================================================

pub fn get_vault_state(e: &Env) -> VaultState {
    e.storage()
        .instance()
        .get(&StateDataKey::VaultState)
        .unwrap_or(VaultState::Uninitialized)
}

pub fn set_vault_state(
    e: &Env,
    new_state: VaultState,
    caller: Address,
) -> Result<VaultState, StateError> {
    let current = get_vault_state(e);
    let validated = current.transition(new_state)?;
    e.storage()
        .instance()
        .set(&StateDataKey::VaultState, &validated);

    emit_state_transition(e, MOD_VAULT, current as u32, validated as u32, caller);

    Ok(validated)
}

// ===========================================================================
// Staking State
// ===========================================================================

pub fn get_staking_state(e: &Env) -> StakingState {
    e.storage()
        .instance()
        .get(&StateDataKey::StakingState)
        .unwrap_or(StakingState::Uninitialized)
}

pub fn set_staking_state(
    e: &Env,
    new_state: StakingState,
    caller: Address,
) -> Result<StakingState, StateError> {
    let current = get_staking_state(e);
    let validated = current.transition(new_state)?;
    e.storage()
        .instance()
        .set(&StateDataKey::StakingState, &validated);

    emit_state_transition(e, MOD_STAKING, current as u32, validated as u32, caller);

    Ok(validated)
}

// ===========================================================================
// Reward State
// ===========================================================================

pub fn get_reward_state(e: &Env) -> RewardState {
    e.storage()
        .instance()
        .get(&StateDataKey::RewardState)
        .unwrap_or(RewardState::Idle)
}

pub fn set_reward_state(
    e: &Env,
    new_state: RewardState,
    caller: Address,
) -> Result<RewardState, StateError> {
    let current = get_reward_state(e);
    let validated = current.transition(new_state)?;
    e.storage()
        .instance()
        .set(&StateDataKey::RewardState, &validated);

    emit_state_transition(e, MOD_REWARDS, current as u32, validated as u32, caller);

    Ok(validated)
}

// ===========================================================================
// Treasury State
// ===========================================================================

pub fn get_treasury_state(e: &Env) -> TreasuryState {
    e.storage()
        .instance()
        .get(&StateDataKey::TreasuryState)
        .unwrap_or(TreasuryState::Normal)
}

pub fn set_treasury_state(
    e: &Env,
    new_state: TreasuryState,
    caller: Address,
) -> Result<TreasuryState, StateError> {
    let current = get_treasury_state(e);
    let validated = current.transition(new_state)?;
    e.storage()
        .instance()
        .set(&StateDataKey::TreasuryState, &validated);

    emit_state_transition(
        e,
        MOD_TREASURY,
        current as u32,
        validated as u32,
        caller,
    );

    Ok(validated)
}

// ===========================================================================
// Governance State
// ===========================================================================

pub fn get_governance_state(e: &Env, proposal_id: Symbol) -> GovernanceState {
    e.storage()
        .persistent()
        .get(&StateDataKey::GovernanceState(proposal_id))
        .unwrap_or(GovernanceState::Draft)
}

pub fn set_governance_state(
    e: &Env,
    proposal_id: Symbol,
    new_state: GovernanceState,
    caller: Address,
) -> Result<GovernanceState, StateError> {
    let current = get_governance_state(e, proposal_id.clone());
    let validated = current.transition(new_state)?;
    e.storage()
        .persistent()
        .set(
            &StateDataKey::GovernanceState(proposal_id.clone()),
            &validated,
        );

    emit_state_transition(e, MOD_GOV, current as u32, validated as u32, caller);

    Ok(validated)
}

// ===========================================================================
// Resource Lifecycle — Persistence
// ===========================================================================

/// Create a new resource in storage.
pub fn create_resource(
    e: &Env,
    resource_id: &Symbol,
    caller: &Address,
    metadata: Option<Bytes>,
) -> Result<ResourceInfo, ResourceError> {
    let key = ResourceDataKey::Resource(resource_id.clone());
    if e.storage().persistent().has(&key) {
        return Err(ResourceError::AlreadyExists);
    }

    let now = e.ledger().timestamp();
    let info = ResourceInfo {
        id: resource_id.clone(),
        state: ResourceState::Created,
        created_at: now,
        updated_at: now,
        metadata,
    };

    e.storage().persistent().set(&key, &info);
    add_resource_to_list(e, resource_id);

    emit_resource_created_event(e, resource_id, caller);

    Ok(info)
}

/// Transition a resource to a new state, validating the transition.
pub fn transition_resource(
    e: &Env,
    resource_id: &Symbol,
    next_state: ResourceState,
    caller: &Address,
) -> Result<ResourceInfo, ResourceError> {
    let key = ResourceDataKey::Resource(resource_id.clone());
    let mut info: ResourceInfo = e
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ResourceError::NotFound)?;

    let old_state = info.state;
    info.state = old_state.transition(next_state)?;
    info.updated_at = e.ledger().timestamp();

    e.storage().persistent().set(&key, &info);

    emit_resource_transition_event(e, old_state, info.state, resource_id, caller);

    if info.state == ResourceState::Retired {
        emit_resource_retired_event(e, resource_id, caller);
    }

    Ok(info)
}

/// Get the current state of a resource.
pub fn get_resource_state(e: &Env, resource_id: &Symbol) -> Result<ResourceState, ResourceError> {
    let key = ResourceDataKey::Resource(resource_id.clone());
    let info: ResourceInfo = e
        .storage()
        .persistent()
        .get(&key)
        .ok_or(ResourceError::NotFound)?;
    Ok(info.state)
}

/// Get full resource info.
pub fn get_resource_info(e: &Env, resource_id: &Symbol) -> Result<ResourceInfo, ResourceError> {
    let key = ResourceDataKey::Resource(resource_id.clone());
    e.storage()
        .persistent()
        .get(&key)
        .ok_or(ResourceError::NotFound)
}

/// Check if a resource exists.
pub fn resource_exists(e: &Env, resource_id: &Symbol) -> bool {
    let key = ResourceDataKey::Resource(resource_id.clone());
    e.storage().persistent().has(&key)
}

/// List all registered resource IDs.
pub fn list_resources(e: &Env) -> Vec<Symbol> {
    e.storage()
        .persistent()
        .get(&ResourceDataKey::ResourceList)
        .unwrap_or_else(|| Vec::new(e))
}

/// Count registered resources.
pub fn resource_count(e: &Env) -> u32 {
    list_resources(e).len()
}

// -----------------------------------------------------------------------
// Internal helpers
// -----------------------------------------------------------------------

fn add_resource_to_list(e: &Env, resource_id: &Symbol) {
    let key = ResourceDataKey::ResourceList;
    let mut list: Vec<Symbol> = e
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(e));
    if !list.contains(resource_id.clone()) {
        list.push_back(resource_id.clone());
        e.storage().persistent().set(&key, &list);
    }
}
