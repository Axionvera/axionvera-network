/// Represents the result of a single execution run.
#[derive(Clone, Debug)]
pub struct ExecutionEntry {
    pub run: u32,
    pub result_xdr: Vec<u8>,
    pub event_count: u32,
    pub storage_keys: Vec<String>,
}

/// The kind of deterministic failure detected.
#[derive(Clone, Debug, PartialEq)]
pub enum FailureKind {
    ResultMismatch,
    EventCountMismatch,
    EventContentMismatch,
    StorageMismatch,
    UnexpectedPanic,
}

/// A single verification failure.
#[derive(Clone, Debug)]
pub struct Failure {
    pub run: u32,
    pub kind: FailureKind,
    pub detail: &'static str,
}

impl Failure {
    pub fn description(&self) -> String {
        format!("Run {}: {}", self.run, self.detail)
    }
}

/// Aggregated report for a deterministic verification session.
#[derive(Clone, Debug)]
pub struct VerificationReport {
    pub label: &'static str,
    pub runs: u32,
    pub passed: bool,
    pub entries: Vec<ExecutionEntry>,
    pub failures: Vec<Failure>,
}

impl VerificationReport {
    pub fn new(label: &'static str, runs: u32) -> Self {
        Self {
            label,
            runs,
            passed: false,
            entries: Vec::new(),
            failures: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: ExecutionEntry) {
        self.entries.push(entry);
    }

    pub fn add_failure(&mut self, failure: Failure) {
        self.failures.push(failure);
    }

    /// Format the report as a multi-line string for display.
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Deterministic Verification Report ===\n");
        output.push_str(&format!("Label: {}\n", self.label));
        output.push_str(&format!("Runs: {}\n", self.runs));
        output.push_str("Status: ");
        if self.passed {
            output.push_str("PASSED");
        } else {
            output.push_str("FAILED");
        }
        output.push('\n');

        if !self.failures.is_empty() {
            output.push_str("\nFailures:\n");
            for f in &self.failures {
                output.push_str(&format!("  - {}\n", f.description()));
            }
        }

        output.push_str("\nExecution Summary:\n");
        for entry in &self.entries {
            output.push_str(&format!(
                "  Run {}: events={}\n",
                entry.run, entry.event_count
            ));
        }

        output
    }

    /// Returns true if no failures were detected.
    pub fn is_passed(&self) -> bool {
        self.passed
    }

    /// Number of failures detected.
    pub fn failure_count(&self) -> u32 {
        self.failures.len() as u32
    }
}
