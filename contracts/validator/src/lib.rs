#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{contracttype, symbol_short, Env, Symbol, Vec};
use axionvera_state::*;
use axionvera_storage::{get_vault_state, get_staking_state, get_reward_state, get_treasury_state};
use axionvera_core::get_global_event_log;
use axionvera_accounting::validate_accounting;

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity { Critical, Major, Minor, Info }

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationStatus { Passed, Failed, Warning }

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleResult {
    pub name: Symbol,
    pub status: ValidationStatus,
    pub severity: Severity,
    pub message: Symbol,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReport {
    pub timestamp: u64,
    pub ledger: u32,
    pub overall: ValidationStatus,
    pub rules: Vec<RuleResult>,
    pub passed: u32,
    pub failed: u32,
    pub warnings: u32,
}

const R_VAULT_STAKING: Symbol = symbol_short!("vault_stk");
const R_VAULT_REWARD: Symbol = symbol_short!("vault_rwd");
const R_VAULT_TREASURY: Symbol = symbol_short!("vault_trs");
const R_REWARD_VAULT: Symbol = symbol_short!("rwd_vault");
const R_TREASURY_VAULT: Symbol = symbol_short!("treas_vlt");
const R_RESOURCE_INV: Symbol = symbol_short!("rsrc_inv");
const R_ACCOUNTING: Symbol = symbol_short!("acct_cons");
const R_EVENT_LOG: Symbol = symbol_short!("evt_log");
const MSG_OK: Symbol = symbol_short!("ok");
const MSG_FAIL: Symbol = symbol_short!("fail");
const MSG_WARN: Symbol = symbol_short!("warn");

// ---------------------------------------------------------------------------
// Cross-state consistency rules (no Env needed — pure logic)
// ---------------------------------------------------------------------------

/// Vault should be Active whenever staking is past Uninitialized.
pub fn rule_vault_staking_consistency(vault: VaultState, staking: StakingState) -> RuleResult {
    let ok = vault == VaultState::Active || staking == StakingState::Uninitialized;
    RuleResult {
        name: R_VAULT_STAKING,
        status: if ok { ValidationStatus::Passed } else { ValidationStatus::Failed },
        severity: Severity::Critical,
        message: if ok { MSG_OK } else { MSG_FAIL },
    }
}

/// If vault is terminated, treasury must be insolvent or emergency-restricted.
pub fn rule_vault_treasury_consistency(vault: VaultState, treasury: TreasuryState) -> RuleResult {
    let ok = vault != VaultState::Terminated
        || treasury == TreasuryState::Insolvent
        || treasury == TreasuryState::EmergencyRestricted;
    RuleResult {
        name: R_VAULT_TREASURY,
        status: if ok { ValidationStatus::Passed } else { ValidationStatus::Failed },
        severity: Severity::Critical,
        message: if ok { MSG_OK } else { MSG_FAIL },
    }
}

/// If reward is past Idle, vault must not be Uninitialized.
pub fn rule_reward_vault_consistency(vault: VaultState, reward: RewardState) -> RuleResult {
    let ok = reward == RewardState::Idle || vault != VaultState::Uninitialized;
    RuleResult {
        name: R_REWARD_VAULT,
        status: if ok { ValidationStatus::Passed } else { ValidationStatus::Failed },
        severity: Severity::Critical,
        message: if ok { MSG_OK } else { MSG_FAIL },
    }
}

/// If vault is paused, reward should not be Distributing.
pub fn rule_vault_reward_consistency(vault: VaultState, reward: RewardState) -> RuleResult {
    let ok = !(vault == VaultState::Paused && reward == RewardState::Distributing);
    RuleResult {
        name: R_VAULT_REWARD,
        status: if ok { ValidationStatus::Passed } else { ValidationStatus::Warning },
        severity: Severity::Major,
        message: if ok { MSG_OK } else { MSG_WARN },
    }
}

/// If treasury is emergency-restricted, vault must be paused or locked.
pub fn rule_treasury_vault_consistency(vault: VaultState, treasury: TreasuryState) -> RuleResult {
    let ok = treasury != TreasuryState::EmergencyRestricted
        || vault == VaultState::Paused
        || vault == VaultState::Locked;
    RuleResult {
        name: R_TREASURY_VAULT,
        status: if ok { ValidationStatus::Passed } else { ValidationStatus::Failed },
        severity: Severity::Critical,
        message: if ok { MSG_OK } else { MSG_FAIL },
    }
}

/// Resource list consistency: every ID in ResourceList must have a stored entry.
pub fn rule_resource_invariant(e: &Env) -> RuleResult {
    let ids = axionvera_storage::list_resources(e);
    let mut ok = true;
    for i in 0..ids.len() {
        if !axionvera_storage::resource_exists(e, &ids.get(i).unwrap()) {
            ok = false;
            break;
        }
    }
    RuleResult {
        name: R_RESOURCE_INV,
        status: if ok { ValidationStatus::Passed } else { ValidationStatus::Failed },
        severity: Severity::Major,
        message: if ok { MSG_OK } else { MSG_FAIL },
    }
}

/// Accounting consistency: total == sum of categories.
pub fn rule_accounting_consistency(e: &Env) -> RuleResult {
    let ok = validate_accounting(e);
    RuleResult {
        name: R_ACCOUNTING,
        status: if ok { ValidationStatus::Passed } else { ValidationStatus::Failed },
        severity: Severity::Critical,
        message: if ok { MSG_OK } else { MSG_FAIL },
    }
}

/// Event log bounds: global log should not exceed MAX_GLOBAL_EVENTS.
pub fn rule_event_log_invariant(e: &Env) -> RuleResult {
    let log = get_global_event_log(e);
    let ok = log.len() <= 200;
    RuleResult {
        name: R_EVENT_LOG,
        status: if ok { ValidationStatus::Passed } else { ValidationStatus::Failed },
        severity: Severity::Minor,
        message: if ok { MSG_OK } else { MSG_FAIL },
    }
}

// ---------------------------------------------------------------------------
// Composite validators
// ---------------------------------------------------------------------------

/// Run all pure-logic consistency rules given current states.
pub fn validate_consistency_rules(
    e: &Env,
    vault: VaultState, staking: StakingState,
    reward: RewardState, treasury: TreasuryState,
) -> Vec<RuleResult> {
    let mut v = Vec::new(e);
    v.push_back(rule_vault_staking_consistency(vault, staking));
    v.push_back(rule_vault_treasury_consistency(vault, treasury));
    v.push_back(rule_reward_vault_consistency(vault, reward));
    v.push_back(rule_vault_reward_consistency(vault, reward));
    v.push_back(rule_treasury_vault_consistency(vault, treasury));
    v
}

/// Run all storage-backed validation checks.
pub fn validate_storage_invariants(e: &Env) -> Vec<RuleResult> {
    let mut v = Vec::new(e);
    v.push_back(rule_resource_invariant(e));
    v.push_back(rule_accounting_consistency(e));
    v.push_back(rule_event_log_invariant(e));
    v
}

/// Generate a full validation report.
pub fn generate_report(e: &Env) -> ValidationReport {
    let mut all = Vec::new(e);
    let vault = get_vault_state(e);
    let staking = get_staking_state(e);
    let reward = get_reward_state(e);
    let treasury = get_treasury_state(e);

    for r in validate_consistency_rules(e, vault, staking, reward, treasury).into_iter() {
        all.push_back(r);
    }
    for r in validate_storage_invariants(e).into_iter() {
        all.push_back(r);
    }

    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut warnings = 0u32;
    for i in 0..all.len() {
        match all.get(i).unwrap().status {
            ValidationStatus::Passed => passed += 1,
            ValidationStatus::Failed => failed += 1,
            ValidationStatus::Warning => warnings += 1,
        }
    }
    let overall = if failed > 0 { ValidationStatus::Failed }
        else if warnings > 0 { ValidationStatus::Warning }
        else { ValidationStatus::Passed };

    ValidationReport { timestamp: e.ledger().timestamp(), ledger: e.ledger().sequence(), overall, rules: all, passed, failed, warnings }
}
