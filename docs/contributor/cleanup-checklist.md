# Contributor Cleanup Checklist

This checklist helps ensure that all contributions to the Axionvera Network repo maintain consistency, quality, and long-term maintainability. Before submitting a PR, review each section and verify your changes meet the standards.

---

## 1. Module Boundaries

### 1.1 Module Organization
- [ ] Each module has a clear, single responsibility
- [ ] Module boundaries are enforced (no circular dependencies)
- [ ] Public API surface is minimal and well-defined
- [ ] Internal implementation details are properly encapsulated
- [ ] `mod.rs` files clearly re-export public items

### 1.2 Dependency Management
- [ ] No unnecessary dependencies added
- [ ] Dependencies are pinned to specific versions (not `*`)
- [ ] No duplicate dependencies (run `cargo tree -d` to check)
- [ ] Development dependencies are in `[dev-dependencies]`

### 1.3 File Placement
- [ ] New files are placed in logical locations
- [ ] Related code is grouped together (e.g., storage, events, errors)
- [ ] No orphaned or unused files remain

---

## 2. Error Handling

### 2.1 Error Types
- [ ] All errors use the project's error type (`ContractError` or similar)
- [ ] Error variants are descriptive and cover all failure cases
- [ ] Error messages are user-friendly and actionable
- [ ] No `unwrap()` or `expect()` in production code (use `?` or proper handling)
- [ ] No `panic!` in contract logic (use `ContractError` instead)

### 2.2 Error Propagation
- [ ] Errors are propagated correctly with proper context
- [ ] Errors are logged appropriately (not just swallowed)
- [ ] Edge cases (e.g., division by zero, overflow) are handled gracefully

---

## 3. Configuration

### 3.1 Config Files
- [ ] New config values have sensible defaults
- [ ] Config values are documented
- [ ] Environment variable names follow consistent naming patterns
- [ ] `.env.example` is updated with any new variables

### 3.2 Constants
- [ ] Magic numbers are extracted as named constants
- [ ] Constants are documented with their purpose
- [ ] No hardcoded values that should be configurable

---

## 4. Testing

### 4.1 Unit Tests
- [ ] All new functions have unit tests
- [ ] Edge cases are covered (empty, overflow, invalid inputs)
- [ ] Tests are deterministic (no flaky tests)
- [ ] Tests are properly isolated (no test pollution)

### 4.2 Integration Tests
- [ ] Contract behavior is tested in `tests/` directory
- [ ] Realistic scenarios are covered
- [ ] Multi-contract interactions are tested
- [ ] Snapshot tests are updated when state changes

### 4.3 Fuzz Tests
- [ ] Critical validation logic has fuzz tests
- [ ] Property-based tests use `proptest` or `fast-check`
- [ ] Fuzz tests generate realistic malformed inputs
- [ ] Fuzz tests never throw unhandled exceptions

### 4.4 Test Coverage
- [ ] Test coverage meets the project's threshold (≥95%)
- [ ] Uncovered lines have an explanation or are marked with `#[cfg(not(test))]`
- [ ] No commented-out tests remain

---

## 5. Peer Logic & Consensus

### 5.1 State Management
- [ ] State transitions are clearly defined and documented
- [ ] State invariants are maintained (e.g., `total_deposits` equals sum of user deposits)
- [ ] No state corruption can occur from reentrant calls

### 5.2 Consensus Rules
- [ ] Protocol-specific rules are correctly implemented
- [ ] Validation logic is complete and secure
- [ ] No assumptions about caller identity that could be exploited

### 5.3 Data Integrity
- [ ] Storage keys are unique and follow naming conventions
- [ ] Data structures are versioned for future upgrades
- [ ] No data loss on upgrade (storage layout is preserved)

---

## 6. Documentation

### 6.1 Code Documentation
- [ ] All public functions have `///` doc comments
- [ ] Doc comments explain inputs, outputs, and edge cases
- [ ] Complex logic has inline comments explaining the reasoning
- [ ] `README.md` is updated with any new functionality

### 6.2 Architecture Documentation
- [ ] New modules are documented in `ARCHITECTURE.md`
- [ ] Changes to the contract spec are reflected in `docs/contract-spec.md`
- [ ] Storage changes are documented in `docs/contract-storage.md`
- [ ] Event changes are documented in `docs/EVENTS.md`

### 6.3 Examples
- [ ] New functions have usage examples in doc comments
- [ ] Examples are tested (run `cargo test --doc`)
- [ ] No dead links in documentation

---

## 7. Code Quality

### 7.1 Rust Standards
- [ ] Code passes `cargo fmt`
- [ ] Code passes `cargo clippy` with no warnings
- [ ] No unused imports, variables, or functions
- [ ] No `TODO` or `FIXME` comments without a linked issue
- [ ] No commented-out code remains

### 7.2 Contract-Specific
- [ ] Contract builds with `cargo build --target wasm32-unknown-unknown`
- [ ] Gas usage is reasonable and within limits
- [ ] No unnecessary loops or expensive operations
- [ ] Event emissions are included for all state-changing operations

### 7.3 Security
- [ ] All functions have proper access control (`admin.require_auth()`)
- [ ] No sensitive data in logs
- [ ] Input validation is performed at the boundary
- [ ] Integer overflow/underflow is handled (use `checked_` operations)
- [ ] No reentrancy vulnerabilities

---

## 8. CI & Build

### 8.1 CI Checks
- [ ] All CI checks pass (formatting, lint, tests, build)
- [ ] No new warnings introduced
- [ ] Security scan passes (or vulnerabilities are justified)
- [ ] SBOM is generated and up-to-date

### 8.2 Build Process
- [ ] `make build` succeeds
- [ ] `make test` passes
- [ ] `make fmt` passes
- [ ] `make lint` passes

---

## 9. PR & Review

### 9.1 PR Description
- [ ] PR description follows the template
- [ ] Related issues are linked (`Closes #123`)
- [ ] Changes are clearly described
- [ ] Testing instructions are provided

### 9.2 Review Feedback
- [ ] All review comments are addressed
- [ ] No unresolved conversations remain
- [ ] Changes are squashed/rebased as appropriate

---

## 10. Cleanup Checklist Summary

Before opening a PR, verify:

- [ ] All new code has tests
- [ ] Documentation is updated
- [ ] No commented-out code
- [ ] No `unwrap()` or `expect()`
- [ ] No unused imports or variables
- [ ] `cargo fmt` and `cargo clippy` pass
- [ ] CI is green
- [ ] PR description is complete
- [ ] Related issues are linked

---

## Quick Commands

```bash
# Format code
cargo fmt

# Run linter
cargo clippy --all-targets -- -D warnings

# Build contract
cargo build --target wasm32-unknown-unknown

# Run all tests
cargo test

# Check for duplicate dependencies
cargo tree -d

# Check test coverage
cargo tarpaulin
Last updated: 2026-07-29
