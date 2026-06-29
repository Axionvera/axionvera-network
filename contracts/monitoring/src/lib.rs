#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env};

// Define health status indicators
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HealthStatus {
    Healthy = 0,
    Degraded = 1,
    Critical = 2,
}

// Define the comprehensive status structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtocolHealthReport {
    pub status: HealthStatus,
    pub timestamp: u64,
    pub ledger_sequence: u32,
    pub checks_passed: u32,
    pub checks_failed: u32,
}

#[contracttype]
pub enum StorageKey {
    MinTreasuryBaseline,
    ProtocolAdmin,
}

#[contract]
pub struct ProtocolMonitor;

#[contractimpl]
impl ProtocolMonitor {
    /// Initialize the monitoring module configurations
    pub fn initialize(e: Env, admin: Address, min_treasury: i128) {
        if e.storage().instance().has(&StorageKey::ProtocolAdmin) {
            panic!("Already initialized");
        }
        e.storage().instance().set(&StorageKey::ProtocolAdmin, &admin);
        e.storage().instance().set(&StorageKey::MinTreasuryBaseline, &min_treasury);
    }

    /// Evaluates the complete status of the protocol by assessing key metrics
    pub fn evaluate_protocol_health(e: Env, current_treasury_balance: i128, core_operational: bool) -> ProtocolHealthReport {
        let min_treasury: i128 = e
            .storage()
            .instance()
            .get(&StorageKey::MinTreasuryBaseline)
            .unwrap_or(0);

        let mut passed = 0;
        let mut failed = 0;

        // Check 1: Contract Availability / Operational state
        if core_operational {
            passed += 1;
        } else {
            failed += 1;
            // Emit explicit event for core failure
            e.events().publish(
                (symbol_short!("fail"), symbol_short!("core")),
                symbol_short!("inactive")
            );
        }

        // Check 2: Treasury Balance consistency check
        if current_treasury_balance >= min_treasury {
            passed += 1;
        } else {
            failed += 1;
            // Emit explicit event for treasury deficit
            e.events().publish(
                (symbol_short!("fail"), symbol_short!("treasury")),
                current_treasury_balance
            );
        }

        // Determine final health calculation status
        let final_status = if failed == 0 {
            HealthStatus::Healthy
        } else if failed == 1 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Critical
        };

        let report = ProtocolHealthReport {
            status: final_status.clone(),
            timestamp: e.ledger().timestamp(),
            ledger_sequence: e.ledger().sequence(),
            checks_passed: passed,
            checks_failed: failed,
        };

        // Emit consistent health update tracking event
        e.events().publish(
            (symbol_short!("monitor"), symbol_short!("report")),
            report.clone()
        );

        // Extend instance TTL safety window
        e.storage().instance().extend_ttl(518400, 518400);

        report
    }

    /// Update the treasury limit threshold configuration
    pub fn set_treasury_baseline(e: Env, admin: Address, new_baseline: i128) {
        let current_admin: Address = e
            .storage()
            .instance()
            .get(&StorageKey::ProtocolAdmin)
            .unwrap();
        admin.require_auth();
        if admin != current_admin {
            panic!("Unauthorized admin change");
        }
        e.storage().instance().set(&StorageKey::MinTreasuryBaseline, &new_baseline);
    }
}
mod test;
