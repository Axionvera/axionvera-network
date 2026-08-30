# Maintainer Handoff Guide: Testnet Deployment

This guide establishes a secure, contributor-safe process for preparing and executing testnet deployments for the Axionvera Network protocol and Soroban smart contracts.

---

## 1. Security & Responsibility Boundary

To protect network security, maintain private key custody, and ensure reproducible deployments, responsibilities are strictly separated between contributors and maintainers:

| Responsibility Area | Contributor Scope (Safe) | Maintainer Scope (Privileged) |
| :--- | :--- | :--- |
| **Code & Contracts** | Build contracts, write unit/integration tests, optimize WASM. | Review and approve code, perform final audit verification. |
| **Configuration** | Provide mock/placeholder `.env.example` templates and schemas. | Configure live testnet RPC endpoints and funded deployment keys. |
| **Secrets & Keys** | **Never touch or commit private keys, mnemonics, or live secrets.** | Manage deployment keypairs and network accounts securely. |
| **Deployment** | Test against local/mocked environments (`cargo test`). | Execute actual contract deployment to public Testnet. |
| **Artifacts** | Define JSON schemas and validate mock deployment outputs. | Sign and commit canonical deployment artifact JSON files. |

---

## 2. Contributor Preparation Workflow (Safe)

Contributors must verify that all artifacts, schemas, and contract checks pass locally before handing off the release:

1. **Local Build & Quality Checks:**
   Verify that the workspace compiles cleanly without warnings:
   ```bash
   cargo fmt --check
   cargo check
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   ```

2. **Release Readiness Checklist:**
   Before handing off, run the automated checklist to confirm all required
   docs, schemas, examples, scripts, and local quality checks are present:
   ```bash
   ./scripts/release-readiness-check.sh
   ```
   Use `--full` to include `cargo test` and `cargo clippy`.

---

## 3. SDK Handoff Artifact Package

Once the maintainer deploys the network contracts and initializes on-chain state, the SDK repository (`axionvera-sdk`) requires contract addresses, network configurations, interface versions, and event schema references to operate.

### Artifact Structure

The SDK handoff artifact package shape is governed by `schemas/sdk-handoff.schema.json`. A placeholder example is committed at `examples/sdk-handoff.json`.

```json
{
  "schema_version": "1",
  "status": "placeholder_for_maintainer_deployment",
  "network": {
    "name": "testnet",
    "rpc_url": "https://soroban-testnet.stellar.org",
    "network_passphrase": "Test SDF Network ; September 2015"
  },
  "contract_id": "CONTRACT_ID_PLACEHOLDER",
  "interface_version": "0.1",
  "interface_schema_ref": "schemas/vault-interface-v0.1.json",
  "event_schema_ref": "schemas/vault-event.schema.json",
  "initialization": {
    "admin": "ADDRESS_PLACEHOLDER",
    "deposit_token": "ADDRESS_PLACEHOLDER",
    "reward_token": "ADDRESS_PLACEHOLDER"
  },
  "maintainer_deployment_boundary": {
    "no_secrets_included": true,
    "real_deployment_performed": false
  }
}
```

### Maintainer Post-Deployment Handoff Workflow

After contract deployment, the maintainer completes the following steps:

1. **Populate Real Values:**
   Update the handoff JSON file with:
   - Deployed contract ID (`contract_id`, e.g. `C...`)
   - Initialization parameters (`admin`, `deposit_token`, `reward_token` addresses, e.g. `G...` / `C...`)
   - Target network parameters (`name`, `rpc_url`, `network_passphrase`)
   - Update `status` to `"ready_for_sdk_consumption"`
   - Set `maintainer_deployment_boundary.real_deployment_performed` to `true`

2. **Validate Handoff Artifact:**
   Run the validation script to ensure schema compliance and verify that no private keys or secrets were exposed:
   ```bash
   python3 scripts/validate-sdk-handoff.py path/to/sdk-handoff.json
   ```

3. **Deliver to SDK Repo:**
   Publish or commit the validated handoff artifact for consumption by `axionvera-sdk`.

---

## 4. Contract ID Registry

To maintain a clear, non-secret record of deployed contracts across environments (local, testnet, futurenet, mainnet), the repository maintains a contract ID registry defined by `schemas/contract-id-registry.schema.json`.

An example placeholder registry is committed at `examples/contract-id-registry.json`.

### Maintainer Registry Recording Workflow

Post-deployment, the maintainer records deployment details in the registry:

1. Update target environment entry (e.g. `testnet`) status to `"deployed"`.
2. Record the deployed contract ID (`contract_id`, e.g. `C...`), compiled binary hash (`wasm_sha256`), deployment ISO timestamp (`deployed_at`), and deployer account (`deployer_address`).
3. Set `maintainer_deployment_boundary.real_deployments_recorded` to `true`.
4. Run the validation script to verify non-secret compliance and schema structure:
   ```bash
   python3 scripts/validate-contract-id-registry.py path/to/contract-id-registry.json
   ```
   Publish or commit the validated handoff artifact for consumption by `axionvera-sdk`.
