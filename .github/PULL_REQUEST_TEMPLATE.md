# Pull Request

## Linked Issue

Closes #

---

## Summary

Briefly describe what this PR changes.

-

---

## Type of Change

Select all that apply:

- [ ] Bug fix
- [ ] New feature
- [ ] Tests
- [ ] Documentation
- [ ] Refactor
- [ ] CI / tooling
- [ ] Contract logic
- [ ] Network-node logic
- [ ] SDK alignment

---

## Scope Confirmation

- [ ] This PR only addresses the linked issue.
- [ ] I did not include unrelated changes.
- [ ] I did not change public contract behavior unless required by the issue.
- [ ] I updated documentation if public behavior, events, methods, or setup steps changed.

---

## Implementation Notes

Explain the main implementation decisions.

-

---

## Testing

Describe the tests added or updated.

-

---

## Required Checks

Confirm all required checks pass locally:

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo check --workspace --all-targets`
- [ ] `cargo test --workspace --all-targets`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`

---

## Unit Test Requirement

For any new or changed function:

- [ ] Unit tests were added or updated.
- [ ] Happy path behavior is covered.
- [ ] Invalid input or failure behavior is covered.
- [ ] Important edge cases are covered.

If this PR does not add or change functions, explain why tests were not needed:

-

---

## Contract Safety Checklist

Complete this section for contract-related changes.

- [ ] Initialization behavior remains safe.
- [ ] Authorization behavior is tested where applicable.
- [ ] Failed calls do not corrupt state.
- [ ] Failed calls do not emit misleading events.
- [ ] Deposit and withdrawal accounting remains consistent.
- [ ] Reward accounting remains consistent.
- [ ] Event behavior is stable or documented.

Not applicable because:

-

---

## Documentation Checklist

- [ ] README updates were added where needed.
- [ ] `docs/` updates were added where needed.
- [ ] Public method changes are documented.
- [ ] Event schema changes are documented.
- [ ] SDK-facing behavior is documented where applicable.

Not applicable because:

-

---

## Screenshots / Logs

Paste relevant screenshots, terminal output, or test logs.

```text

```

---

## Contributor Confirmation

- [ ] I have read the issue requirements.
- [ ] I have followed the repository contribution rules.
- [ ] I understand that incomplete or out-of-scope PRs may be requested for changes or closed.
- [ ] I understand that `maybe-rewarded` issues are only reward-eligible after maintainer review and approval.

---

## Maintainer Review Notes

For maintainers only:

- [ ] Scope matches linked issue.
- [ ] Tests are sufficient.
- [ ] Checks pass.
- [ ] Documentation is sufficient.
- [ ] PR is eligible for merge.
- [ ] Reward eligibility reviewed, if applicable.
