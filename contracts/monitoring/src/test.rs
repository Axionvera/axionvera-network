#![cfg(test)]
use super::{HealthStatus, ProtocolMonitor, ProtocolMonitorClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

#[test]
fn test_healthy_status() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register(ProtocolMonitor, ());
    let client = ProtocolMonitorClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    let min_treasury = 10_000_i128;

    client.initialize(&admin, &min_treasury);

    // Both checks pass (Core operational, treasury balance >= baseline)
    let report = client.evaluate_protocol_health(&15_000_i128, &true);

    assert_eq!(report.status, HealthStatus::Healthy);
    assert_eq!(report.checks_passed, 2);
    assert_eq!(report.checks_failed, 0);
}

#[test]
fn test_degraded_status_on_treasury_deficit() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register(ProtocolMonitor, ());
    let client = ProtocolMonitorClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin, &10_000_i128);

    // One check fails (Treasury drops below baseline)
    let report = client.evaluate_protocol_health(&5_000_i128, &true);

    assert_eq!(report.status, HealthStatus::Degraded);
    assert_eq!(report.checks_passed, 1);
    assert_eq!(report.checks_failed, 1);
}

#[test]
fn test_critical_status_on_total_failure() {
    let e = Env::default();
    e.mock_all_auths();

    let contract_id = e.register(ProtocolMonitor, ());
    let client = ProtocolMonitorClient::new(&e, &contract_id);

    let admin = Address::generate(&e);
    client.initialize(&admin, &10_000_i128);

    // Both checks fail (Core offline AND treasury deficit)
    let report = client.evaluate_protocol_health(&2_000_i128, &false);

    assert_eq!(report.status, HealthStatus::Critical);
    assert_eq!(report.checks_passed, 0);
    assert_eq!(report.checks_failed, 2);
}
