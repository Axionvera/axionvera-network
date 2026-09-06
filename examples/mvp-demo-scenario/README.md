# MVP Demo Scenario Fixtures

This directory contains deterministic, mocked scenario fixtures for the Axionvera Vault MVP Demo.

## Files
- `scenario.json`: The master scripted demo sequence covering initialization, deposit, reward funding, claim, and withdrawal.
- `state-transitions.json`: Pre- and post-state transition vectors for each step.
- `event-outputs.json`: Contract event topics and payloads emitted per step.

## Usage
Validated by:
- Rust integration tests in `contracts/vault-contract/tests/demo_scenario.rs`
- CLI validator in `scripts/validate-mvp-demo-scenario.py`
