# Vault interface schema

The repository publishes a versioned, machine-readable description of the
public Soroban interface implemented by `VaultContract`.

| Artifact | Purpose |
| --- | --- |
| `schemas/vault-interface.schema.json` | JSON Schema (draft 2020-12) that validates and pins the v0.1 interface document shape. |
| `schemas/vault-interface-v0.1.json` | Complete v0.1 interface document for SDK compatibility tests and binding metadata. |
| `examples/vault-interface/consume-schema.js` | Dependency-free example that reads the document and derives ordered SDK invocation metadata. |
| `contracts/vault-contract/src/lib.rs` | Contract implementation and tests that guard the document against interface drift. |

The current `schema_version` is `1`, and the contract `interface_version` is
`0.1`. The schema format version and contract interface version are separate:
a non-breaking improvement to the document format need not imply a change to
an on-chain method.

## What is described

The v0.1 document includes every function exported by the current
`#[contractimpl]`, including functions marked `internal` that are still
publicly invocable on-chain. For each method it records:

- the exact contract symbol;
- ordered arguments, zero-based positions, and Soroban types;
- success and contract-error return types;
- initialization requirements;
- current `Address.require_auth` behavior;
- read/write classification and stability;
- emitted event references; and
- implementation notes and limitations.

The document also records `VaultError` discriminants and all four current event
shapes. Event topics are ordered typed values, not a single joined string. The
first topic is always the `Symbol` `vault`; the second is one of `init`,
`deposit`, `withdraw`, or `claim`.

## SDK consumption

SDK tests may load `schemas/vault-interface-v0.1.json` directly or copy it
into an SDK fixture. Consumers should:

1. Reject an unsupported `schema_version` before reading fields.
2. Match `interface_version` against the deployed contract version expected by
   the SDK.
3. Look up a method by its exact `name`; do not derive aliases such as
   `get_balance`.
4. Sort `arguments` by `position` and encode them as the declared Soroban
   types. The `Env` Rust parameter is host-provided and is intentionally absent.
5. Decode `returns.kind = "result"` using `ok` and `error`. A
   `returns.kind = "value"` method has no `VaultError` result wrapper.
6. Use `authorization.address_argument` to identify the address that must sign
   when `authorization.required` is true.
7. Filter events using the ordered `topics` values and decode `data.fields` in
   position order.
8. Treat `stability = "internal"` as discoverable contract ABI, not as a
   recommendation to expose the method in a wallet SDK.

`i128` values must be encoded as Soroban `i128` values. A JavaScript SDK should
use `bigint`, or a base-10 integer string converted by its Soroban encoder, and
must not pass floating-point numbers.

### Example lookup

```js
const fs = require("node:fs");

const vault = JSON.parse(
  fs.readFileSync("schemas/vault-interface-v0.1.json", "utf8"),
);

if (vault.schema_version !== "1" || vault.interface_version !== "0.1") {
  throw new Error("Unsupported vault interface");
}

const deposit = vault.methods.find((method) => method.name === "deposit");
const orderedArguments = [...deposit.arguments]
  .sort((left, right) => left.position - right.position)
  .map(({ name, type }) => ({ name, type }));

// orderedArguments is:
// [{ name: "from", type: "Address" }, { name: "amount", type: "i128" }]
```

Run the committed example from the repository root with:

```bash
node examples/vault-interface/consume-schema.js deposit
```

## Authorization semantics

`authorization.required` reports only authorization that is implemented today.
It is not a desired future policy.

- `initialize` requires authorization from `admin`.
- `deposit` requires authorization from `from`.
- `withdraw` requires authorization from `to`.
- `claim_rewards` requires authorization from `user`.
- Queries require no authorization.
- `set_claimable_reward` and `set_reward_balance` currently require no
  authorization. They are marked `internal` for SDK visibility, but they remain
  publicly invocable contract methods in v0.1.

Missing authorization rejects at the Soroban host level rather than returning
a `VaultError` variant.

## Return and error semantics

All public methods except `is_initialized` return a Rust
`Result<SuccessType, VaultError>`. The interface document represents that as:

```json
{
  "kind": "result",
  "ok": "i128",
  "error": "VaultError",
  "description": "..."
}
```

`is_initialized` directly returns a `bool` and uses `kind: "value"`.
`InvalidRewardState` remains in the error table because it is part of the
current `VaultError` type, but its empty `returned_by` list explicitly records
that no current public method returns it.

## Event behavior

The methods `initialize`, `deposit`, and `withdraw` emit their listed event
after a successful state update. `claim_rewards` emits `("vault", "claim")`
when the stored claimable amount is nonzero. Because the internal setter does
not validate signs, this describes the exact current `claimable != 0` check. A
successful zero-value claim returns `0` without emitting. Failed invocations emit no vault
event.

The `sdk_event_type` field preserves the current SDK naming bridge:
`init` maps to `initialized`, and `claim` maps to `claim_rewards`. Indexers
should filter on the on-chain `topics`, not `sdk_event_type`.

## Explicit non-features

The interface deliberately does not claim behavior that the contract does not
implement:

- `deposit` and `withdraw` update accounting but do not transfer the configured
  deposit token;
- `claim_rewards` clears accounting but does not transfer the configured reward
  token;
- `pending_rewards` is not the stored amount paid by `claim_rewards`;
- no share token, exchange rate, lock period, or aggregate `get_info` method is
  present; and
- `get_balance` and `get_pending_rewards` aliases are not present.

These limitations are also machine-readable in the top-level `limitations`
array.

## Updating the interface

A contract PR that changes a public method, argument order, type, return,
authorization check, error use, or event must update the interface artifacts in
the same PR.

For a compatible documentation correction, update the v0.1 document and its
tests. For an interface-breaking contract change:

1. increment `interface_version`;
2. add a new versioned interface document rather than silently changing an old
   compatibility fixture;
3. update the JSON Schema constraints or add a version-specific schema;
4. update SDK compatibility tests and the interface changelog; and
5. retain only behavior that is actually implemented by the contract.

Validate changes with the repository quality commands:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```
