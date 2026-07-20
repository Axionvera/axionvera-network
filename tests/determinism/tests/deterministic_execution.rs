use axionvera_testing::{
    DeterministicCriteria, ExecutionEntry, Failure, FailureKind, VerificationCriteria,
    VerificationReport, verify_deterministic,
};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events, Ledger, LedgerInfo},
    Address, Env, Symbol,
};

// -----------------------------------------------------------------------
// Test helpers
// -----------------------------------------------------------------------

/// A minimal contract for testing deterministic execution.
#[soroban_sdk::contract]
pub struct DeterminismHarness;

#[soroban_sdk::contractimpl]
impl DeterminismHarness {
    pub fn noop() {}

    pub fn add(e: Env, a: i32, b: i32) -> i32 {
        a + b
    }

    pub fn store_and_retrieve(e: Env, key: Symbol, value: u32) -> u32 {
        e.storage().instance().set(&key, &value);
        value
    }

    pub fn read_stored(e: Env, key: Symbol) -> u32 {
        e.storage().instance().get(&key).unwrap_or(0)
    }

    pub fn emit_event(e: Env, topic: Symbol) {
        e.events().publish((symbol_short!("test"), topic), ());
    }

    pub fn transfer_events(e: Env, count: u32, topic: Symbol) {
        for _ in 0..count {
            e.events().publish((symbol_short!("test"), topic.clone()), ());
        }
    }
}

fn harness_env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set(LedgerInfo {
        timestamp: 1000,
        protocol_version: 22,
        sequence_number: 1,
        network_id: [0; 32],
        base_reserve: 10,
        min_persistent_entry_ttl: 518400,
        min_temp_entry_ttl: 518400,
        max_entry_ttl: 6312000,
    });
    e.register(DeterminismHarness, ());
    e
}

fn harness_client(e: &Env) -> DeterminismHarnessClient {
    DeterminismHarnessClient::new(e)
}

/// Compare the events emitted by two separate environments.
fn compare_executions(e1: &Env, e2: &Env) -> core::cmp::Ordering {
    use soroban_sdk::xdr::ToXdr;
    let events1 = e1.events().all();
    let events2 = e2.events().all();

    if events1.len() != events2.len() {
        return core::cmp::Ordering::Greater;
    }

    for i in 0..events1.len() {
        let ev1 = events1.get(i).unwrap();
        let ev2 = events2.get(i).unwrap();
        if ev1.0 != ev2.0 || ev1.1 != ev2.1 || ev1.2 != ev2.2 {
            return core::cmp::Ordering::Greater;
        }

        let data1 = ev1.2.to_xdr(e1);
        let data2 = ev2.2.to_xdr(e2);
        if data1 != data2 {
            return core::cmp::Ordering::Greater;
        }
    }

    core::cmp::Ordering::Equal
}

/// Run a closure N times and verify that event count is consistent.
fn verify_event_count_determinism<F, R>(
    label: &'static str,
    env_factory: fn() -> Env,
    operation: F,
    runs: u32,
) -> VerificationReport
where
    F: Fn(&Env) -> R,
{
    let mut report = VerificationReport::new(label, runs);
    let mut event_counts: Vec<u32> = Vec::new();

    for i in 0..runs {
        let env = env_factory();
        let _ = operation(&env);
        let count = env.events().all().len();

        if i > 0 {
            let prev = event_counts[(i - 1) as usize];
            if count != prev {
                report.add_failure(Failure {
                    run: i + 1,
                    kind: FailureKind::EventCountMismatch,
                    detail: "Event count differs from previous run",
                });
            }
        }

        event_counts.push(count);
        report.add_entry(ExecutionEntry {
            run: i + 1,
            result_xdr: Vec::new(),
            event_count: count,
            storage_keys: Vec::new(),
        });
    }

    if report.failures.is_empty() {
        report.passed = true;
    }

    report
}

// -----------------------------------------------------------------------
// Deterministic execution tests
// -----------------------------------------------------------------------

/// Verify that a simple pure function is deterministic across 10 runs.
#[test]
fn test_pure_function_determinism() {
    let runs = 10u32;
    let report = verify_deterministic("pure_add", harness_env, |e| {
        let client = harness_client(e);
        client.add(&3, &7)
    }, runs);

    assert!(
        report.is_passed(),
        "Pure function should be deterministic: {}",
        report.format()
    );
    assert_eq!(report.failure_count(), 0);
}

/// Verify that a store-and-retrieve operation is deterministic.
#[test]
fn test_storage_determinism() {
    let runs = 5u32;
    let report = verify_deterministic("storage_store", harness_env, |e| {
        let client = harness_client(e);
        client.store_and_retrieve(&Symbol::new(e, "test_key"), &42)
    }, runs);

    assert!(
        report.is_passed(),
        "Storage operation should be deterministic: {}",
        report.format()
    );
}

/// Verify event count consistency across multiple runs.
#[test]
fn test_event_count_determinism() {
    let runs = 5u32;
    let report = verify_event_count_determinism("event_count", harness_env, |e| {
        let client = harness_client(e);
        client.transfer_events(&3, &symbol_short!("deposit"))
    }, runs);

    assert!(
        report.is_passed(),
        "Event count should be deterministic: {}",
        report.format()
    );
}

/// Verify that identical environments produce identical event sequences.
#[test]
fn test_event_content_determinism() {
    let e1 = harness_env();
    let e2 = harness_env();

    let c1 = harness_client(&e1);
    let c2 = harness_client(&e2);

    c1.transfer_events(&2, &symbol_short!("test_topic"));
    c2.transfer_events(&2, &symbol_short!("test_topic"));

    let cmp = compare_executions(&e1, &e2);
    assert_eq!(cmp, core::cmp::Ordering::Equal);
}

/// Verify that reading initial state is deterministic.
#[test]
fn test_initial_state_determinism() {
    let runs = 5u32;
    let report = verify_deterministic("initial_state", harness_env, |e| {
        let client = harness_client(e);
        client.read_stored(&Symbol::new(e, "nonexistent"))
    }, runs);

    assert!(
        report.is_passed(),
        "Initial state should be deterministic: {}",
        report.format()
    );
}

/// Verify that noop (empty function) is deterministic.
#[test]
fn test_noop_determinism() {
    let runs = 5u32;
    let report = verify_deterministic("noop", harness_env, |e| {
        let client = harness_client(e);
        client.noop()
    }, runs);

    assert!(
        report.is_passed(),
        "Noop should be deterministic: {}",
        report.format()
    );
}

// -----------------------------------------------------------------------
// State transition determinism tests
// -----------------------------------------------------------------------

/// Verify that vault state transitions are deterministic.
#[test]
fn test_state_machine_transition_determinism() {
    use axionvera_storage::{set_vault_state, get_vault_state};
    use axionvera_state::VaultState;

    let runs = 5u32;

    let report = verify_deterministic("vault_state_transition", || {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().set(LedgerInfo {
            timestamp: 1000,
            protocol_version: 22,
            sequence_number: 1,
            network_id: [0; 32],
            base_reserve: 10,
            min_persistent_entry_ttl: 518400,
            min_temp_entry_ttl: 518400,
            max_entry_ttl: 6312000,
        });
        e
    }, |e| {
        let admin = Address::generate(e);
        let _ = set_vault_state(e, VaultState::Active, admin.clone());
        let _ = set_vault_state(e, VaultState::Paused, admin);
        get_vault_state(e) as u32
    }, runs);

    assert!(
        report.is_passed(),
        "State transitions should be deterministic: {}",
        report.format()
    );
}

/// Verify that reward state transitions are deterministic.
#[test]
fn test_reward_state_determinism() {
    use axionvera_storage::set_reward_state;
    use axionvera_state::RewardState;

    let runs = 5u32;

    let report = verify_deterministic("reward_state_cycle", || {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().set(LedgerInfo {
            timestamp: 1000,
            protocol_version: 22,
            sequence_number: 1,
            network_id: [0; 32],
            base_reserve: 10,
            min_persistent_entry_ttl: 518400,
            min_temp_entry_ttl: 518400,
            max_entry_ttl: 6312000,
        });
        e
    }, |e| {
        let admin = Address::generate(e);
        let _ = set_reward_state(e, RewardState::Accruing, admin.clone());
        let _ = set_reward_state(e, RewardState::ReadyForDistribution, admin.clone());
        let _ = set_reward_state(e, RewardState::Distributing, admin);
        true
    }, runs);

    assert!(
        report.is_passed(),
        "Reward state cycle should be deterministic: {}",
        report.format()
    );
}

// -----------------------------------------------------------------------
// Edge case tests
// -----------------------------------------------------------------------

/// Verify determinism with zero values.
#[test]
fn test_zero_value_determinism() {
    let runs = 5u32;
    let report = verify_deterministic("zero_values", harness_env, |e| {
        let client = harness_client(e);
        client.add(&0, &0)
    }, runs);

    assert!(
        report.is_passed(),
        "Zero values should be deterministic: {}",
        report.format()
    );
}

/// Verify determinism with max i32 values.
#[test]
fn test_max_value_determinism() {
    let runs = 5u32;
    let report = verify_deterministic("max_values", harness_env, |e| {
        let client = harness_client(e);
        client.add(&i32::MAX, &0)
    }, runs);

    assert!(
        report.is_passed(),
        "Max values should be deterministic: {}",
        report.format()
    );
}

/// Verify determinism with negative values.
#[test]
fn test_negative_value_determinism() {
    let runs = 5u32;
    let report = verify_deterministic("negative_values", harness_env, |e| {
        let client = harness_client(e);
        client.add(&(-100), &50)
    }, runs);

    assert!(
        report.is_passed(),
        "Negative values should be deterministic: {}",
        report.format()
    );
}

// -----------------------------------------------------------------------
// Verification report tests
// -----------------------------------------------------------------------

/// Verify that the VerificationReport struct formats correctly.
#[test]
fn test_report_formatting() {
    let mut report = VerificationReport::new("test_report", 3);
    report.add_entry(ExecutionEntry {
        run: 1,
        result_xdr: Vec::new(),
        event_count: 0,
        storage_keys: Vec::new(),
    });
    report.add_failure(Failure {
        run: 1,
        kind: FailureKind::ResultMismatch,
        detail: "Mismatch detected",
    });

    let formatted = report.format();
    assert!(!formatted.is_empty());
    assert!(!report.is_passed());
    assert_eq!(report.failure_count(), 1);
}

/// Verify that a passing report is correctly identified.
#[test]
fn test_passing_report() {
    let mut report = VerificationReport::new("passing_test", 2);
    report.add_entry(ExecutionEntry {
        run: 1,
        result_xdr: Vec::new(),
        event_count: 0,
        storage_keys: Vec::new(),
    });
    report.add_entry(ExecutionEntry {
        run: 2,
        result_xdr: Vec::new(),
        event_count: 0,
        storage_keys: Vec::new(),
    });
    report.passed = true;

    assert!(report.is_passed());
    assert_eq!(report.failure_count(), 0);
}

// -----------------------------------------------------------------------
// Verification criteria tests
// -----------------------------------------------------------------------

/// Verify that the VerificationCriteria correctly manages criteria.
#[test]
fn test_verification_criteria() {
    let criteria = VerificationCriteria::all();
    assert_eq!(criteria.len(), 7);

    let c = criteria.get(0).unwrap();
    assert_eq!(*c, DeterministicCriteria::OutputConsistency);

    let desc = c.description();
    assert!(!desc.is_empty());
}

/// Verify individual criteria descriptions.
#[test]
fn test_criteria_descriptions() {
    assert!(
        DeterministicCriteria::OutputConsistency
            .description()
            .contains("Output Consistency")
    );
    assert!(
        DeterministicCriteria::EventCountConsistency
            .description()
            .contains("Event Count")
    );
    assert!(
        DeterministicCriteria::NoNonDeterministicSources
            .description()
            .contains("Non-Deterministic")
    );
}

// -----------------------------------------------------------------------
// Methodology documentation tests
// -----------------------------------------------------------------------

/// Verify the testing methodology using deterministic properties.
#[test]
fn test_methodology_documentation() {
    let e = harness_env();
    let client = harness_client(&e);
    let result_a = client.read_stored(&Symbol::new(&e, "test"));

    let e2 = harness_env();
    let client2 = harness_client(&e2);
    let result_b = client2.read_stored(&Symbol::new(&e2, "test"));

    assert_eq!(result_a, result_b);
}

// -----------------------------------------------------------------------
// Edge case: empty key determinism
// -----------------------------------------------------------------------

/// Verify determinism with empty Symbol keys.
#[test]
fn test_empty_key_determinism() {
    let e1 = harness_env();
    let e2 = harness_env();

    let c1 = harness_client(&e1);
    let c2 = harness_client(&e2);

    let empty_key = Symbol::new(&e1, "");
    c1.store_and_retrieve(&empty_key, &99);
    c2.store_and_retrieve(&empty_key, &99);

    let cmp = compare_executions(&e1, &e2);
    assert_eq!(cmp, core::cmp::Ordering::Equal);
}

/// Verify determinism with multiple keys.
#[test]
fn test_multiple_key_determinism() {
    let e1 = harness_env();
    let e2 = harness_env();

    let c1 = harness_client(&e1);
    let c2 = harness_client(&e2);

    let keys = ["a", "b", "c", "d", "e"];
    for k in &keys {
        let sym = Symbol::new(&e1, k);
        c1.store_and_retrieve(&sym, &1);
        c2.store_and_retrieve(&sym, &1);
    }

    let cmp = compare_executions(&e1, &e2);
    assert_eq!(cmp, core::cmp::Ordering::Equal);
}

// -----------------------------------------------------------------------
// No random or time-dependent operations
// -----------------------------------------------------------------------

/// Verify that operations using ledger timestamp are deterministic.
#[test]
fn test_ledger_time_is_deterministic() {
    let runs = 5u32;
    let report = verify_deterministic("ledger_timestamp", || {
        let e = Env::default();
        e.mock_all_auths();
        e.ledger().set(LedgerInfo {
            timestamp: 5000,
            protocol_version: 22,
            sequence_number: 1,
            network_id: [0; 32],
            base_reserve: 10,
            min_persistent_entry_ttl: 518400,
            min_temp_entry_ttl: 518400,
            max_entry_ttl: 6312000,
        });
        e
    }, |e| e.ledger().timestamp(), runs);

    assert!(
        report.is_passed(),
        "Ledger timestamp should be deterministic: {}",
        report.format()
    );
}
