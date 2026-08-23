# SDK v2 to Network vault contract interface

This is the bridge document between [Axionvera SDK v2](https://github.com/Axionvera/axionvera-sdk)
and this Network vault. SDK and Network contributors should treat the contract
symbols, argument order, types, return values, and event topics below as the
invoke surface. Future issues should align to this mapping instead of
re-deriving it from either repo in isolation.

| Side | Source of truth |
| --- | --- |
| Network vault | `contracts/vault-contract/src/lib.rs` |
| SDK v2 wrapper | `packages/core/src/contracts/vault.ts` in [axionvera-sdk](https://github.com/Axionvera/axionvera-sdk) |
| SDK v2 event types | `packages/core/src/events.ts` in axionvera-sdk |

This document describes only what the current vault implements. It does not
treat share tokens, exchange rates, lock periods, or token transfers as part of
this contract — those ideas exist in older SDK helpers but are not present here.

---

## Conventions

| Item | Contract rule | SDK v2 expectation |
| --- | --- | --- |
| Host env | Every function receives `env: Env` first. | SDK invocations omit it. |
| Amounts | `i128` integer in the token's smallest units. Must be `> 0` on `deposit` / `withdraw`. Overflowing checked arithmetic returns `VaultError::InvalidAmount`. | `AmountInput` (`bigint \| number \| string`) is passed through `normalizeAmount()`, which converts to `bigint` and rejects `<= 0`. Write helpers then send `normalizeAmount(amount).toString()`. The invoker must encode that decimal string as Soroban `i128`, not as a float or scaled decimal. |
| Addresses | Soroban `Address`. | Stellar account / contract IDs as `string`. |
| Auth | Write methods below call `require_auth` on the acting address, except `set_claimable_reward` and `set_reward_balance`. Queries do not. | The connected wallet must sign for `from` / `to` / `user`. |
| Initialization | All writes and all queries except `is_initialized` return `VaultError::NotInitialized` until `initialize` succeeds. | Reads/writes against an uninitialized contract must surface that error. |
| Results | Public methods return `Result<T, VaultError>` unless noted (`is_initialized` returns `bool`). | SDK write helpers currently return `VaultTransaction` (`hash` / `status`) and do **not** decode the contract `i128` return. Read helpers currently return SDK view types (`VaultInfo`, `VaultBalance`, `VaultReward`) rather than raw `i128`. |

Deposit, withdraw, and claim currently update vault accounting only. They do
not transfer `deposit_token` or `reward_token`.

### `VaultError`

| Variant | Code | When it is returned |
| --- | --- | --- |
| `AlreadyInitialized` | 1 | `initialize` is called after the vault is already initialized |
| `NotInitialized` | 2 | A gated method is called before `initialize` |
| `InvalidAmount` | 3 | Amount is `<= 0`, or checked arithmetic overflows |
| `InsufficientBalance` | 4 | Withdraw amount is greater than the user's stored balance |
| `InvalidRewardState` | 5 | Defined on the contract; not returned by the functions in this document |

---

## SDK read methods

SDK v2 reads go through `invoker.read` when present, otherwise `invoker.invoke`.
They must use the **contract** symbol in the table, not the SDK helper name.

| SDK v2 helper | Current SDK symbol | Contract method | Contract args (after `env`) | Contract return | Status |
| --- | --- | --- | --- | --- | --- |
| `getInfo()` | `get_info` | _none_ — compose the config + totals queries below | see [Configuration and totals](#configuration-and-totals) | see below | **gap** |
| `getBalance(address)` | `get_balance` | `user_balance` | `user: Address` | `i128` stored deposit balance (`0` if the account has never deposited) | **symbol gap** |
| `getPendingRewards(address)` | `get_pending_rewards` | `pending_rewards` | `user: Address` | `i128` proportional pending share | **symbol gap** |

### `getInfo()` → configuration and totals

There is no `get_info` method on this contract. `VaultContract.getInfo()` should
not invoke `get_info`. It should simulate the following independent reads and
fold them into `VaultInfo`:

| Contract function | Inputs | Output | Suggested `VaultInfo` field |
| --- | --- | --- | --- |
| `is_initialized` | none | `bool` | not currently modeled; call first and fail closed if `false` |
| `admin` | none | `Result<Address, VaultError>` | not currently modeled |
| `deposit_token` | none | `Result<Address, VaultError>` | not a Stellar `assetCode` / `assetIssuer` pair |
| `reward_token` | none | `Result<Address, VaultError>` | not currently modeled |
| `total_deposits` | none | `Result<i128, VaultError>` | `totalDeposits` (`bigint`) |

`is_initialized` is safe to call before setup. The other queries return
`NotInitialized` until `initialize` succeeds.

`VaultInfo.contractId` is an SDK-side field (the wrapper's `contractId`), not a
contract return. `assetCode`, `assetIssuer`, and `rewardPool` have **no**
matching public methods. Vault-wide reward inventory is `RewardBalance` and is
only writable via `set_reward_balance`; it is not directly readable.

### `getBalance(address)` → `user_balance`

**Inputs**

| Argument | Type | Notes |
| --- | --- | --- |
| `user` | `Address` | Account whose deposited balance is returned |

**Output**

`Result<i128, VaultError>` — that user's stored deposit balance, or `0` if they
have never deposited.

**SDK mapping**

- Invoke contract symbol `user_balance` with one argument: `address` (`Address`).
- Argument order: `[user]`.
- Decode as `i128`, then wrap as `VaultBalance { address, amount }` if the SDK
  keeps that view type (`amount` is `bigint`).
- Current SDK helper queries `get_balance`. Align the symbol to `user_balance`.
- `getVaultShares` / `shares_of` are not backed by this contract. User deposit
  state is this balance query only.

### `getPendingRewards(address)` → `pending_rewards`

**Inputs**

| Argument | Type | Notes |
| --- | --- | --- |
| `user` | `Address` | Account whose pending share is returned |

**Output**

`Result<i128, VaultError>` — `(user_balance * RewardBalance) / total_deposits`,
using `contracts/rewards` `calculate_pending_rewards`. Returns `0` when total
deposits, user balance, or the reward pool is not positive, or when checked
arithmetic overflows.

This value is **not** the amount `claim_rewards` will pay. Claim payouts come
from per-user `ClaimableReward` storage, which has no dedicated public getter.
See [Write methods](#sdk-write-methods).

**SDK mapping**

- Invoke contract symbol `pending_rewards` with one argument: `address`
  (`Address`).
- Argument order: `[user]`.
- Decode as `i128`, then wrap as `VaultReward { address, amount }` if the SDK
  keeps that view type.
- Current SDK helper queries `get_pending_rewards`. Align the symbol to
  `pending_rewards`.
- Do not call `pending_rewards()` with an empty argument list. This Network
  function requires `user`.

---

## SDK write methods

| SDK v2 helper | Current SDK symbol | Contract method | Contract args (after `env`) | Contract return | Status |
| --- | --- | --- | --- | --- | --- |
| _none_ | _none_ | `initialize` | `admin`, `deposit_token`, `reward_token` | `()` | **missing helper** |
| `deposit(from, amount)` | `deposit` | `deposit` | `from`, `amount` | `i128` **new** user balance | argument order aligned; amount encoding + return type differ |
| `withdraw(to, amount)` | `withdraw` | `withdraw` | `to`, `amount` | `i128` **new** user balance | argument order aligned; amount encoding + return type differ |
| `claimRewards(address)` | `claim_rewards` | `claim_rewards` | `user` | `i128` claimed amount (`0` if nothing to claim) | symbol and args aligned; return type differs |

### `initialize`

Sets admin, deposit token, reward token, and zero starting totals. May be
called once.

**Inputs**

| Argument | Type | Notes |
| --- | --- | --- |
| `admin` | `Address` | Must authorize the call (`admin.require_auth()`) |
| `deposit_token` | `Address` | Stored for later configuration queries |
| `reward_token` | `Address` | Stored for later configuration queries |

**Output**

`Result<(), VaultError>` — `Ok(())` on success.

On success the contract also stores `TotalDeposits = 0` and `RewardBalance = 0`,
and emits the `init` event described below.

**SDK mapping**

`new VaultContract({ contractId, invoker })` only binds the SDK to a deployed
contract ID. It does not call this function. SDK contributors should invoke
contract symbol `initialize` with arguments in this order: `admin`,
`deposit_token`, `reward_token`.

### `deposit(from, amount)` → `deposit`

Credits `amount` to `from` and increases `total_deposits`.

**Inputs**

| Argument | Type | Notes |
| --- | --- | --- |
| `from` | `Address` | Depositor. Must authorize (`from.require_auth()`) |
| `amount` | `i128` | Must be `> 0` |

**Output**

`Result<i128, VaultError>` — the user's **new** stored balance, not the
deposited amount.

**SDK mapping**

Maps to `vault.deposit(from, amount)`.

- Encode arguments in **contract order**: `from` (`Address`), then `amount`
  (`i128`). Current SDK v2 already sends `[from, amountString]`.
- Format `amount` as a base-10 integer string of smallest units, then encode as
  Soroban `i128`. Do not send a JavaScript `number` through the invoker.
- Decode the contract return as `i128` (updated user balance). Current SDK v2
  returns `VaultTransaction` and drops this value.
- Do not assume share minting. This contract has no share token and no
  `preview_deposit` method.

### `withdraw(to, amount)` → `withdraw`

Debits `amount` from `to` and decreases `total_deposits`. There is no lock
period or withdrawal delay on this contract.

**Inputs**

| Argument | Type | Notes |
| --- | --- | --- |
| `to` | `Address` | Account whose balance is reduced. Must authorize (`to.require_auth()`) |
| `amount` | `i128` | Must be `> 0` and `<=` that account's stored balance |

**Output**

`Result<i128, VaultError>` — the user's **new** stored balance, not the
withdrawn amount.

**SDK mapping**

Maps to `vault.withdraw(to, amount)`.

- Encode arguments in **contract order**: `to` (`Address`), then `amount`
  (`i128`). Current SDK v2 already sends `[to, amountString]`.
- Amount formatting is the same as `deposit`.
- Decode the contract return as `i128` (updated user balance). Current SDK v2
  returns `VaultTransaction` and drops this value.
- Do not assume share redemption or `preview_withdraw`. Those methods are not
  on this contract.

### `claimRewards(address)` → `claim_rewards`

Reads the caller's stored claimable amount, clears it to `0`, and returns the
amount that was claimed. Claiming with no stored rewards returns `0` and does
not error.

**Inputs**

| Argument | Type | Notes |
| --- | --- | --- |
| `user` | `Address` | Claimant. Must authorize (`user.require_auth()`) |

**Output**

`Result<i128, VaultError>` — claimed amount (`0` when nothing is claimable).

Per-user claimable state is `ClaimableReward(Address)`. That storage is
separate from vault-wide `RewardBalance` and from the `pending_rewards` query.
This contract does not transfer `reward_token` on claim.

**SDK mapping**

Maps to `vault.claimRewards(address)`.

- Invoke contract symbol `claim_rewards` with one argument: `user` (`Address`).
  Current SDK v2 already sends `[address]`.
- Decode the return value as `i128` (claimed amount). A `0` result is success,
  not an error. Current SDK v2 returns `VaultTransaction` and drops this value.
- Do not preflight this call against `pending_rewards(user)`; that query is not
  the claimable balance.

---

## Argument order and amount formatting

Contract argument order is always the Rust parameter order after `env`.

| Action | SDK v2 call | Bytes-to-host args | Required contract types |
| --- | --- | --- | --- |
| Read info | `getInfo()` | _compose several no-arg reads_ | see above |
| Read balance | `getBalance(address)` | `[address]` | `Address` |
| Read pending rewards | `getPendingRewards(address)` | `[address]` | `Address` |
| Deposit | `deposit(from, amount)` | `[from, amount]` | `Address`, `i128` |
| Withdraw | `withdraw(to, amount)` | `[to, amount]` | `Address`, `i128` |
| Claim | `claimRewards(address)` | `[address]` | `Address` |
| Initialize | _none yet_ | `[admin, deposit_token, reward_token]` | `Address`, `Address`, `Address` |

Amount rules shared by `deposit` and `withdraw`:

1. Value is an integer count of smallest token units (`i128`), not a decimal
   display amount.
2. SDK v2 accepts `bigint | number | string`, runs `normalizeAmount()`, and
   rejects `<= 0` before invoke with `ValidationError`.
3. The string sent on the wire must be the base-10 representation of that
   positive integer (`"100"`, not `"100.0"` or scientific notation).
4. The contract also rejects `amount <= 0` as `VaultError::InvalidAmount`.
5. Do not swap amount and address. Older SDK snippets used `{ amount, from }`
   object bags; SDK v2 uses positional `(from, amount)` / `(to, amount)`, which
   already matches this contract.

---

## Event expectations

Successful `initialize`, `deposit`, `withdraw`, and `claim_rewards` publish a
Soroban contract event with **two topics** and a typed data payload. Failed
calls do not emit. `claim_rewards` also emits nothing when the stored
claimable amount is `0` (the call still succeeds and returns `0`).

Topics are `symbol_short` values: `("vault", <action>)`.

| Flow | Topics | Data | When emitted |
| --- | --- | --- | --- |
| Initialize | `("vault", "init")` | `admin: Address` | After storage is written |
| Deposit | `("vault", "deposit")` | `(from: Address, amount: i128)` | After the user's balance and `total_deposits` are updated. `amount` is the deposited amount, not the new balance. |
| Withdraw | `("vault", "withdraw")` | `(to: Address, amount: i128)` | After the user's balance and `total_deposits` are updated. `amount` is the withdrawn amount, not the new balance. |
| Claim | `("vault", "claim")` | `(user: Address, claimed: i128)` | Only when `claimed > 0`, after `ClaimableReward` is cleared to `0`. |

`set_claimable_reward` and `set_reward_balance` do not emit events.

### SDK event type gaps

SDK v2 `VaultEventType` is `"deposit" | "withdraw" | "claim_rewards" | "initialized"`.
That union does not match the on-chain second topic:

| On-chain second topic | SDK `VaultEventType` |
| --- | --- |
| `init` | `initialized` |
| `deposit` | `deposit` |
| `withdraw` | `withdraw` |
| `claim` | `claim_rewards` |

Indexers and the SDK event helper should match on topics `("vault", "deposit")`,
`("vault", "withdraw")`, and `("vault", "claim")` rather than on the SDK string
alone. The first topic is always `vault`.

---

## Network methods with no SDK v2 helper

These are on the contract today and are not wrapped by `VaultContract`:

| Contract function | Args (after `env`) | Return | Notes |
| --- | --- | --- | --- |
| `initialize` | `admin`, `deposit_token`, `reward_token` | `()` | Deployment-time write |
| `is_initialized` | _(none)_ | `bool` | Ungated |
| `admin` | _(none)_ | `Address` | |
| `deposit_token` | _(none)_ | `Address` | |
| `reward_token` | _(none)_ | `Address` | |
| `total_deposits` | _(none)_ | `i128` | Needed by `getInfo()` |
| `set_claimable_reward` | `user`, `amount` | `()` | Writes `ClaimableReward`. **No `require_auth`**. Not a wallet-facing SDK method. |
| `set_reward_balance` | `amount` | `()` | Writes vault-wide `RewardBalance`. **No `require_auth`**. Not a wallet-facing SDK method. |

---

## Current gaps between SDK v2 and this contract

1. **Read symbols are wrong.** `getInfo` → `get_info` (missing), `getBalance` →
   `get_balance` (should be `user_balance`), `getPendingRewards` →
   `get_pending_rewards` (should be `pending_rewards`).
2. **`getInfo` cannot be a single simulate.** There is no `get_info` and no
   `totalAssets` / `totalSupply` / `apy` / `lockPeriod`. Compose
   `is_initialized`, `admin`, `deposit_token`, `reward_token`, and
   `total_deposits`.
3. **Read response shapes differ.** The contract returns `i128` or `Address`.
   SDK v2 types (`VaultBalance`, `VaultReward`, `VaultInfo`) are wrappers the
   SDK must build after decoding.
4. **Write returns are dropped.** `deposit` / `withdraw` / `claim_rewards`
   return `i128`; SDK v2 returns `VaultTransaction` only.
5. **Amount encoding is stringly typed.** SDK v2 sends `normalizeAmount(...).toString()`.
   The invoker must encode that as `i128`. The contract does not accept a
   decimal string.
6. **Pending rewards ≠ claimable rewards.** `pending_rewards(user)` is a
   proportional view over `RewardBalance`. `claim_rewards(user)` pays and
   clears `ClaimableReward(user)`, which is only set by `set_claimable_reward`.
7. **Event names differ** for init (`init` vs `initialized`) and claim
   (`claim` vs `claim_rewards`). Deposit and withdraw topic names already match.
8. **No initialize helper** on `VaultContract`.
9. **No token transfers, shares, exchange rate, or lock period** on this
   contract. SDK helpers or types that assume those must not be mapped here.
10. **`set_claimable_reward` / `set_reward_balance` are unprotected** (no
    `require_auth`). They are not part of the wallet SDK surface.

Functions that appear in older SDK wrappers but are **not** part of this
Network interface: `preview_deposit`, `preview_withdraw`, `get_shares`,
`get_exchange_rate`, `shares_of`, `exchange_rate`, `balance`, `get_balance`,
`get_pending_rewards`, `get_info`, `totalAssets`, `totalSupply`, `apy`,
`lockPeriod`.
