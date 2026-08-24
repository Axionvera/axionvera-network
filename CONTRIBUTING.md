# Contributing to Axionvera Network

Thank you for contributing to Axionvera Network.

Axionvera Network is the smart contract and network foundation for transparent vaults, rewards, and community payouts.

This repo is part of the restarted-codebase campaign. The goal is to keep the foundation clean, tested, reliable, and easy to extend.

---

## Contribution Rules

Before working on an issue, make sure:

- the issue is assigned to you
- the issue requirements are clear
- your changes stay within the issue scope
- every new or changed function includes unit tests
- all local checks pass before opening a PR

---

## Testing Standard

Every new function or implementation must include tests.

Tests should cover:

- happy path
- invalid input
- important edge cases
- expected failure behavior
- authorization behavior where applicable
- state consistency where applicable

For contract work, tests should usually be added in the relevant Rust crate:

```text
contracts/vault-contract/src/lib.rs
contracts/rewards/src/lib.rs
network-node/src/lib.rs
```

---

## Required Local Checks

Before opening a PR, run:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

All checks must pass.

These are the same checks the CI pipeline runs on every PR and every push to `main`.

For a full explanation of what each check does, how to reproduce CI failures locally, and how to fix common failures, see [docs/ci-and-local-checks.md](./docs/ci-and-local-checks.md).

---

## Commit Guidelines

Keep commits focused and readable.

Good examples:

```text
Add vault admin query tests
Implement network config validation
Document SDK-to-network interface
Add reward accounting edge-case tests
```

Avoid:

```text
fix stuff
updates
misc changes
big commit
```

---

## Pull Request Guidelines

Each PR should include:

- a short summary of the change
- the issue number it closes
- tests added or updated
- confirmation that checks passed
- screenshots or logs if useful

Example PR body:

```text
Closes #123

Summary:
- Added vault owner query
- Added tests for initialized and uninitialized behavior
- Updated SDK interface docs

Checks:
- cargo fmt passed
- cargo check passed
- cargo test passed
- cargo clippy passed
```

---

## Scope Control

Keep PRs focused.

A PR should not mix unrelated changes such as:

- contract logic changes
- unrelated documentation updates
- formatting-only changes
- config changes
- new feature work outside the assigned issue

If a related bug is discovered, mention it in the PR and create a separate issue if needed.

---

## Contract Safety Expectations

For Soroban contract changes:

- keep state transitions explicit
- avoid silent failures
- validate inputs clearly
- test authorization paths
- test uninitialized behavior
- test failed calls do not corrupt state
- test accounting consistency
- keep events stable and predictable

---

## Documentation Expectations

Update documentation when changes affect:

- public contract methods
- method arguments
- return values
- emitted events
- SDK integration expectations
- setup or development commands
- security assumptions

---

## Review Expectations

Maintainers may ask for changes if:

- tests are missing
- checks fail
- the PR scope is too broad
- the implementation does not match the issue
- public method behavior is unclear
- docs are outdated
- edge cases are not covered

---

## Local Development Reminder

Use this command before pushing:

```bash
cargo fmt --all -- --check && \
cargo check --workspace --all-targets && \
cargo test --workspace --all-targets && \
cargo clippy --workspace --all-targets -- -D warnings
```

Clean commits. Tested code. Stronger foundation.
