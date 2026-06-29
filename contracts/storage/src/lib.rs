use soroban_sdk::{Address, Env, Symbol};
use axionvera_state::{GovernanceState, RewardState, StakingState, TreasuryState, VaultState};

pub fn get_vault_state(_e: &Env) -> VaultState { VaultState::Active }
pub fn set_vault_state(_e: &Env, _s: VaultState, _c: Address) -> Result<VaultState, axionvera_state::StateError> { Ok(VaultState::Active) }
pub fn get_staking_state(_e: &Env) -> StakingState { StakingState::Active }
pub fn set_staking_state(_e: &Env, _s: StakingState, _c: Address) -> Result<StakingState, axionvera_state::StateError> { Ok(StakingState::Active) }
pub fn get_reward_state(_e: &Env) -> RewardState { RewardState::Idle }
pub fn set_reward_state(_e: &Env, _s: RewardState, _c: Address) -> Result<RewardState, axionvera_state::StateError> { Ok(RewardState::Idle) }
pub fn get_treasury_state(_e: &Env) -> TreasuryState { TreasuryState::Normal }
pub fn set_treasury_state(_e: &Env, _s: TreasuryState, _c: Address) -> Result<TreasuryState, axionvera_state::StateError> { Ok(TreasuryState::Normal) }
pub fn get_governance_state(_e: &Env, _id: Symbol) -> GovernanceState { GovernanceState::Draft }
pub fn set_governance_state(_e: &Env, _id: Symbol, _s: GovernanceState, _c: Address) -> Result<GovernanceState, axionvera_state::StateError> { Ok(GovernanceState::Draft) }
