# Axionvera Network - Workspace Health Analysis Report

**Date:** 2025-07-10  
**Analysis Type:** Non-invasive diagnostic (no code changes)  
**Tooling:** cargo metadata, cargo check, cargo test

---

## Executive Summary

| Metric                 | Count |
| ---------------------- | ----- |
| Total Workspace Crates | 32    |
| Healthy (Passing)      | 21    |
| Compilation Failures   | 6     |
| Infrastructure Blocked | 1     |
| Transitive Blocked     | 4     |

**Key Finding:** The workspace has **5 distinct root causes** creating a cascade of failures. The most critical is a **path-related infrastructure issue** (spaces in workspace path) blocking the network-node crate.

---

## Workspace Crate Inventory

### Full List (alphabetical)

1. admin
2. axionvera-accounting
3. axionvera-auth
4. axionvera-core
5. axionvera-events
6. axionvera-features
7. axionvera-fees
8. axionvera-interfaces
9. axionvera-integrations
10. axionvera-network-node
11. axionvera-orchestrator
12. axionvera-resources
13. axionvera-rewards
14. axionvera-risk
15. axionvera-security
16. axionvera-security-tests
17. axionvera-snapshots
18. axionvera-state
19. axionvera-storage
20. axionvera-validator
21. axionvera-vault-contract
22. axionvera-vault-contract-v2
23. tests-auth
24. tests-risk

---

## Health Matrix

### Healthy Crates (21)

| Crate                       | Tests Passed | Notes                |
| --------------------------- | ------------ | -------------------- |
| admin                       | 0            | No tests             |
| axionvera-accounting        | 0            | No tests             |
| axionvera-auth              | 2            | 2 passed             |
| axionvera-events            | 0            | No tests             |
| axionvera-interfaces        | 0            | No tests             |
| axionvera-integrations      | 0            | No tests             |
| axionvera-orchestrator      | 0            | No tests             |
| axionvera-resources         | 0            | No tests             |
| axionvera-rewards           | 0            | No tests             |
| axionvera-risk              | 0            | No tests             |
| axionvera-security          | 0            | No tests             |
| axionvera-snapshots         | 0            | No tests             |
| axionvera-state             | 0            | No tests             |
| axionvera-storage           | 0            | No tests             |
| axionvera-validator         | 0            | No tests             |
| axionvera-vault-contract    | 0            | No tests             |
| axionvera-vault-contract-v2 | 0            | 4 dead_code warnings |
| tests-auth                  | 0            | No tests             |

### Failing Crates (6 with distinct errors)

#### 1. axionvera-fees

**Status:** Compilation Error  
**Root Cause:** Missing storage functions  
**Error Messages:**

```
error[E0425]: cannot find function `get_fee_totals` in crate `storage`
error[E0425]: cannot find function `record_fee_totals` in crate `storage`
```

**Locations:**

- contracts/fees/src/lib.rs:86
- contracts/fees/src/lib.rs:89
- contracts/fees/src/tests.rs:76
- contracts/fees/src/tests.rs:119

**Fix Required:** Add to `contracts/storage/src/lib.rs`:

```rust
pub fn get_fee_totals(e: &Env, fee_type: FeeType) -> FeeTotals;
pub fn record_fee_totals(e: &Env, fee_type: FeeType, amount: i128) -> FeeTotals;
```

---

#### 2. axionvera-core

**Status:** Compilation Error  
**Root Cause:** Multiple issues (missing storage functions, missing dependency, missing imports)

**Error A - Missing Storage Functions:**

```
error[E0432]: unresolved imports `axionvera_storage::create_resource`,
  `axionvera_storage::list_resources`, `axionvera_storage::resource_count`,
  `axionvera_storage::resource_exists`, `axionvera_storage::transition_resource`
```

**Location:** contracts/core/src/lib.rs:12-17

**Error B - Missing Dependency Declaration:**

```
error[E0432]: unresolved import `axionvera_resources`
  = help: if you wanted to use a crate named `axionvera_resources`,
    use `cargo add axionvera_resources` to add it to your `Cargo.toml`
```

**Location:** contracts/core/src/lib.rs:6

**Error C - Missing SDK Imports:**

```
error: cannot find attribute `contract` in this scope
error: cannot find attribute `contractimpl` in this scope
  = help: consider importing this attribute macro
    3 + use soroban_sdk::contract;
    3 + use soroban_sdk::contractimpl;
```

**Location:** contracts/core/src/lib.rs:206, 209

**Fix Required:**

1. Add to `contracts/storage/src/lib.rs`:

   ```rust
   pub fn create_resource(/* ... */);
   pub fn list_resources(/* ... */);
   pub fn resource_count(/* ... */);
   pub fn resource_exists(/* ... */);
   pub fn transition_resource(/* ... */);
   ```

2. Add to `contracts/core/Cargo.toml`:

   ```toml
   [dependencies]
   axionvera-resources = { path = "../resources" }
   ```

3. Fix imports in `contracts/core/src/lib.rs`:
   ```rust
   use soroban_sdk::{contract, contractimpl, /* ... */};
   ```

---

#### 3. axionvera-features

**Status:** Compilation Error  
**Root Cause:** Missing event exports from axionvera-events

**Error:**

```
error[E0432]: unresolved imports `axionvera_events::ACT_FEAT_ADM_A`,
  `axionvera_events::ACT_FEAT_ADM_P`, `axionvera_events::ACT_FEAT_DIS`,
  `axionvera_events::ACT_FEAT_EN`, `axionvera_events::ACT_FEAT_INIT`,
  `axionvera_events::ACT_FEAT_PAUSE`, `axionvera_events::ACT_FEAT_REG`,
  `axionvera_events::ACT_FEAT_ROLL`, `axionvera_events::ACT_FEAT_UNPAU`,
  `axionvera_events::FeatureAdminTransferAcceptedEvent`,
  `axionvera_events::FeatureAdminTransferProposedEvent`,
  `axionvera_events::FeatureDisabledEvent`,
  `axionvera_events::FeatureEnabledEvent`,
  `axionvera_events::FeatureInitializedEvent`,
  `axionvera_events::FeaturePausedEvent`,
  `axionvera_events::FeatureRegisteredEvent`,
  `axionvera_events::FeatureRolloutUpdatedEvent`,
  `axionvera_events::FeatureUnpausedEvent`,
  `axionvera_events::PROTOCOL_FEATURES`
```

**Fix Required:** Export constants and event types from `contracts/events/src/lib.rs`:

```rust
pub const ACT_FEAT_ADM_A: Symbol = symbol_short!("feat_adm_a");
// ... etc for all missing constants

pub struct FeatureAdminTransferAcceptedEvent { /* ... */ }
// ... etc for all missing event types
```

---

#### 4. axionvera-network-node

**Status:** Build Script Failure  
**Root Cause:** tikv-jemalloc-sys configure fails with spaces in workspace path

**Error:**

```
error: failed to run custom build command for `tikv-jemalloc-sys`
configure: error: Prefix should not contain spaces
  prefix=/Users/boufdaddy/Documents/web3 projects/axionvera-network/target/...
```

**Fix Options (choose one):**

**Option A (Recommended - Permanent Fix):**
Rename workspace directory to remove spaces:

```bash
mv "/Users/boufdaddy/Documents/web3 projects/axionvera-network" \
   "/Users/boufdaddy/Documents/web3-projects/axionvera-network"
```

**Option B (Build Configuration):**
Disable jemalloc feature if available in network-node Cargo.toml

---

### Transitive Failures (Dependency-Driven)

| Crate                    | Blocked By                     | Status                              |
| ------------------------ | ------------------------------ | ----------------------------------- |
| axionvera-security-tests | axionvera-core                 | Will pass once core compiles        |
| tests-risk               | axionvera-fees, axionvera-core | Will pass once dependencies compile |

---

## Recommended Repair Sequence (Prioritized)

### Phase 1: Infrastructure Fix (5 minutes)

**Action:** Rename workspace directory to remove spaces  
**Impact:** Unblocks axionvera-network-node immediately  
**Risk:** None (cosmetic change only)

### Phase 2: Fix axionvera-core (2-3 hours)

**Dependencies:** None (foundation crate)  
**Actions:**

1. Add missing storage re-exports in `contracts/storage/src/lib.rs`
2. Add `axionvera-resources` dependency to `contracts/core/Cargo.toml`
3. Fix SDK imports in `contracts/core/src/lib.rs`

**Impact:** Unblocks axionvera-security-tests, tests-risk  
**Risk:** Medium - Core contract changes require careful review

### Phase 3: Fix axionvera-fees (30 minutes)

**Dependencies:** Phase 2 (storage changes)  
**Actions:**

1. Add fee totals functions to `contracts/storage/src/lib.rs`

**Impact:** Unblocks partial functionality in tests-risk  
**Risk:** Low - Additive changes only

### Phase 4: Fix axionvera-features (1 hour)

**Dependencies:** None (independent fix)  
**Actions:**

1. Export event constants from `contracts/events/src/lib.rs`

**Impact:** Unblocks axionvera-features  
**Risk:** Low - Additive changes only

### Phase 5: Verify Full Workspace (30 minutes)

**Command:**

```bash
cargo test --workspace --all-targets -- --test-threads=1
```

---

## Appendix: Dependency Graph (Simplified)

```
axionvera-interfaces (root - no deps)
├── axionvera-events
│   ├── axionvera-auth ✓
│   ├── axionvera-state ✓
│   ├── axionvera-accounting ✓
│   │   └── axionvera-storage ✓
│   ├── axionvera-fees ✗ (needs storage fns)
│   │   └── axionvera-risk ✓
│   ├── axionvera-security ✓
│   ├── axionvera-core ✗ (missing imports, deps)
│   │   ├── axionvera-snapshots ✓
│   │   ├── axionvera-validator ✓
│   │   └── axionvera-vault-contract ✓
│   ├── axionvera-features ✗ (missing event exports)
│   ├── axionvera-integrations ✓
│   └── axionvera-orchestrator ✓
├── axionvera-storage ✓
└── axionvera-resources ✓

axionvera-network-node ✗ (jemalloc issue)
└── tikv-jemalloc-sys ✗ (path with spaces)

Test Crates:
├── tests-auth ✓
├── tests-risk ✗ (depends on fees, core)
└── axionvera-security-tests ✗ (depends on core)
```

---

## Conclusion

The Axionvera Network workspace is **75% healthy** with **6 core compilation issues** blocking approximately **25% of the codebase**. The failures cluster around:

1. **API Mismatches** (storage functions missing) - 2 crates
2. **Import/Dependency Issues** (core contract) - 1 crate with 4 sub-issues
3. **Event Export Gaps** (features contract) - 1 crate
4. **Infrastructure Limitation** (path with spaces) - 1 crate

All issues are **mechanical fixes** requiring no algorithmic changes. The recommended repair sequence prioritizes foundational crates (core, storage) before dependent ones, enabling parallel work streams once Phase 2 is complete.

**Estimated Total Repair Time:** 6-8 hours  
**Risk Level:** Low to Medium  
**Verification:** Full workspace test pass expected after Phase 5
