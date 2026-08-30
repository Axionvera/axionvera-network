# Release Readiness Checklist

This document describes the release readiness validation script for the Axionvera Network repository. It is intended for maintainer review before any deployment or release tag.

## Purpose

Before maintainer deployment, the repo needs a repeatable way to verify required files, documentation, schemas, examples, scripts, and local commands. The checklist ensures all artifacts are present and quality checks pass without secrets or privileged actions.

## Script

- **Location:** `scripts/release-readiness-check.sh`
- **Usage (quick):** `./scripts/release-readiness-check.sh`
- **Usage (full):** `./scripts/release-readiness-check.sh --full`

The quick mode checks required files and runs `cargo fmt --check` and `cargo check`. The full mode also runs `cargo test` and `cargo clippy`.

## What It Checks

1. **Documentation** — maintainer handoff guide, deployment checklist, CI checks, testnet configuration, README, license, contributing guide, security policy.
2. **Schemas** — `build-metadata.schema.json`, `mock-vault-deployment.schema.json`, `vault-event.schema.json`, `sdk-handoff.schema.json`, `contract-id-registry.schema.json`.
3. **Examples** — build metadata, testnet config, vault deployment examples, vault event examples, sdk handoff example, contract id registry example.
4. **Scripts** — build WASM, deploy template, validate testnet config, validate mock vault deployment, validate SDK handoff, validate contract ID registry.
2. **Schemas** — `build-metadata.schema.json`, `mock-vault-deployment.schema.json`, `vault-event.schema.json`, `sdk-handoff.schema.json`.
3. **Examples** — build metadata, testnet config, vault deployment examples, vault event examples, sdk handoff example.
4. **Scripts** — build WASM, deploy template, validate testnet config, validate mock vault deployment, validate SDK handoff.
5. **Project files** — `.env.example`, `Cargo.toml`, `Cargo.lock`.
6. **Local commands** — formatting, workspace compilation, tests, and clippy (full mode only).

No secrets or privileged actions are required.

## Example Output — Success (Quick)

```text
=== Axionvera Network Release Readiness Checklist ===
Project root: /home/user/axionvera-network
Mode: QUICK

--- Required Documentation ---
[OK] Maintainer Handoff Guide
[OK] Testnet Deployment Checklist
[OK] CI and Local Checks
[OK] Testnet Configuration
[OK] README
[OK] License
[OK] Contributing Guide
[OK] Security Policy

--- Required Schemas ---
[OK] Build Metadata Schema
[OK] Mock Vault Deployment Schema
[OK] Vault Event Schema
[OK] SDK Handoff Schema
[OK] Contract ID Registry Schema

--- Required Examples ---
[OK] Build Metadata Example
[OK] Testnet Config Example
[OK] Vault Deployment Examples
[OK] Vault Event Examples
[OK] SDK Handoff Example
[OK] Contract ID Registry Example

--- Required Scripts ---
[OK] Build Vault WASM
[OK] Deploy Vault Template
[OK] Validate Testnet Config
[OK] Validate Mock Vault Deployment
[OK] Validate SDK Handoff
[OK] Validate Contract ID Registry

--- Required Project Files ---
[OK] .env Example
[OK] Workspace Cargo Manifest
[OK] Locked Dependencies

--- Local Command Checks ---
Checking cargo fmt --check ...
[OK] cargo fmt --check
Checking cargo check ...
[OK] cargo check
[SKIPPED] cargo test (run with --full)
[SKIPPED] cargo clippy (run with --full)

=== RELEASE READINESS: ALL CHECKS PASSED ===
```

## Example Output — Success (Full)

The full mode produces the same file checks plus:

```text
Checking cargo test ...
[OK] cargo test
Checking cargo clippy ...
[OK] cargo clippy

=== RELEASE READINESS: ALL CHECKS PASSED ===
```

## Example Output — Missing Items

If required files or commands fail, the script prints clear messages and exits with an error code:

```text
[MISSING] Maintainer Handoff Guide (docs/maintainer-handoff-guide.md)
[FAIL] cargo check (see /tmp/release_readiness_cargo_check.log)

=== RELEASE READINESS: 2 MISSING / FAILED ITEMS ===
```

## Tests

A practical dry-run test is provided at `tests/test-release-readiness.py`. It runs the checklist script and verifies that it exits cleanly with no missing items when the repository is intact.

```bash
python3 tests/test-release-readiness.py
```

## Integration

Refer to the [Maintainer Handoff Guide](./maintainer-handoff-guide.md) for the full contributor-to-maintainer workflow, and to the [Vault Contract Testnet Deployment Checklist](./testnet-deployment-checklist.md) for deployment-specific steps.
