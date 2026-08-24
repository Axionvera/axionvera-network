# Vault Contract Testnet Deployment Checklist

Use this checklist to prepare and validate the vault contract on Stellar
testnet. Run commands from the repository root unless a step says otherwise.

> **Testnet only:** Completing this checklist does not make the contract
> production-ready or production-audited. Stellar testnet data may be reset,
> and the current vault records accounting values without transferring tokens.
> The current `set_claimable_reward` and `set_reward_balance` methods also do
> not require authorization. Review these limitations before any testnet use.

## 1. Confirm the deployment inputs

- [ ] Install the Rust toolchain, the `wasm32-unknown-unknown` Rust target, and
      the Stellar CLI:

  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- [ ] Confirm the `testnet` network is available:

  ```bash
  stellar network ls
  ```

- [ ] Use the Stellar testnet network passphrase (`Test SDF Network ; September
      2015`) and a testnet RPC endpoint. The commands below always include
      `--network testnet` so a different default network is not used by
      accident.
- [ ] Select a testnet-funded Stellar CLI identity for deployment and
      initialization. In the examples below its local name is `deployer`.
- [ ] Record the public addresses that will be used for `admin`,
      `deposit_token`, and `reward_token`. They must be valid addresses on
      testnet. This contract does not deploy or configure token contracts.
- [ ] Confirm the admin can authorize the initialization transaction. The
      simplest test setup uses the `deployer` account as the admin.
- [ ] Never put a secret key or seed phrase in this repository, a deployment
      record, terminal history, or a pull request.

## 2. Run local quality checks

- [ ] Start from the commit that is intended for deployment and make sure any
      local changes are understood.
- [ ] Run every repository quality check and resolve all failures:

  ```bash
  cargo fmt --all -- --check
  cargo check --workspace --all-targets
  cargo test --workspace --all-targets
  cargo clippy --workspace --all-targets -- -D warnings
  ```

  See [CI Workflow and Local Checks](./ci-and-local-checks.md) for details and
  troubleshooting.

## 3. Build and identify the WASM artifact

- [ ] Build only the vault contract with the locked dependency versions:

  ```bash
  cargo build --locked --release --package axionvera-vault-contract --target wasm32-unknown-unknown --target-dir target
  ```

- [ ] Confirm the deployable artifact exists at:

  ```text
  target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm
  ```

- [ ] Record a SHA-256 checksum of that exact WASM file. Keep the checksum with
      the deployment record so the deployed build can be traced back to an
      artifact and source commit.

  On Linux or macOS:

  ```bash
  sha256sum target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm
  ```

  On PowerShell:

  ```powershell
  Get-FileHash target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm -Algorithm SHA256
  ```

## 4. Deploy to testnet

- [ ] Recheck that the source identity is funded on testnet and that every
      remote command below says `--network testnet`.
- [ ] Deploy the WASM. Replace `deployer` if the funded CLI identity has a
      different local name:

  ```bash
  stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm \
    --source deployer \
    --network testnet \
    --alias axionvera-vault-testnet
  ```

- [ ] Copy the returned contract ID, which starts with `C`, immediately. The
      alias is a local convenience and is not a substitute for recording the
      actual ID.
- [ ] Record the following public deployment metadata in the testnet
      environment configuration or the team deployment log:

  | Field | Value to record |
  | --- | --- |
  | Network | `testnet` |
  | Contract ID | Returned `C...` value |
  | Contract alias | `axionvera-vault-testnet` |
  | Source commit | Full Git commit SHA |
  | WASM checksum | SHA-256 from step 3 |
  | Deployer | Public `G...` account address only |
  | Deployment transaction | Transaction hash and timestamp |

- [ ] Keep testnet and future mainnet contract IDs in separate configuration
      values. Contract IDs are public and may be committed when the project's
      configuration policy allows it; signing secrets must never be committed.

## 5. Initialize the deployed contract

Deployment creates an uninitialized contract instance. Replace each placeholder
below with its testnet address. The account named by `--source` must satisfy the
admin authorization required by `initialize`.

- [ ] Initialize the contract exactly once:

  ```bash
  stellar contract invoke \
    --id <CONTRACT_ID> \
    --source deployer \
    --network testnet \
    -- initialize \
    --admin <ADMIN_ADDRESS> \
    --deposit_token <DEPOSIT_TOKEN_CONTRACT_ID> \
    --reward_token <REWARD_TOKEN_CONTRACT_ID>
  ```

- [ ] Record the initialization transaction hash and the three configured
      addresses. Do not retry initialization after it succeeds; a second call
      returns `AlreadyInitialized`.

## 6. Validate the deployment

Use `--send no` for read-only simulation. These checks should not write to the
ledger.

- [ ] Confirm initialization returns `true`:

  ```bash
  stellar contract invoke --id <CONTRACT_ID> --source deployer --network testnet --send no -- is_initialized
  ```

- [ ] Confirm the stored admin and token addresses match the deployment record:

  ```bash
  stellar contract invoke --id <CONTRACT_ID> --source deployer --network testnet --send no -- admin
  stellar contract invoke --id <CONTRACT_ID> --source deployer --network testnet --send no -- deposit_token
  stellar contract invoke --id <CONTRACT_ID> --source deployer --network testnet --send no -- reward_token
  ```

- [ ] Confirm a fresh deployment reports zero total deposits:

  ```bash
  stellar contract invoke --id <CONTRACT_ID> --source deployer --network testnet --send no -- total_deposits
  ```

- [ ] Confirm the initialization transaction succeeded and emitted the
      `(vault, init)` contract event using the team's testnet explorer or
      RPC tooling.
- [ ] Update the testnet consumer configuration with the recorded contract ID,
      then perform any SDK or integration smoke tests against **testnet only**.
- [ ] Save the validation results with the deployment record, including any
      failed checks. Do not describe this checklist, deployment, or validation
      as evidence of production readiness.
