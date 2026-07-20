pub mod report;
pub mod criteria;

pub use report::*;
pub use criteria::*;

/// Run a closure `N` times on freshly created environments and verify
/// every execution produces an identical result.
///
/// `setup` must return a fresh `Env` for each run.
/// Returns a `VerificationReport` summarising all runs.
pub fn verify_deterministic<F, R>(
    label: &'static str,
    setup: fn() -> soroban_sdk::Env,
    operation: F,
    runs: u32,
) -> VerificationReport
where
    F: Fn(&soroban_sdk::Env) -> R,
    R: PartialEq + Clone + core::fmt::Debug,
{
    let mut report = VerificationReport::new(label, runs);
    let mut results: Vec<R> = Vec::new();
    let mut event_counts: Vec<u32> = Vec::new();

    for i in 0..runs {
        let env = setup();
        let result = operation(&env);

        let entry = ExecutionEntry {
            run: i + 1,
            result_xdr: Vec::new(),
            event_count: 0,
            storage_keys: Vec::new(),
        };

        if i > 0 {
            let prev = &results[(i - 1) as usize];
            if result != *prev {
                report.add_failure(Failure {
                    run: i + 1,
                    kind: FailureKind::ResultMismatch,
                    detail: "Execution result differs from previous run",
                });
            }
        }

        results.push(result);
        event_counts.push(0);
        report.add_entry(entry);
    }

    if report.failures.is_empty() {
        report.passed = true;
    }

    report
}


