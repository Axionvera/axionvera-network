use axionvera_accounting::{
    AccountingCategory, AccountingEntry, AccountingOperation, OperationResources,
    record_operation,
};
use axionvera_state::{RewardState, StakingState, TreasuryState, VaultState};
use axionvera_storage::{
    set_reward_state, set_staking_state, set_treasury_state, set_vault_state,
};
use axionvera_validator::*;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger},
    Address, Env,
};

// -----------------------------------------------------------------------
// Harness contract for storage-backed tests
// -----------------------------------------------------------------------

#[soroban_sdk::contract]
pub struct InvariantHarness;

#[soroban_sdk::contractimpl]
impl InvariantHarness {
    pub fn noop() {}
}

fn harness(e: &Env) {
    let id = e.register(InvariantHarness, ());
    e.as_contract(&id, || {});
}

// -----------------------------------------------------------------------
// Pure consistency tests
// -----------------------------------------------------------------------

#[test]
fn consistency_default_states_pass() {
    let e = Env::default();
    let results = validate_consistency_rules(&e,
        VaultState::Uninitialized,
        StakingState::Uninitialized,
        RewardState::Idle,
        TreasuryState::Normal,
    );
    for i in 0..results.len() {
        assert_eq!(
            results.get(i).unwrap().status,
            ValidationStatus::Passed
        );
    }
}

#[test]
fn consistency_active_vault_with_staking_fails() {
    let e = Env::default();
    let results = validate_consistency_rules(&e,
        VaultState::Paused,
        StakingState::Active,
        RewardState::Idle,
        TreasuryState::Normal,
    );
    let vault_staking = results
        .iter()
        .find(|r| r.name == symbol_short!("vault_stk"))
        .unwrap();
    assert_eq!(vault_staking.status, ValidationStatus::Failed);
}

#[test]
fn consistency_terminated_vault_with_normal_treasury_fails() {
    let e = Env::default();
    let results = validate_consistency_rules(&e,
        VaultState::Terminated,
        StakingState::Uninitialized,
        RewardState::Idle,
        TreasuryState::Normal,
    );
    let vault_treasury = results
        .iter()
        .find(|r| r.name == symbol_short!("vault_trs"))
        .unwrap();
    assert_eq!(vault_treasury.status, ValidationStatus::Failed);
}

#[test]
fn consistency_detects_multiple_violations() {
    let e = Env::default();
    let results = validate_consistency_rules(&e,
        VaultState::Paused,
        StakingState::Active,
        RewardState::Distributing,
        TreasuryState::Normal,
    );
    let mut failed = 0u32;
    for i in 0..results.len() {
        if results.get(i).unwrap().status == ValidationStatus::Failed {
            failed += 1;
        }
    }
    assert!(failed >= 1);
}

// -----------------------------------------------------------------------
// Storage-backed validation tests
// -----------------------------------------------------------------------

#[test]
fn full_report_on_default_states() {
    let e = Env::default();
    let caller = Address::generate(&e);
    let id = e.register(InvariantHarness, ());
    e.as_contract(&id, || {
        set_vault_state(&e, VaultState::Uninitialized, caller.clone()).ok();
        set_staking_state(&e, StakingState::Uninitialized, caller.clone()).ok();
        set_reward_state(&e, RewardState::Idle, caller.clone()).ok();
        set_treasury_state(&e, TreasuryState::Normal, caller).ok();

        let report = generate_report(&e);
        assert_eq!(report.overall, ValidationStatus::Passed);
    });
}

#[test]
fn full_report_detects_inconsistencies() {
    let e = Env::default();
    let caller = Address::generate(&e);
    let id = e.register(InvariantHarness, ());
    e.as_contract(&id, || {
        set_vault_state(&e, VaultState::Terminated, caller.clone()).ok();
        set_staking_state(&e, StakingState::Active, caller.clone()).ok();
        set_reward_state(&e, RewardState::Accruing, caller.clone()).ok();
        set_treasury_state(&e, TreasuryState::Normal, caller).ok();

        let report = generate_report(&e);
        assert_eq!(report.overall, ValidationStatus::Failed);
        assert!(report.failed > 0);
    });
}

#[test]
fn accounting_invariant_pass() {
    let e = Env::default();
    e.ledger().set_timestamp(100);
    let asset = Address::generate(&e);
    let id = e.register(InvariantHarness, ());

    let caller = Address::generate(&e);
    e.as_contract(&id, || {
        set_vault_state(&e, VaultState::Active, caller).ok();
        record_operation(
            &e,
            AccountingEntry {
                category: AccountingCategory::Vault,
                operation: AccountingOperation::VaultDeposit,
                actor: None,
                asset: Some(asset),
                amount_in: 500,
                amount_out: 0,
                amount_processed: 500,
                resources: OperationResources::new(2, 3, 2, 1),
            },
        )
        .ok();

        let r = rule_accounting_consistency(&e);
        assert_eq!(r.status, ValidationStatus::Passed);
    });
}

#[test]
fn report_has_all_rule_names() {
    let e = Env::default();
    let caller = Address::generate(&e);
    let id = e.register(InvariantHarness, ());
    e.as_contract(&id, || {
        set_vault_state(&e, VaultState::Uninitialized, caller).ok();

        let report = generate_report(&e);
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
        for name in expected {
            let found = report
                .rules
                .iter()
                .any(|r| r.name == name);
            assert!(found, "rule {:?} missing from report", name);
        }
    });
}
