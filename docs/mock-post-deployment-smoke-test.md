# Mocked Post-Deployment Smoke Test Flow

## Overview
Before maintainers execute live testnet deployments or end-to-end user smoke testing, contributors and CI pipelines need a safe, deterministic, and mocked smoke test flow.

This workflow verifies:
1. Deployed contract ID resolution from multi-environment registries.
2. Initialization parameter loading and schema compliance.
3. Pre-operation contract state verification.
4. End-to-end vault lifecycle accounting (initialization, deposit, reward funding, claim, and withdrawal).
5. Emitted event audit trails matching documented event topics and payloads.

All checks are conducted **without live RPC calls, network connections, or private keys**.

## Artifacts & Components
- **Schema**: [`schemas/mock-post-deployment-smoke.schema.json`](../schemas/mock-post-deployment-smoke.schema.json)
- **Input Example**: [`examples/mock-post-deployment-smoke-input.json`](../examples/mock-post-deployment-smoke-input.json)
- **Output Report Example**: [`examples/mock-post-deployment-smoke-output.json`](../examples/mock-post-deployment-smoke-output.json)
- **CLI Runner**: [`scripts/run-mocked-smoke-test.py`](../scripts/run-mocked-smoke-test.py)
- **Shell Runner**: [`scripts/run-mocked-smoke-test.sh`](../scripts/run-mocked-smoke-test.sh)
- **Python Regression Tests**: [`scripts/test-mocked-smoke-test.py`](../scripts/test-mocked-smoke-test.py)
- **Rust Integration Test**: [`contracts/vault-contract/tests/smoke_mocked.rs`](../contracts/vault-contract/tests/smoke_mocked.rs)

## Lifecycle Stages

| Stage | Name | Checks Performed |
|---|---|---|
| 1 | `contract_id_loading` | Resolves contract ID from registry or fallback, validates Stellar `C...` format |
| 2 | `initialization_input_loading` | Loads admin public key (`G...`), deposit token, and reward token |
| 3 | `read_verification` | Verifies `is_initialized == true`, `total_deposits == 0`, `reward_balance == 0` |
| 4 | `deposit_accounting_simulation` | Simulates user deposit, verifies updated user balance and total deposits |
| 5 | `reward_and_claim_simulation` | Simulates reward balance update, pending reward calculation, claim, and withdrawal |

## Running the Mocked Smoke Flow

### 1. Execute CLI Runner
```bash
python3 scripts/run-mocked-smoke-test.py
```

To emit structured JSON matching [`schemas/mock-post-deployment-smoke.schema.json`](../schemas/mock-post-deployment-smoke.schema.json):
```bash
python3 scripts/run-mocked-smoke-test.py --json --output /tmp/smoke-report.json
```

### 2. Execute Shell Template
```bash
./scripts/run-mocked-smoke-test.sh
```

### 3. Run Python Regression Tests
```bash
python3 scripts/test-mocked-smoke-test.py
```
Output:
```text
PASS mocked post-deployment smoke test flow validation
```

### 4. Run Rust Integration Test
```bash
cargo test --test smoke_mocked
```
Output:
```text
running 2 tests
test test_fixture_loading_and_contract_id_resolution ... ok
test test_mocked_post_deployment_smoke_flow_lifecycle ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

## Security & Maintenance Boundary
- **Zero Secrets**: No seed phrases, private keys, or credentials are required or accepted.
- **Zero Live Network Calls**: All testing occurs within local unit/mock environments or through deterministic schema validators.
- **Maintainer Handoff**: Once a maintainer deploys the contract to Stellar Testnet, the identical sequence of read and state calls can be directed to the live contract instance via the Soroban CLI.
