# MVP Demo Scenario Fixtures & Lifecycle Story

## Overview
The Axionvera Vault MVP Demo provides a scripted, end-to-end demonstration narrative illustrating the complete operational lifecycle of an Axionvera transparent vault.

This scenario guides developers, community reviewers, and SDK consumers through:
1. Vault setup and initialization by the community admin.
2. Contributor capital deposit and balance tracking.
3. Reward balance funding and claimable allocation.
4. Reward claiming and accounting reconciliation.
5. Principal capital withdrawal and final state verification.

All fixtures are contributor-safe, deterministic, and runnable in local mocked environments with **zero live secrets or testnet deployment requirements**.

## Participating Actors & Roles
- **Admin (`Alice`)**: `GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA` (Community Lead / Vault Deployer)
- **User 1 (`Bob`)**: `GBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB` (Contributor / Staker)
- **User 2 (`Charlie`)**: `GCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC` (Secondary Contributor)
- **Deposit Token**: `CDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD` (e.g. USDC / Community Pool Token)
- **Reward Token**: `CEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEEE` (e.g. VERA / Governance Token)

## Artifacts & Schema
- **Scenario Definition**: [`examples/mvp-demo-scenario/scenario.json`](../examples/mvp-demo-scenario/scenario.json)
- **State Transitions**: [`examples/mvp-demo-scenario/state-transitions.json`](../examples/mvp-demo-scenario/state-transitions.json)
- **Event Outputs**: [`examples/mvp-demo-scenario/event-outputs.json`](../examples/mvp-demo-scenario/event-outputs.json)
- **Schema**: [`schemas/mvp-demo-scenario.schema.json`](../schemas/mvp-demo-scenario.schema.json)
- **CLI Validator**: [`scripts/validate-mvp-demo-scenario.py`](../scripts/validate-mvp-demo-scenario.py)
- **Rust Integration Test**: [`contracts/vault-contract/tests/demo_scenario.rs`](../contracts/vault-contract/tests/demo_scenario.rs)

## Step-by-Step Scenario Walkthrough

### Step 1: Initialize Vault
- **Action**: `initialize(admin, deposit_token, reward_token)`
- **Caller**: Admin (`Alice`)
- **Pre-State**: `initialized: false`, `total_deposits: 0`
- **Post-State**: `initialized: true`, `admin: Alice`, `total_deposits: 0`, `reward_balance: 0`
- **Event Emitted**: `["vault", "init"]` with data `Alice`

### Step 2: User Deposit
- **Action**: `deposit(user_1, 1000)`
- **Caller**: User 1 (`Bob`)
- **Pre-State**: `user_1_balance: 0`, `total_deposits: 0`
- **Post-State**: `user_1_balance: 1000`, `total_deposits: 1000`
- **Event Emitted**: `["vault", "deposit"]` with data `(Bob, 1000)`

### Step 3: Reward Funding & Allocation
- **Action**: `set_reward_balance(500)` & `set_claimable_reward(user_1, 250)`
- **Caller**: Admin (`Alice`)
- **Pre-State**: `reward_balance: 0`, `user_1_claimable: 0`
- **Post-State**: `reward_balance: 500`, `user_1_claimable: 250`
- **Event Emitted**: None (internal accounting updates)

### Step 4: Claim Rewards
- **Action**: `claim_rewards(user_1)`
- **Caller**: User 1 (`Bob`)
- **Pre-State**: `user_1_claimable: 250`
- **Post-State**: `user_1_claimable: 0`, `claimed: 250`
- **Event Emitted**: `["vault", "claim"]` with data `(Bob, 250)`

### Step 5: Partial Principal Withdrawal
- **Action**: `withdraw(user_1, 400)`
- **Caller**: User 1 (`Bob`)
- **Pre-State**: `user_1_balance: 1000`, `total_deposits: 1000`
- **Post-State**: `user_1_balance: 600`, `total_deposits: 600`
- **Event Emitted**: `["vault", "withdraw"]` with data `(Bob, 400)`

## Validation & Verification

### 1. Validate Fixture Schema & Consistency
```bash
python3 scripts/validate-mvp-demo-scenario.py
```
Output:
```text
VALID: MVP demo scenario fixtures are consistent and valid
```

### 2. Run Python Regression Tests
```bash
python3 scripts/test-mvp-demo-scenario.py
```
Output:
```text
PASS MVP demo scenario fixtures validation
```

### 3. Run Rust Integration Test
```bash
cargo test --test demo_scenario
```
Output:
```text
running 2 tests
test test_mvp_demo_scenario_fixtures_consistency ... ok
test test_mvp_demo_scenario_execution_flow ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

## Mocked vs. Live Testnet Usage
- **Local / Mocked Testing**: The scenario runs in-memory using Soroban SDK test utilities with zero latency and simulated authorizations.
- **Maintainer Testnet Execution**: The exact sequence of 5 steps maps 1:1 to `soroban contract invoke` calls using real funded testnet accounts.
