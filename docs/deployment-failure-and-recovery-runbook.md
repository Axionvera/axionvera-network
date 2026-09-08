# Deployment Failure and Recovery Runbook

This runbook establishes standard operating procedures for diagnosing, containing, and recovering from testnet deployment failures in the Axionvera Network protocol and Soroban smart contracts.

---

## 1. Safety Principles and Role Separation

To protect protocol security, custody of cryptographic credentials, and network stability, deployment tasks are strictly separated:

| Responsibility | Contributor Scope (Safe / Non-Privileged) | Maintainer Scope (Privileged / Authorized) |
| :--- | :--- | :--- |
| **Code & Artifacts** | Local compilation, formatting, unit/integration tests, schema validation. | Final review, build verification, and git release tagging. |
| **Secrets & Keys** | **STRICTLY PROHIBITED:** Never handle, commit, or request private keys or seeds. | Secure key custody, local hardware/file signing, testnet identity management. |
| **Dry-Run & Mocks** | Run mock smoke tests, simulate scenarios, validate schemas. | Live transaction broadcast, contract deployment to public Testnet. |
| **Failure Recovery** | Reproduce bugs locally, fix contract logic, submit PRs. | Re-fund deployer accounts, execute redeployments, update canonical registries. |

> [!CAUTION]
> **CRITICAL SECRET HYGIENE RULE:**
> Under NO circumstance should any private key, seed phrase, secret key (`S...`), or live API token ever be committed to git, written to log files, pasted into pull requests, or shared in chat sessions. All maintainer operations must use secure local identity keystores (e.g. `stellar keys`) or ephemeral environment variables that are excluded by `.gitignore`.

---

## 2. Failure Scenarios and Step-by-Step Recovery

### Scenario A: Build and Compilation Failures

#### Symptoms
- `cargo build --target wasm32-unknown-unknown --release` fails with missing target, missing core library, or clippy compiler errors.
- Resulting WASM artifact exceeds Soroban contract size limits or fails bytecode verification.
- Mock Log: [`examples/deployment-failures/failed-build.log`](../examples/deployment-failures/failed-build.log)

#### Root Causes
1. The `wasm32-unknown-unknown` target is missing from the local Rust toolchain.
2. Outdated Rust toolchain or incompatible Soroban SDK dependency pin.
3. Unused imports, formatting violations, or warnings treated as errors under `-D warnings`.

#### Contributor Actions (Safe)
1. Verify the compilation target is installed:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
2. Run the repository's required local quality checks:
   ```bash
   cargo fmt --all -- --check
   cargo check --workspace --all-targets
   cargo test --workspace --all-targets
   cargo clippy --workspace --all-targets -- -D warnings
   ```
3. Run the automated build script:
   ```bash
   ./scripts/build-vault-wasm.sh
   ```

#### Maintainer Recovery Actions
- Confirm that the build produces a deterministic WASM binary under `target/wasm32-unknown-unknown/release/vault_contract.wasm`.
- Validate that the SHA-256 hash matches the committed build metadata (`examples/build-metadata.json`).

---

### Scenario B: Deployment and RPC Broadcast Failures

#### Symptoms
- `stellar contract deploy` times out after 30 seconds (HTTP 504 Gateway Timeout).
- RPC node returns rate limiting (HTTP 429 Too Many Requests) or "insufficient fee / account underfunded".
- Nonce / sequence number desynchronization during submission.
- Mock Log: [`examples/deployment-failures/failed-deploy-rpc-timeout.log`](../examples/deployment-failures/failed-deploy-rpc-timeout.log)

#### Root Causes
1. Public Stellar testnet RPC node (`https://soroban-testnet.stellar.org`) is undergoing heavy load or maintenance.
2. Deployer account balance is depleted (needs Friendbot funding).
3. Network congestion caused base fee spikes above transaction limit.
4. Periodic testnet ledger reset occurred, invalidating previous sequence numbers.

#### Contributor Actions (Safe)
1. Verify the network configuration schema:
   ```bash
   ./scripts/validate-testnet-config.sh
   ```
2. Check public network status at `https://dashboard.stellar.org`.
3. Do not attempt manual broadcasts or solicit live keys.

#### Maintainer Recovery Actions
1. **Check Deployer Balance:**
   ```bash
   stellar keys address deployer
   curl "https://horizon-testnet.stellar.org/accounts/$(stellar keys address deployer)"
   ```
2. **Re-fund via Friendbot (if balance is below 10 XLM):**
   ```bash
   curl "https://friendbot.stellar.org?addr=$(stellar keys address deployer)"
   ```
3. **Retry with Increased Gas / Fee Allowance:**
   ```bash
   stellar contract deploy \
     --wasm target/wasm32-unknown-unknown/release/vault_contract.wasm \
     --source deployer \
     --network testnet \
     --fee 100000
   ```
4. **Fallback RPC Endpoint:**
   If the default RPC is unresponsive, configure an alternate reliable Soroban RPC provider (e.g. Infura, Blockdaemon, or a local standalone container).

---

### Scenario C: Initialization Failures and Auth Rejections

#### Symptoms
- Contract invocation of `initialize` returns HostError `#1` (`already_initialized`).
- Invocation fails with missing signature or unauthorized caller.
- Token contract addresses for `deposit_token` or `reward_token` fail validation.
- Mock Log: [`examples/deployment-failures/failed-initialization-auth.log`](../examples/deployment-failures/failed-initialization-auth.log)

#### Root Causes
1. **Double Initialization:** `initialize` was already invoked on this contract instance; Soroban contract state is immutable once initialized.
2. **Auth Missing:** The specified `admin` address did not co-sign the invocation.
3. **Invalid Token Address:** Mismatched address format or non-existent token contract on testnet.

#### Contributor Actions (Safe)
1. Validate initialization input schemas and test vectors:
   ```bash
   python3 scripts/validate-vault-initialization-input.py
   python3 scripts/test-vault-initialization-input.py
   ```
2. Ensure initialization payloads match `schemas/vault-initialization-input.schema.json`.

#### Maintainer Recovery Actions
1. **If Contract is Already Initialized with Wrong Parameters:**
   - Soroban does not support re-initializing an existing contract instance.
   - Deploy a **fresh contract instance** using `stellar contract deploy`.
   - Update `contract-id-registry.json` and `sdk-handoff.json` with the new Contract ID.
2. **If Invocation Failed Due to Signature:**
   - Ensure the invocation provides the `--source` that corresponds to the `admin` key, or provide a multisig transaction envelope for co-signing.

---

### Scenario D: Wrong Contract ID and Environment Mismatch

#### Symptoms
- Automated verification script reports `[FAIL] Contract ID not found on Testnet ledger`.
- Client or SDK requests result in 404 or missing ledger entry.
- Standalone / Localnet contract ID was accidentally copied into testnet configurations.
- Mock Log: [`examples/deployment-failures/wrong-contract-id.log`](../examples/deployment-failures/wrong-contract-id.log)

#### Root Causes
1. Stale contract ID from a previous testnet epoch before a testnet reset.
2. Cross-environment pollution (e.g. copying a local container contract ID into `sdk-handoff.json`).
3. Out-of-sync `contract-id-registry.json`.

#### Contributor Actions (Safe)
1. Run local validation tooling to check registry consistency:
   ```bash
   python3 scripts/validate-contract-id-registry.py
   python3 scripts/test-contract-id-registry.py
   ```

#### Maintainer Recovery Actions
1. Validate live on-chain contract existence using the deployment verification script:
   ```bash
   python3 scripts/verify-vault-deployment.py --contract-id <CONTRACT_ID> --network testnet
   ```
2. If the contract was lost in a testnet reset:
   a. Re-deploy the contract WASM.
   b. Initialize the contract with verified testnet token addresses.
   c. Update `examples/contract-id-registry.json` and `examples/sdk-handoff.json`.
   d. Run `./scripts/verify-vault-deployment.sh` to confirm ledger sync.

---

### Scenario E: Post-Deployment Smoke Test Failures

#### Symptoms
- Smoke test assertion fails (e.g. deposit recorded does not match event payload, or withdrawal calculation diverges).
- Event emission topics missing or corrupted.
- Mock Log: [`examples/deployment-failures/failed-smoke-test.log`](../examples/deployment-failures/failed-smoke-test.log)

#### Root Causes
1. Contract logic divergence between local Soroban SDK mock environment and real testnet protocol version.
2. Incompatible event schema version between contract emission and client decoder.
3. Rounding error or fee deduction in testnet token contract.

#### Contributor Actions (Safe)
1. Run the local mocked smoke test suite to isolate logic errors:
   ```bash
   python3 scripts/run-mocked-smoke-test.py --input examples/mock-post-deployment-smoke-input.json
   python3 scripts/test-mocked-smoke-test.py
   ```
2. Verify contract tests in Rust:
   ```bash
   cargo test -p vault-contract --test smoke_mocked
   ```
3. If a bug is detected, implement a fix, add regression test, and submit a PR.

#### Maintainer Recovery Actions
1. If the live contract deployed on testnet contains a defect:
   - Mark the deployed contract ID as `deprecated` or `failed_smoke_test` in `contract-id-registry.json`.
   - Notify consumers and SDK maintainers to pause integration against that contract ID.
   - Await contributor bugfix PR, review and merge, then execute a clean re-deployment.

---

## 3. Incident Decision Tree and Triage Matrix

```text
Deployment Failure Occurred
  │
  ├─► Compilation / Build Error?
  │     └─► Verify toolchain -> rustup target add wasm32-unknown-unknown -> cargo clippy -> re-build
  │
  ├─► RPC Timeout / Broadcast Error?
  │     └─► Check https://dashboard.stellar.org -> Verify Friendbot balance -> Increase --fee -> Retry
  │
  ├─► Initialization Rejected?
  │     ├─► Already Initialized -> Deploy fresh contract instance -> Re-run initialize
  │     └─► Auth Failed -> Verify admin public key -> Re-sign with admin identity
  │
  ├─► Wrong Contract ID / Ledger 404?
  │     └─► Run verify-vault-deployment.py -> Confirm testnet vs standalone -> Update registry
  │
  └─► Smoke Test Failed?
        └─► Run run-mocked-smoke-test.py -> Check Rust smoke_mocked test -> Patch logic -> Redeploy
```

---

## 4. Post-Recovery Verification Checklist

Once the recovery actions are completed, run the full release readiness suite to ensure the repository remains in a clean, verifiable state:

- [ ] Run release readiness checklist:
  ```bash
  ./scripts/release-readiness-check.sh --full
  ```
- [ ] Run contract ID registry validation:
  ```bash
  python3 scripts/validate-contract-id-registry.py
  ```
- [ ] Run SDK handoff validation:
  ```bash
  python3 scripts/validate-sdk-handoff.py
  ```
- [ ] Confirm zero committed secrets:
  ```bash
  git diff HEAD~1 | grep -iE 'secret|mnemonic|private_key|seed' || true
  ```
