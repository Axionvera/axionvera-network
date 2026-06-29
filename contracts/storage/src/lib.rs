#![no_std]

use soroban_sdk::{contracttype, Address, Env, Symbol};
use axionvera_state::{VaultState, StakingState, RewardState, TreasuryState, GovernanceState};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
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

pub fn set_vault_state(e: &Env, state: VaultState, _caller: Address) -> Result<VaultState, axionvera_state::StateError> {
    e.storage().instance().set(&DataKey::VaultState, &state);
    Ok(state)
}

pub fn get_staking_state(e: &Env) -> StakingState {
    e.storage()
        .instance()
        .get(&DataKey::StakingState)
        .unwrap_or(StakingState::Uninitialized)
}

pub fn set_staking_state(e: &Env, state: StakingState, _caller: Address) -> Result<StakingState, axionvera_state::StateError> {
    e.storage().instance().set(&DataKey::StakingState, &state);
    Ok(state)
}

pub fn get_reward_state(e: &Env) -> RewardState {
    e.storage()
        .instance()
        .get(&DataKey::RewardState)
        .unwrap_or(RewardState::Idle)
}

pub fn set_reward_state(e: &Env, state: RewardState, _caller: Address) -> Result<RewardState, axionvera_state::StateError> {
    e.storage().instance().set(&DataKey::RewardState, &state);
    Ok(state)
}

pub fn get_treasury_state(e: &Env) -> TreasuryState {
    e.storage()
        .instance()
        .get(&DataKey::TreasuryState)
        .unwrap_or(TreasuryState::Normal)
}

pub fn set_treasury_state(e: &Env, state: TreasuryState, _caller: Address) -> Result<TreasuryState, axionvera_state::StateError> {
    e.storage().instance().set(&DataKey::TreasuryState, &state);
    Ok(state)
}

pub fn get_governance_state(e: &Env, proposal_id: Symbol) -> GovernanceState {
    e.storage()
        .instance()
        .get(&DataKey::GovernanceState(proposal_id))
        .unwrap_or(GovernanceState::Draft)
}

pub fn set_governance_state(e: &Env, proposal_id: Symbol, state: GovernanceState, _caller: Address) -> Result<GovernanceState, axionvera_state::StateError> {
    e.storage().instance().set(&DataKey::GovernanceState(proposal_id), &state);
    Ok(state)
}
