#![no_std]

use soroban_sdk::{contracttype, Address, Env, Symbol};
use axionvera_state::{
    emit_state_transition, GovernanceState, RewardState, StateError, StakingState, TreasuryState,
    VaultState,
};

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    VaultState,
    StakingState,
    RewardState,
    TreasuryState,
    GovernanceState(Symbol),
}

pub fn get_vault_state(e: &Env) -> VaultState {
    e.storage()
        .instance()
        .get(&DataKey::VaultState)
        .unwrap_or(VaultState::Uninitialized)
}

pub fn set_vault_state(e: &Env, new_state: VaultState, caller: Address) -> Result<VaultState, StateError> {
    let old_state = get_vault_state(e);
    let state = old_state.transition(new_state)?;
    e.storage().instance().set(&DataKey::VaultState, &state);
    emit_state_transition(e, Symbol::new(e, "Vault"), old_state as u32, state as u32, caller);
    Ok(state)
}

pub fn get_staking_state(e: &Env) -> StakingState {
    e.storage()
        .instance()
        .get(&DataKey::StakingState)
        .unwrap_or(StakingState::Uninitialized)
}

pub fn set_staking_state(e: &Env, new_state: StakingState, caller: Address) -> Result<StakingState, StateError> {
    let old_state = get_staking_state(e);
    let state = old_state.transition(new_state)?;
    e.storage().instance().set(&DataKey::StakingState, &state);
    emit_state_transition(e, Symbol::new(e, "Staking"), old_state as u32, state as u32, caller);
    Ok(state)
}

pub fn get_reward_state(e: &Env) -> RewardState {
    e.storage()
        .instance()
        .get(&DataKey::RewardState)
        .unwrap_or(RewardState::Idle)
}

pub fn set_reward_state(e: &Env, new_state: RewardState, caller: Address) -> Result<RewardState, StateError> {
    let old_state = get_reward_state(e);
    let state = old_state.transition(new_state)?;
    e.storage().instance().set(&DataKey::RewardState, &state);
    emit_state_transition(e, Symbol::new(e, "Reward"), old_state as u32, state as u32, caller);
    Ok(state)
}

pub fn get_treasury_state(e: &Env) -> TreasuryState {
    e.storage()
        .instance()
        .get(&DataKey::TreasuryState)
        .unwrap_or(TreasuryState::Normal)
}

pub fn set_treasury_state(e: &Env, new_state: TreasuryState, caller: Address) -> Result<TreasuryState, StateError> {
    let old_state = get_treasury_state(e);
    let state = old_state.transition(new_state)?;
    e.storage().instance().set(&DataKey::TreasuryState, &state);
    emit_state_transition(e, Symbol::new(e, "Treasury"), old_state as u32, state as u32, caller);
    Ok(state)
}

pub fn get_governance_state(e: &Env, proposal_id: Symbol) -> GovernanceState {
    e.storage()
        .persistent()
        .get(&DataKey::GovernanceState(proposal_id))
        .unwrap_or(GovernanceState::Draft)
}

pub fn set_governance_state(
    e: &Env,
    proposal_id: Symbol,
    new_state: GovernanceState,
    caller: Address,
) -> Result<GovernanceState, StateError> {
    let old_state = get_governance_state(e, proposal_id.clone());
    let state = old_state.transition(new_state)?;
    e.storage()
        .persistent()
        .set(&DataKey::GovernanceState(proposal_id), &state);
    emit_state_transition(e, Symbol::new(e, "Governance"), old_state as u32, state as u32, caller);
    Ok(state)
}
