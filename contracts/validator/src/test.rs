#![cfg(test)]

use crate::*;
use axionvera_storage::{set_reward_state, set_staking_state, set_treasury_state, set_vault_state};
use soroban_sdk::{Address, Env, testutils::Address as _};

// -----------------------------------------------------------------------
// Pure-logic consistency rule tests (no contract env needed beyond default)
// -----------------------------------------------------------------------

#[test]
fn vault_staking_consistency_active_ok() {
    let r = rule_vault_staking_consistency(VaultState::Active, StakingState::Active);
    assert_eq!(r.status, ValidationStatus::Passed);
}

#[test]
fn vault_staking_consistency_terminal_with_staking_fails() {
    let r = rule_vault_staking_consistency(VaultState::Terminated, StakingState::Active);
    assert_eq!(r.status, ValidationStatus::Failed);
}

#[test]
fn vault_staking_consistency_paused_uninitialized_ok() {
    let r = rule_vault_staking_consistency(VaultState::Paused, StakingState::Uninitialized);
    assert_eq!(r.status, ValidationStatus::Passed);
}

#[test]
fn vault_treasury_consistency_terminal_insolvent_ok() {
    let r = rule_vault_treasury_consistency(VaultState::Terminated, TreasuryState::Insolvent);
    assert_eq!(r.status, ValidationStatus::Passed);
}

#[test]
fn vault_treasury_consistency_terminal_emergency_ok() {
    let r =
        rule_vault_treasury_consistency(VaultState::Terminated, TreasuryState::EmergencyRestricted);
    assert_eq!(r.status, ValidationStatus::Passed);
}

#[test]
fn vault_treasury_consistency_terminal_normal_fails() {
    let r = rule_vault_treasury_consistency(VaultState::Terminated, TreasuryState::Normal);
    assert_eq!(r.status, ValidationStatus::Failed);
}

#[test]
fn reward_vault_consistency_accruing_active_ok() {
    let r = rule_reward_vault_consistency(VaultState::Active, RewardState::Accruing);
    assert_eq!(r.status, ValidationStatus::Passed);
}

#[test]
fn reward_vault_consistency_non_idle_uninitialized_fails() {
    let r = rule_reward_vault_consistency(VaultState::Uninitialized, RewardState::Accruing);
    assert_eq!(r.status, ValidationStatus::Failed);
}

#[test]
fn vault_reward_consistency_paused_distributing_warning() {
    let r = rule_vault_reward_consistency(VaultState::Paused, RewardState::Distributing);
    assert_eq!(r.status, ValidationStatus::Warning);
}

#[test]
fn vault_reward_consistency_paused_idle_ok() {
    let r = rule_vault_reward_consistency(VaultState::Paused, RewardState::Idle);
    assert_eq!(r.status, ValidationStatus::Passed);
}

#[test]
fn treasury_vault_consistency_emergency_locked_ok() {
    let r = rule_treasury_vault_consistency(VaultState::Locked, TreasuryState::EmergencyRestricted);
    assert_eq!(r.status, ValidationStatus::Passed);
}

#[test]
fn treasury_vault_consistency_emergency_active_fails() {
    let r = rule_treasury_vault_consistency(VaultState::Active, TreasuryState::EmergencyRestricted);
    assert_eq!(r.status, ValidationStatus::Failed);
}

#[test]
fn all_pure_rules_pass_on_default_states() {
    let e = Env::default();
    let results = validate_consistency_rules(
        &e,
        VaultState::Uninitialized,
        StakingState::Uninitialized,
        RewardState::Idle,
        TreasuryState::Normal,
    );
    for i in 0..results.len() {
        assert_eq!(
            results.get(i).unwrap().status,
            ValidationStatus::Passed,
            "rule {:?} should pass on default states",
            results.get(i).unwrap().name
        );
    }
}

#[test]
fn detect_mixed_inconsistencies() {
    let e = Env::default();
    let results = validate_consistency_rules(
        &e,
        VaultState::Terminated,
        StakingState::Active,
        RewardState::Accruing,
        TreasuryState::Normal,
    );
    let mut fail = 0;
    for i in 0..results.len() {
        match results.get(i).unwrap().status {
            ValidationStatus::Passed => (),
            ValidationStatus::Failed => fail += 1,
            ValidationStatus::Warning => (),
        }
    }
    assert!(
        fail >= 2,
        "should detect vault-terminated + staking-active + treasury-normal inconsistencies"
    );
}

// -----------------------------------------------------------------------
// Storage-backed tests (need registered contract + env)
// -----------------------------------------------------------------------

#[soroban_sdk::contract]
pub struct ValidatorHarness;

#[soroban_sdk::contractimpl]
impl ValidatorHarness {
    pub fn noop() {}
}

fn with_env<F: FnOnce(&Env)>(
    vault: VaultState,
    staking: StakingState,
    reward: RewardState,
    treasury: TreasuryState,
    f: F,
) {
    let e = Env::default();
    let caller = Address::generate(&e);
    let id = e.register(ValidatorHarness, ());
    e.as_contract(&id, || {
        set_vault_state(&e, vault, caller.clone()).ok();
        set_staking_state(&e, staking, caller.clone()).ok();
        set_reward_state(&e, reward, caller.clone()).ok();
        set_treasury_state(&e, treasury, caller).ok();
        f(&e);
    });
}

#[test]
fn storage_backed_rules_on_default_states() {
    with_env(
        VaultState::Uninitialized,
        StakingState::Uninitialized,
        RewardState::Idle,
        TreasuryState::Normal,
        |e| {
            let report = generate_report(e);
            assert_eq!(report.overall, ValidationStatus::Passed);
            assert!(report.passed >= report.rules.len() - report.warnings);
        },
    );
}

#[test]
fn report_detects_inconsistencies() {
    with_env(
        VaultState::Terminated,
        StakingState::Active,
        RewardState::Accruing,
        TreasuryState::Normal,
        |e| {
            let report = generate_report(e);
            assert_eq!(report.overall, ValidationStatus::Failed);
            assert!(report.failed > 0);
        },
    );
}

#[test]
fn rule_names_are_present_in_report() {
    with_env(
        VaultState::Active,
        StakingState::Active,
        RewardState::Accruing,
        TreasuryState::Normal,
        |e| {
            let report = generate_report(e);
            let expected = [
                symbol_short!("vault_stk"),
                symbol_short!("vault_trs"),
                symbol_short!("rwd_vault"),
                symbol_short!("vault_rwd"),
                symbol_short!("treas_vlt"),
                symbol_short!("rsrc_inv"),
                symbol_short!("acct_cons"),
                symbol_short!("evt_log"),
            ];
            let mut found_count = 0u32;
            for ei in 0..expected.len() {
                for ri in 0..report.rules.len() {
                    if report.rules.get(ri).unwrap().name == expected[ei] {
                        found_count += 1;
                        break;
                    }
                }
            }
            assert_eq!(
                found_count,
                expected.len() as u32,
                "all 8 rule names must appear"
            );
        },
    );
}
