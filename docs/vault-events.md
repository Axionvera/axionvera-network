# Vault Lifecycle Event Schema

This document outlines the event schema for the AxionVera Vault smart contract, defining topics, payload structures, emission contexts, and compatibility fixtures for the `initialize`, `deposit`, `withdraw`, and `claim_rewards` flows.

## General Information

All events emitted by the vault contract use `Vault` (symbol: `"vault"`) as the primary topic.

The compatibility fixture schema is maintained at
`schemas/vault-event.schema.json`. Example payloads live in
`examples/vault-events/` and are intended for SDK and dashboard indexing tests,
not as a live indexer implementation. `examples/vault-events/catalog.json`
provides a machine-readable list of the fixture files.

| Flow | Fixture | On-chain topics | SDK event type |
| --- | --- | --- | --- |
| Initialize | `examples/vault-events/initialize.json` | `["vault", "init"]` | `initialized` |
| Deposit | `examples/vault-events/deposit.json` | `["vault", "deposit"]` | `deposit` |
| Withdraw | `examples/vault-events/withdraw.json` | `["vault", "withdraw"]` | `withdraw` |
| Claim | `examples/vault-events/claim.json` | `["vault", "claim"]` | `claim_rewards` |

## 1. Init Event

**When it is emitted:**
Emitted once during the successful execution of the `initialize` function, indicating the vault is now active and gated methods can be accessed.

**Topics:**
1. `symbol_short!("vault")`
2. `symbol_short!("init")`

**Payload Structure:**
* `admin` (`Address`): The address granted administrative privileges over the vault.

**Compatibility fixture data shape:**
```json
{
  "kind": "address",
  "field": "admin",
  "value": "ADDRESS_PLACEHOLDER"
}
```

**Example:**
```rust
env.events().publish(
    (symbol_short!("vault"), symbol_short!("init")),
    admin_address
);
```

## 2. Deposit Event

**When it is emitted:**
Emitted when a user successfully deposits tokens into the vault via the `deposit` function.

**Topics:**
1. `symbol_short!("vault")`
2. `symbol_short!("deposit")`

**Payload Structure:**
A tuple containing:
* `from` (`Address`): The address making the deposit.
* `amount` (`i128`): The number of tokens deposited.

**Compatibility fixture data shape:**
```json
{
  "kind": "tuple",
  "fields": [
    { "name": "from", "type": "Address", "value": "ADDRESS_PLACEHOLDER" },
    { "name": "amount", "type": "i128", "value": "100" }
  ]
}
```

**Example:**
```rust
env.events().publish(
    (symbol_short!("vault"), symbol_short!("deposit")),
    (user_address, 100_i128)
);
```

## 3. Withdraw Event

**When it is emitted:**
Emitted when a user successfully withdraws tokens from the vault via the `withdraw` function.

**Topics:**
1. `symbol_short!("vault")`
2. `symbol_short!("withdraw")`

**Payload Structure:**
A tuple containing:
* `to` (`Address`): The address receiving the withdrawn tokens.
* `amount` (`i128`): The number of tokens withdrawn.

**Compatibility fixture data shape:**
```json
{
  "kind": "tuple",
  "fields": [
    { "name": "to", "type": "Address", "value": "ADDRESS_PLACEHOLDER" },
    { "name": "amount", "type": "i128", "value": "25" }
  ]
}
```

**Example:**
```rust
env.events().publish(
    (symbol_short!("vault"), symbol_short!("withdraw")),
    (user_address, 50_i128)
);
```

## 4. Claim Event

**When it is emitted:**
Emitted when a user successfully claims their accumulated rewards via the `claim_rewards` function, provided the claimable amount is greater than 0. 

**Topics:**
1. `symbol_short!("vault")`
2. `symbol_short!("claim")`

**Payload Structure:**
A tuple containing:
* `user` (`Address`): The address claiming the rewards.
* `claimable` (`i128`): The number of reward tokens claimed.

**Compatibility fixture data shape:**
```json
{
  "kind": "tuple",
  "fields": [
    { "name": "user", "type": "Address", "value": "ADDRESS_PLACEHOLDER" },
    { "name": "claimable", "type": "i128", "value": "50" }
  ]
}
```

**Example:**
```rust
env.events().publish(
    (symbol_short!("vault"), symbol_short!("claim")),
    (user_address, 25_i128)
);
```

## Failed-Call Event Behavior

Events are strictly emitted only upon the successful completion of a state transition within the contract. If a transaction fails (e.g., due to a panic, `VaultError::NotInitialized`, `VaultError::InsufficientBalance`, or validation failure), the Soroban host rolls back any pending state changes and discards all events buffered during that contract call. Therefore, indexers and observers will never see events from failed or reverted invocations.

The fixture field `indexing.failed_calls_emit` is always `false` for this
interface version. Consumers should treat failed-call details as transaction
simulation or RPC status data, not as vault event data.

## SDK Indexing Expectations

When building an SDK or indexer to track the vault:
- **Filtering:** Use the primary topic `vault` to filter for events originating from the vault contract.
- **Handling Types:** The payloads contain `Address` and `i128` types. Deserializers must strictly adhere to the expected tuple structures (e.g., `(Address, i128)` for deposits).
- **Ordering:** Events represent canonical state changes only. Processing them sequentially accurately replays the vault's state evolution (e.g., maintaining balances or total deposit history).
- **Failures:** Do not attempt to index or track failed transactions via these events, as they will not be emitted to the ledger.
- **Fixtures:** Treat `examples/vault-events/*.json` as mocked compatibility
  examples. Replace `ADDRESS_PLACEHOLDER` with real account or contract
  addresses when replaying testnet examples.
- **Testnet readiness:** The fixture schema uses Soroban topic names and typed
  payload field order exactly as emitted by the contract tests, so it can be
  reused by SDK and dashboard parsers before live indexing is built.
