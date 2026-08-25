# Vault Lifecycle Event Schema

This document outlines the event schema for the AxionVera Vault smart contract, defining topics, payload structures, and emission contexts for the `initialize`, `deposit`, `withdraw`, and `claim_rewards` flows.

## General Information

All events emitted by the vault contract use `Vault` (symbol: `"vault"`) as the primary topic.

## 1. Init Event

**When it is emitted:**
Emitted once during the successful execution of the `initialize` function, indicating the vault is now active and gated methods can be accessed.

**Topics:**
1. `symbol_short!("vault")`
2. `symbol_short!("init")`

**Payload Structure:**
* `admin` (`Address`): The address granted administrative privileges over the vault.

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

**Example:**
```rust
env.events().publish(
    (symbol_short!("vault"), symbol_short!("claim")),
    (user_address, 25_i128)
);
```

## Failed-Call Event Behavior

Events are strictly emitted only upon the successful completion of a state transition within the contract. If a transaction fails (e.g., due to a panic, `VaultError::NotInitialized`, `VaultError::InsufficientBalance`, or validation failure), the Soroban host rolls back any pending state changes and discards all events buffered during that contract call. Therefore, indexers and observers will never see events from failed or reverted invocations.

## SDK Indexing Expectations

When building an SDK or indexer to track the vault:
- **Filtering:** Use the primary topic `vault` to filter for events originating from the vault contract.
- **Handling Types:** The payloads contain `Address` and `i128` types. Deserializers must strictly adhere to the expected tuple structures (e.g., `(Address, i128)` for deposits).
- **Ordering:** Events represent canonical state changes only. Processing them sequentially accurately replays the vault's state evolution (e.g., maintaining balances or total deposit history).
- **Failures:** Do not attempt to index or track failed transactions via these events, as they will not be emitted to the ledger.
