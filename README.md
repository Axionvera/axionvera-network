<div align="center">

# Axionvera Network

**The on-chain protocol layer for programmable merchant incentives, rewards, vaults, and payouts on Stellar.**

Axionvera Network contains the Soroban smart contracts, protocol interfaces, deployment tooling, and network configuration that power Axionvera.

</div>

---

## Overview

Axionvera Network is the blockchain foundation of the Axionvera protocol.

The repository currently contains two primary Soroban contract systems:

- **Campaign Contract** — merchant-funded incentive campaigns, reward allocation, verifier authorization, claims, campaign lifecycle management, and treasury safeguards.
- **Vault Contract** — deposits, withdrawals, reward accounting, claims, and vault lifecycle functionality.

The Network repository defines the protocol behavior that higher-level Axionvera components, including the SDK, integrate with.

The current focus is on keeping this layer:

- deterministic
- well tested
- secure by default
- event-driven
- SDK-friendly
- explicitly versioned
- reproducibly deployable

---

## Current Status

The core Network foundation is implemented and tested.

### Campaign Contract

The Campaign Contract currently supports:

- campaign creation
- campaign funding
- configurable activation rules
- verifier registration and removal
- verifier-authorized reward allocation
- duplicate activation protection
- per-agent reward caps
- reward claiming
- campaign pause and resume
- campaign closure
- unused-funds calculation and withdrawal
- reserved reward accounting
- persistent-state TTL management
- stable contract events
- versioned SDK-facing interface metadata

The Campaign Contract has also been deployed and exercised through a complete live flow on Stellar testnet.

### Vault Contract

The Vault Contract currently supports:

- initialization
- owner/admin state
- deposits
- withdrawals
- reward accounting
- claimable reward management
- reward claims
- user balance queries
- total deposit queries
- authorization controls
- stable lifecycle events
- SDK-facing interface metadata

### Network Support

The repository also includes:

- network configuration and validation
- health helpers
- deployment artifact schemas
- contract ID registry
- deployment and recovery runbooks
- testnet configuration examples
- release tooling
- mock deployment flows
- Husky pre-commit checks
- GitHub Actions validation

---

## Repository Structure

```text
axionvera-network/
├── contracts/
│   ├── campaign-contract/
│   │   └── Merchant-funded campaign and reward protocol
│   │
│   ├── vault-contract/
│   │   └── Soroban vault and reward contract
│   │
│   └── rewards/
│       └── Shared reward calculation helpers
│
├── network-node/
│   └── Network configuration and health helpers
│
├── schemas/
│   └── Versioned protocol, deployment, and interface schemas
│
├── examples/
│   └── Deployment artifacts, registries, and integration examples
│
├── scripts/
│   └── Build, validation, deployment, and release tooling
│
├── docs/
│   └── Contract, deployment, and SDK integration documentation
│
├── .github/
│   └── GitHub Actions workflows
│
└── .husky/
    └── Local pre-commit checks
```

---

## Contracts

### `contracts/campaign-contract`

The Campaign Contract provides the core programmable incentive layer.

A campaign contains:

- an administrator
- a reward token
- a start and end time
- campaign lifecycle state
- funded, allocated, claimed, and withdrawn accounting
- a per-agent reward cap
- configurable activation rules
- authorized verifiers

Campaign lifecycle states are:

```text
Active
Paused
Closed
```

A typical reward flow is:

```text
Merchant creates campaign
        ↓
Merchant funds campaign
        ↓
Merchant configures activation rules
        ↓
Merchant authorizes verifier
        ↓
Verifier validates off-chain activity
        ↓
Verifier allocates reward on-chain
        ↓
Agent receives claimable reward
        ↓
Agent claims reward
```

Allocated rewards are reserved for agents and cannot subsequently be withdrawn by the campaign administrator.

After a campaign is closed, only genuinely unused funds may be withdrawn.

---

### `contracts/vault-contract`

The Vault Contract provides general-purpose deposit, withdrawal, and reward accounting functionality.

Current capabilities include:

- vault initialization
- owner/admin state
- deposit accounting
- withdrawal accounting
- claimable reward accounting
- reward claims
- user balance queries
- total deposit queries
- lifecycle events
- authorization checks
- initialization protection
- failure-path and edge-case coverage

---

### `contracts/rewards`

Shared reward calculation helpers.

Current capabilities include:

- proportional reward calculation
- pending reward calculation
- zero-value handling
- overflow-safe behavior
- large-value edge-case handling

---

### `network-node`

Network configuration and health support.

Current capabilities include:

- default network configuration
- config validation
- structured health status
- environment validation
- serialization support

---

## Stellar Testnet Deployments

The repository maintains non-secret deployment information in:

```text
examples/contract-id-registry.json
```

### Campaign Contract

| Field | Value |
|---|---|
| Network | Stellar Testnet |
| Contract ID | `CAAXCSTGNQ6S73XRXYSKAEEWZNVS7XWA4EF67DRWDPS2XFSXXA3AC2C6` |
| WASM SHA-256 | `cef31e82808155f38afd561bf4aa78290e3083a4251edbf3c709a91530b0ee3e` |
| Deployment timestamp | `2026-09-25T10:03:42Z` |
| Source commit | `2c51d8837fc7c303630346d0a9e895f39c4350` |

The complete deployment record is available at:

```text
examples/campaign-deployment-testnet.json
```

The recorded WASM hash is reproducible from the deployment source commit.

### Live Campaign Smoke Test

The deployed Campaign Contract has completed a full testnet lifecycle using Stellar native assets:

```text
Create campaign
      ↓
Fund with 10 XLM
      ↓
Add 1 XLM activation rule
      ↓
Authorize separate verifier
      ↓
Allocate 1 XLM reward
      ↓
Agent claims 1 XLM
      ↓
Pause campaign
      ↓
Resume campaign
      ↓
Close campaign
      ↓
Withdraw remaining 9 XLM
```

The final campaign contract balance and available unused balance were both verified as zero.

Transaction hashes and smoke-test evidence are recorded in:

```text
examples/campaign-deployment-testnet.json
```

---

## Contract Interfaces

Axionvera maintains versioned, machine-readable contract interface definitions so downstream SDKs and applications do not need to infer contract behavior directly from Rust source code.

### Campaign Interface

```text
schemas/campaign-interface-v0.1.json
schemas/campaign-interface.schema.json
docs/campaign-interface-schema.md
```

The Campaign interface records:

- public method names
- argument ordering
- return types
- initialization requirements
- authorization requirements
- logical mutability
- contract types
- contract errors
- emitted events
- event topics

### Vault Interface

```text
schemas/vault-interface-v0.1.json
schemas/vault-interface.schema.json
docs/vault-interface-schema.md
```

These interface definitions act as the compatibility boundary between the Network and SDK repositories.

---

## SDK Alignment

Axionvera Network is designed to be consumed through the Axionvera SDK.

The SDK can use the versioned interface definitions to provide typed access to protocol functionality without requiring application developers to construct raw Soroban invocations manually.

Campaign SDK integration is expected to cover:

- campaign reads
- campaign creation
- campaign funding
- activation-rule management
- verifier management
- reward allocation
- reward claims
- campaign lifecycle operations
- treasury reads and withdrawals
- event parsing
- protocol error mapping

Vault SDK integration covers:

- vault information
- user balances
- pending rewards
- deposits
- withdrawals
- reward claims
- lifecycle events

Public contract interfaces should remain stable once versioned. Breaking changes should require an explicit interface-version update.

See:

```text
docs/sdk-contract-interface.md
docs/campaign-interface-schema.md
docs/vault-interface-schema.md
```

---

## Building Soroban Contracts

Axionvera contracts target:

```text
wasm32v1-none
```

The Stellar CLI should be used for deployment-ready Soroban builds.

### Campaign Contract

```bash
stellar contract build \
  --package axionvera-campaign-contract \
  --locked
```

Output:

```text
target/wasm32v1-none/release/axionvera_campaign_contract.wasm
```

### Vault Contract

A repeatable build helper is provided:

```bash
./scripts/build-vault-wasm.sh
```

The script uses `stellar contract build` and generates build metadata containing:

- package name
- target
- artifact path
- SHA-256 hash
- build timestamp
- source commit

Output:

```text
target/wasm32v1-none/release/axionvera_vault_contract.wasm
```

---

## Local Development

Run the complete local Rust validation suite before committing:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

For deployment-target validation:

```bash
cargo clippy \
  -p axionvera-campaign-contract \
  --target wasm32v1-none \
  --release \
  -- -D warnings
```

```bash
cargo clippy \
  -p axionvera-vault-contract \
  --target wasm32v1-none \
  --release \
  -- -D warnings
```

---

## Quality Standard

New or changed implementation behavior should include appropriate test coverage.

Tests should cover, where applicable:

- happy paths
- invalid input
- authorization
- state transitions
- accounting invariants
- failure behavior
- event behavior
- boundary conditions
- arithmetic safety
- persistent-state lifecycle

Contract changes should not be merged unless the full workspace checks pass.

---

## Local Commit Checks

This repository uses Husky pre-commit checks.

The pre-commit workflow validates:

1. formatting
2. Soroban contract compilation
3. Soroban target-specific Clippy
4. workspace compilation
5. workspace tests
6. workspace Clippy

This provides an additional validation layer before changes reach GitHub.

---

## CI Pipeline

GitHub Actions runs validation on pull requests and pushes to `main`.

The core checks are:

| Check | Command |
|---|---|
| Formatting | `cargo fmt --all -- --check` |
| Compilation | `cargo check --workspace --all-targets` |
| Tests | `cargo test --workspace --all-targets` |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` |

All required checks must pass before merge.

See:

```text
docs/ci-and-local-checks.md
```

---

## Testnet Configuration

The repository contains non-secret Stellar testnet configuration examples.

Relevant files include:

```text
.env.example
examples/testnet-config.json
docs/testnet-configuration.md
```

Network-node configuration includes:

```text
AXIONVERA_NETWORK_NAME
AXIONVERA_RPC_URL
AXIONVERA_ENVIRONMENT
```

Validate the committed configuration examples with:

```bash
./scripts/validate-testnet-config.sh
```

Private keys and deployment secrets must never be committed.

---

## Deployment Tooling

The repository includes maintainer-oriented deployment and verification tooling.

Key resources include:

```text
docs/testnet-deployment-checklist.md
docs/pre-deployment-evidence-checklist.md
docs/maintainer-handoff-guide.md
docs/deployment-failure-and-recovery-runbook.md
docs/post-deployment-verification.md
docs/mock-post-deployment-smoke-test.md
```

Supporting schemas, examples, and validators are stored under:

```text
schemas/
examples/
scripts/
```

### Contract ID Registry

Deployment identities are maintained in:

```text
examples/contract-id-registry.json
```

Validate with:

```bash
python3 scripts/validate-contract-id-registry.py \
  examples/contract-id-registry.json
```

### Deployment Artifacts

Deployment evidence records:

- contract ID
- network
- deployer address
- WASM hash
- deployment timestamp
- initialization status
- source commit
- relevant transaction hashes

Campaign testnet evidence is currently stored in:

```text
examples/campaign-deployment-testnet.json
```

---

## Contract Design Principles

Axionvera Network aims to keep protocol behavior:

- explicit
- deterministic
- testable
- predictable
- authorization-aware
- accounting-safe
- event-driven
- SDK-friendly
- easy to audit
- easy to extend deliberately

Protocol complexity should be introduced only when it provides clear product value or strengthens protocol safety.

---

## Security

Axionvera Network is under active development.

The current contracts have extensive automated test coverage and live testnet validation, but the codebase has **not yet completed a formal independent security audit**.

Do not treat the current contracts as production-audited.

Private keys, secret seeds, and signing credentials must never be committed to this repository.

For security guidance, see:

```text
SECURITY.md
```

---

## Contributing

Contributions are welcome through assigned issues.

Before opening a pull request:

- ensure the issue is assigned to you
- keep the PR focused
- add or update tests where behavior changes
- run all required local checks
- document public behavior changes
- include a clear PR summary
- reference the relevant issue

See:

```text
CONTRIBUTING.md
```

for full contribution guidance.

---

## License

This project is licensed under the MIT License.

See:

```text
LICENSE
```

---

<div align="center">

**Axionvera Network — programmable incentives, transparent rewards, and verifiable on-chain accounting.**

</div>
