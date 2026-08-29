# Security Policy

Axionvera Network is the smart contract and network foundation for transparent vaults, rewards, and community payouts.

Security is important because this repository contains contract logic, reward accounting, authorization behavior, and SDK-facing interfaces.

---

## Current Status

Axionvera Network is under active development.

The codebase is not yet production-audited.

Do not treat the current contracts as production-ready until a formal security review has been completed.

---

## Supported Security Scope

Security reports may include:

- vault contract vulnerabilities
- authorization bypasses
- unsafe initialization behavior
- incorrect deposit accounting
- incorrect withdrawal accounting
- incorrect reward calculation
- incorrect reward claim behavior
- state corruption risks
- event/state mismatch issues
- SDK-to-contract interface risks
- network-node configuration risks
- test gaps affecting contract safety

---

## Reporting a Vulnerability

Please do not open a public GitHub issue for serious vulnerabilities.

Report serious security concerns privately through GitHub Security Advisories where available, or contact the project maintainers directly.

When reporting a vulnerability, include:

- affected file or contract
- description of the issue
- steps to reproduce
- expected behavior
- actual behavior
- potential impact
- suggested fix, if available

---

## Security Expectations for Contributors

Contributors should:

- add tests for every new or changed function
- test invalid input and failure paths
- test authorization behavior
- test uninitialized contract behavior
- test failed calls do not corrupt state
- keep accounting logic explicit
- keep state transitions predictable
- avoid silent failures
- avoid introducing unaudited privileged behavior

---

## Contract Safety Checklist

For contract changes, contributors must ensure:

- initialization cannot be repeated unsafely
- admin or owner behavior is protected
- deposits update balances correctly
- withdrawals cannot exceed balances
- total deposits remain consistent
- reward claims cannot be duplicated incorrectly
- failed calls do not emit misleading events
- failed calls do not mutate state unexpectedly
- public methods return predictable values
- event formats remain stable

### Maintainer Review Checklist

Before testnet deployment, maintainers should complete a formal review using the [Vault Security Review Checklist Template](docs/vault-security-review-template.md). See the [example checklist](docs/vault-security-review-example.md) for how this should be filled out.

---

## Dependency Security

When changing dependencies:

- keep dependency changes minimal
- avoid unnecessary new packages
- explain why the dependency is needed
- check that local tests still pass
- do not run broad automated fixes without reviewing the result

---

## Local Security Checks

Before opening a PR, run:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings