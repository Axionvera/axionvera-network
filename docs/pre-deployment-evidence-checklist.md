# Pre-Deployment Evidence Checklist

This checklist provides a structured framework for maintainers to gather and review evidence before approving testnet deployment. It ensures that all required tests, documentation, schemas, scripts, examples, and security checks are complete and verified.

> **Important:** This checklist is for maintainer decision-making before deployment. Completing this checklist does not make the contract production-ready or production-audited. The current codebase has not completed a formal security audit.

## Purpose

The pre-deployment evidence checklist:
- Captures required commands and their execution results
- Documents expected artifacts and their validation status
- Provides a structured review process for maintainer approval
- Identifies known limitations and risks
- Ensures no production audit claims are made

## Checklist Structure

The checklist is organized into the following sections:

1. **Test Evidence** - Unit tests, integration tests, and smoke test results
2. **Documentation Evidence** - Required documentation completeness and accuracy
3. **Schema Evidence** - JSON schema validation and version consistency
4. **Script Evidence** - Build, deployment, and validation script verification
5. **Example Evidence** - Example configuration and data file validation
6. **Security Review Evidence** - Security review completion and findings
7. **Release Packet Evidence** - Release packet generation and validation
8. **Known Limitations** - Documented limitations and risk assessment
9. **Maintainer Approval** - Final approval decision and rationale

## Evidence Collection

### 1. Test Evidence

#### Unit Tests
- **Command:** `cargo test --workspace --all-targets`
- **Expected Result:** All tests pass (70+ vault contract tests, 13 reward tests, network-node tests)
- **Evidence:** Test output showing all tests passed
- **Status:** [ ] Passed / [ ] Failed / [ ] Skipped
- **Notes:** __________________________________________________________

#### Integration Tests
- **Command:** `cargo test --workspace --all-targets --test integration`
- **Expected Result:** All integration tests pass
- **Evidence:** Test output showing integration tests passed
- **Status:** [ ] Passed / [ ] Failed / [ ] Skipped
- **Notes:** __________________________________________________________

#### Mocked Smoke Tests
- **Command:** `./scripts/run-mocked-smoke-test.sh`
- **Expected Result:** Mocked smoke test flow completes successfully
- **Evidence:** Smoke test output showing lifecycle verification passed
- **Status:** [ ] Passed / [ ] Failed / [ ] Skipped
- **Notes:** __________________________________________________________

### 2. Documentation Evidence

#### Required Documentation
- **Files to Verify:**
  - [ ] `docs/maintainer-handoff-guide.md`
  - [ ] `docs/testnet-deployment-checklist.md`
  - [ ] `docs/deployment-failure-and-recovery-runbook.md`
  - [ ] `docs/testnet-configuration.md`
  - [ ] `docs/post-deployment-verification.md`
  - [ ] `docs/mock-post-deployment-smoke-test.md`
  - [ ] `docs/vault-security-review-template.md`
  - [ ] `docs/vault-security-review-example.md`
- **Verification Method:** Manual review of each document
- **Status:** [ ] Complete / [ ] Incomplete
- **Notes:** __________________________________________________________

#### Documentation Accuracy
- **Verification:** Cross-reference documentation with actual code and scripts
- **Status:** [ ] Accurate / [ ] Inaccurate / [ ] Needs Update
- **Notes:** __________________________________________________________

### 3. Schema Evidence

#### Schema Files Present
- **Files to Verify:**
  - [ ] `schemas/vault-interface.schema.json`
  - [ ] `schemas/vault-interface-v0.1.json`
  - [ ] `schemas/vault-event.schema.json`
  - [ ] `schemas/sdk-handoff.schema.json`
  - [ ] `schemas/contract-id-registry.schema.json`
  - [ ] `schemas/deployment-artifact.schema.json`
  - [ ] `schemas/mock-vault-deployment.schema.json`
  - [ ] `schemas/build-metadata.schema.json`
- **Status:** [ ] All Present / [ ] Missing Files
- **Notes:** __________________________________________________________

#### Schema Validation
- **Command:** Run individual validation scripts for each schema
- **Expected Result:** All schemas validate successfully
- **Evidence:** Validation script outputs
- **Status:** [ ] Valid / [ ] Invalid
- **Notes:** __________________________________________________________

#### Schema Version Consistency
- **Verification:** Check that schema versions are consistent across files
- **Status:** [ ] Consistent / [ ] Inconsistent
- **Notes:** __________________________________________________________

### 4. Script Evidence

#### Build Scripts
- **Files to Verify:**
  - [ ] `scripts/build-vault-wasm.sh` - Executable and functional
  - [ ] `scripts/deploy-vault-template.sh` - Executable and functional
- **Test Command:** `./scripts/build-vault-wasm.sh`
- **Status:** [ ] Functional / [ ] Non-functional
- **Notes:** __________________________________________________________

#### Validation Scripts
- **Files to Verify:**
  - [ ] `scripts/validate-testnet-config.sh`
  - [ ] `scripts/validate-sdk-handoff.py`
  - [ ] `scripts/validate-contract-id-registry.py`
  - [ ] `scripts/validate-mock-vault-deployment.py`
  - [ ] `scripts/validate-vault-initialization-input.py`
  - [ ] `scripts/validate-deployment-artifact.py`
  - [ ] `scripts/validate-mvp-demo-scenario.py`
- **Test Command:** Run each validation script against its example
- **Status:** [ ] All Functional / [ ] Some Non-functional
- **Notes:** __________________________________________________________

#### Verification Scripts
- **Files to Verify:**
  - [ ] `scripts/verify-vault-deployment.sh`
  - [ ] `scripts/verify-vault-deployment.py`
- **Status:** [ ] Functional / [ ] Non-functional
- **Notes:** __________________________________________________________

#### Release Packet Scripts
- **Files to Verify:**
  - [ ] `scripts/generate-release-packet.py`
  - [ ] `scripts/validate-release-packet.py`
  - [ ] `scripts/test-release-packet.py`
- **Test Command:** `python3 scripts/test-release-packet.py`
- **Status:** [ ] All Functional / [ ] Some Non-functional
- **Notes:** __________________________________________________________

### 5. Example Evidence

#### Example Files Present
- **Files to Verify:**
  - [ ] `examples/testnet-config.json`
  - [ ] `examples/build-metadata.json`
  - [ ] `examples/deployment-artifact.json`
  - [ ] `examples/sdk-handoff.json`
  - [ ] `examples/contract-id-registry.json`
  - [ ] `examples/vault-initialization-input.json`
  - [ ] `examples/mock-post-deployment-smoke-input.json`
  - [ ] `examples/mock-post-deployment-smoke-output.json`
  - [ ] `examples/post-deployment-verification.json`
  - [ ] `examples/mvp-demo-scenario/` (directory)
  - [ ] `examples/vault-events/` (directory)
  - [ ] `examples/vault-interface/` (directory)
  - [ ] `examples/deployment-failures/` (directory)
  - [ ] `examples/vault-deployment/` (directory)
- **Status:** [ ] All Present / [ ] Missing Files
- **Notes:** __________________________________________________________

#### Example Validation
- **Command:** Run validation scripts against examples
- **Expected Result:** All examples validate successfully
- **Evidence:** Validation script outputs
- **Status:** [ ] Valid / [ ] Invalid
- **Notes:** __________________________________________________________

### 6. Security Review Evidence

#### Security Review Template
- **File:** `docs/vault-security-review-template.md`
- **Status:** [ ] Reviewed / [ ] Not Reviewed
- **Findings:** __________________________________________________________

#### Security Review Example
- **File:** `docs/vault-security-review-example.md`
- **Status:** [ ] Reviewed / [ ] Not Reviewed
- **Findings:** __________________________________________________________

#### Known Security Issues
- **Issues Identified:** __________________________________________________________
- **Mitigation Status:** [ ] Mitigated / [ ] Accepted Risk / [ ] Blocked
- **Notes:** __________________________________________________________

#### Secret Exclusion Verification
- **Verification:** Confirm no secrets or private keys in repository
- **Method:** Manual review + pattern-based validation
- **Status:** [ ] Verified / [ ] Not Verified
- **Notes:** __________________________________________________________

### 7. Release Packet Evidence

#### Release Packet Generation
- **Command:** `python3 scripts/generate-release-packet.py --output-dir release-packet-<version>`
- **Expected Result:** Release packet generated successfully
- **Evidence:** Release packet directory with manifest.json and artifacts/
- **Status:** [ ] Generated / [ ] Failed
- **Notes:** __________________________________________________________

#### Release Packet Validation
- **Command:** `python3 scripts/validate-release-packet.py release-packet-<version>/manifest.json`
- **Expected Result:** All validations pass
- **Evidence:** Validation output showing all checks passed
- **Status:** [ ] Valid / [ ] Invalid
- **Notes:** __________________________________________________________

#### Release Packet Review
- **Documentation Review:** [ ] Complete / [ ] Incomplete
- **Schema Review:** [ ] Complete / [ ] Incomplete
- **Example Review:** [ ] Complete / [ ] Incomplete
- **Script Review:** [ ] Complete / [ ] Incomplete
- **Notes:** __________________________________________________________

### 8. Known Limitations

#### Current Codebase Limitations
- **Limitation 1:** Current vault records accounting values without transferring tokens
- **Impact:** [ ] Low / [ ] Medium / [ ] High
- **Mitigation:** __________________________________________________________

- **Limitation 2:** `set_claimable_reward` and `set_reward_balance` methods do not require authorization
- **Impact:** [ ] Low / [ ] Medium / [ ] High
- **Mitigation:** __________________________________________________________

- **Limitation 3:** No formal security audit completed
- **Impact:** [ ] High
- **Mitigation:** This is testnet-only deployment; not production-ready

#### Testnet-Specific Limitations
- **Limitation:** Stellar testnet data may be reset
- **Impact:** [ ] Medium
- **Mitigation:** Document testnet-only nature clearly

#### Documentation Limitations
- **Limitation:** __________________________________________________________
- **Impact:** [ ] Low / [ ] Medium / [ ] High
- **Mitigation:** __________________________________________________________

### 9. Maintainer Approval

#### Pre-Deployment Checklist Completion
- **All Sections Complete:** [ ] Yes / [ ] No
- **All Evidence Collected:** [ ] Yes / [ ] No
- **All Validations Passed:** [ ] Yes / [ ] No

#### Risk Assessment
- **Overall Risk Level:** [ ] Low / [ ] Medium / [ ] High
- **Risk Rationale:** __________________________________________________________

#### Deployment Decision
- **Approval Status:** [ ] Approved / [ ] Deferred / [ ] Rejected
- **Approval Date:** ____________________
- **Approver Name:** ____________________
- **Approval Rationale:** __________________________________________________________

#### Conditions for Deployment
- **Pre-conditions:** __________________________________________________________
- **Post-Deployment Verification Required:** [ ] Yes / [ ] No
- **Rollback Plan:** __________________________________________________________

## Appendix: Required Commands Reference

### Quality Checks
```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Release Readiness
```bash
./scripts/release-readiness-check.sh --full
```

### Release Packet
```bash
python3 scripts/generate-release-packet.py --output-dir release-packet-<version>
python3 scripts/validate-release-packet.py release-packet-<version>/manifest.json
python3 scripts/test-release-packet.py
```

### Validation Scripts
```bash
./scripts/validate-testnet-config.sh
python3 scripts/validate-sdk-handoff.py examples/sdk-handoff.json
python3 scripts/validate-contract-id-registry.py examples/contract-id-registry.json
python3 scripts/validate-mock-vault-deployment.py
python3 scripts/validate-vault-initialization-input.py
python3 scripts/validate-deployment-artifact.py
python3 scripts/validate-mvp-demo-scenario.py
```

### Smoke Tests
```bash
./scripts/run-mocked-smoke-test.sh
```

## Related Documentation

- [Maintainer Handoff Guide](./maintainer-handoff-guide.md) - Security boundaries and deployment workflow
- [Testnet Deployment Checklist](./testnet-deployment-checklist.md) - Step-by-step deployment process
- [Deployment Failure and Recovery Runbook](./deployment-failure-and-recovery-runbook.md) - Troubleshooting deployment issues
- [Release Packet Generator](./release-packet-generator.md) - Artifact collection and validation
- [Security Policy](../SECURITY.md) - Security guidelines and reporting
