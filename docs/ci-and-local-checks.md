# CI Workflow and Local Checks

This document explains the Axionvera Network CI pipeline, what each check does, how to reproduce every check locally before opening a PR, and how to fix the most common failures.

---

## Why CI Exists

Every pull request and every push to `main` runs the same set of quality checks on GitHub Actions.

The goal is to catch formatting issues, compilation errors, broken tests, and Clippy warnings before they reach the main branch.

A PR cannot be merged unless all CI checks pass.

Running the same checks locally before pushing means you fix problems on your machine instead of waiting for the pipeline to fail.

---

## CI Workflow File

The pipeline is defined in `.github/workflows/ci.yml`.

It runs on:

- every pull request
- every push to `main`

The job name is `rust-quality` and it runs on `ubuntu-latest`.

Steps in order:

1. Checkout the repository
2. Install the stable Rust toolchain with `rustfmt` and `clippy` components
3. Cache Rust dependencies
4. Check formatting
5. Check workspace compilation
6. Run tests
7. Run Clippy

---

## The Four Checks

### 1. Formatting

**CI command:**

```bash
cargo fmt --all -- --check
```

**What it does:**

Checks that every Rust file in the workspace is formatted according to the standard `rustfmt` rules.

This command does not reformat files. It only checks and exits with a non-zero code if anything is unformatted.

**How to fix formatting failures:**

Run the formatter to apply changes automatically:

```bash
cargo fmt --all
```

Then verify the check passes:

```bash
cargo fmt --all -- --check
```

Commit the formatted files before pushing.

---

### 2. Workspace Check

**CI command:**

```bash
cargo check --workspace --all-targets
```

**What it does:**

Compiles every crate in the workspace and every build target (lib, bins, tests, examples, benchmarks) without producing output binaries.

This is faster than a full build and catches compilation errors, missing imports, type mismatches, and broken dependencies.

**How to fix check failures:**

Read the error output carefully. Each error includes the file path and line number.

Common causes:

- missing `use` imports
- type mismatches
- changed function signatures that were not updated in callers
- missing trait implementations
- removed or renamed items still referenced elsewhere

Fix all errors until the command exits cleanly.

---

### 3. Tests

**CI command:**

```bash
cargo test --workspace --all-targets
```

**What it does:**

Runs every test in the workspace across all crates and all targets.

**How to fix test failures:**

Read the test output. Failing tests are shown with the test name, the expected value, and the actual value.

Common causes:

- logic error in the implementation
- test setup that no longer matches the current state
- a function was changed but the test was not updated
- an edge case that was not handled

Fix the implementation or the test depending on which is wrong. Do not delete tests to make the suite pass.

To run tests for a single crate while debugging:

```bash
cargo test -p axionvera-vault-contract
cargo test -p axionvera-rewards
cargo test -p axionvera-network-node
```

To run a single test by name:

```bash
cargo test -p axionvera-vault-contract test_deposit
```

---

### 4. Clippy

**CI command:**

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

**What it does:**

Runs the Rust linter across every crate and every target.

The `-D warnings` flag treats every Clippy warning as a hard error. A single warning will fail the CI job.

**How to fix Clippy failures:**

Read the Clippy output. Each warning includes the file path, the line number, a description, and usually a suggested fix.

Common causes:

- unnecessary clones or allocations
- redundant closures
- match arms that can be simplified
- unused variables or imports
- needless borrows
- incorrect use of iterators

Apply the suggested fix or suppress the lint with `#[allow(...)]` only when there is a deliberate reason and a comment explaining why.

Do not suppress lints to make the job pass without understanding them.

---

## Full Local Reproduction Command

Run all four checks in order before pushing or opening a PR:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Run them as a single chained command to stop on the first failure:

```bash
cargo fmt --all -- --check && \
cargo check --workspace --all-targets && \
cargo test --workspace --all-targets && \
cargo clippy --workspace --all-targets -- -D warnings
```

All four commands must exit cleanly before a PR is opened.

---

## Recommended Check Order

Run the checks in this order:

1. **Formatting first** — formatting errors are the fastest to fix and unformatted code can make other error output harder to read.
2. **Workspace check second** — catch compilation errors before running tests.
3. **Tests third** — verify behavior is correct after compilation passes.
4. **Clippy last** — clean up lint warnings after tests pass.

---

## Local Pre-Commit Hook

This repository uses Husky to run a pre-commit hook automatically when you commit.

The pre-commit hook runs:

1. Formatting check
2. Soroban contract check targeting `wasm32-unknown-unknown`
3. Clippy on the Soroban contract targeting `wasm32-unknown-unknown`
4. Workspace check on all native crates
5. Tests on all native crates
6. Clippy on all native crates

If any step fails, the commit is blocked until the failure is fixed.

The hook is defined in `.husky/pre-commit`.

Note that the pre-commit hook splits native crates and the Soroban contract because the vault contract must be checked as a WASM target rather than as a native binary. The CI pipeline uses workspace-wide commands without that split because it runs in a compatible environment. Both approaches validate the same code quality.

---

## Common Questions

**Why does my PR fail CI even though my code works locally?**

The most common reason is that formatting was not checked before pushing. Run `cargo fmt --all -- --check` and fix any output before committing.

Another common reason is that a Clippy warning was introduced. Run `cargo clippy --workspace --all-targets -- -D warnings` and address every warning.

**Can I ignore a Clippy warning?**

Only with a clear reason. Use `#[allow(clippy::lint_name)]` with a comment explaining why the suppression is intentional. Do not use blanket suppression.

**Can I skip the pre-commit hook?**

Do not use `--no-verify` to skip the hook. The hook exists to catch issues before they reach CI. Skipping it does not remove the CI requirement.

**My test is failing in CI but passing locally. What should I check?**

- Make sure your local Rust toolchain is on the stable channel: `rustup show`
- Make sure you have no uncommitted changes that were not included in the pushed branch
- Check if the test depends on state or ordering that differs between environments

---

## Summary

| Check | Command | What fails it |
|---|---|---|
| Formatting | `cargo fmt --all -- --check` | Any unformatted Rust file |
| Workspace | `cargo check --workspace --all-targets` | Compilation errors |
| Tests | `cargo test --workspace --all-targets` | Failing assertions |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | Any lint warning |

All four checks must pass before a PR can be merged.
