# Vault Security Review Checklist Template

> **Note:** This checklist is for maintainer-only review before testnet deployment. Completing this checklist does not constitute a formal production security audit. No production security claims are made.

Use this template to verify the safety and stability of the Axionvera Vault Contract before deploying.

## 1. Initialization
- [ ] Contract initializes exactly once.
  - *Mapped tests:* `test_rejects_repeated_initialization`
- [ ] Admin address is securely stored and cannot be overwritten.
  - *Mapped tests:* `test_repeated_initialization_cannot_overwrite_admin`
- [ ] Uninitialized state rejects public queries and mutations.
  - *Mapped tests:* `test_total_deposits_query_rejected_before_initialization`, `test_user_balance_query_rejected_before_initialization`, `test_withdraw_rejected_before_initialization`

## 2. Authorization
- [ ] Withdrawal requires caller to be the authorized withdrawer.
  - *Mapped tests:* `test_withdraw_requires_caller_to_be_authorized_withdrawer`
- [ ] Protected endpoints appropriately enforce admin/owner signatures (or are explicitly noted as mock/testnet implementations).

## 3. Accounting
- [ ] Deposits correctly increase user balance and total deposits.
  - *Mapped tests:* `test_tracks_deposits_and_claims_rewards`, `lifecycle_multiple_deposits_and_withdrawals`
- [ ] Withdrawals correctly decrease user balance and total deposits.
  - *Mapped tests:* `lifecycle_multiple_deposits_and_withdrawals`
- [ ] Withdrawals cannot exceed available user balance.
- [ ] State representations naturally prevent overflow or underflow.

## 4. Reward Claims
- [ ] Valid users receive expected rewards upon claim.
  - *Mapped tests:* `test_valid_user_receives_expected_reward`
- [ ] Repeated claims do not duplicate rewards.
  - *Mapped tests:* `test_repeated_claim_does_not_duplicate_rewards`
- [ ] Claiming rewards resets the claimable amount correctly.
  - *Mapped tests:* `lifecycle_reward_claim_resets_claimable`
- [ ] Users without a balance or reward state cannot claim rewards.
  - *Mapped tests:* `test_user_without_balance_cannot_claim_reward`, `test_zero_user_balance_returns_zero_reward`

## 5. Events and Failed Calls
- [ ] State changes emit stable and correctly formatted events.
  - *Mapped tests:* `test_withdraw_emits_stable_withdraw_event`, `test_withdraw_event_matches_documented_fixture_shape`
- [ ] Failed calls do not emit misleading events (relies on Soroban's native transaction rollback).
- [ ] Failed calls do not mutate state unexpectedly (relies on Soroban's native transaction rollback).

## 6. Upgrade Assumptions
- [ ] Storage keys are stable and documented to prevent collisions.
- [ ] Data structures are designed considering future contract upgrades (if applicable).

---

**Reviewed By:** 
**Date:** 
**Target Network:** 
**Target Commit/WASM Hash:** 
