# SDK to Network vault contract interface

This document is the Network-side contract of record for the Soroban vault in
`contracts/vault-contract`. SDK contributors should treat the function names,
argument order, types, and return values below as the expected invoke surface.

It describes only what the current vault implements. It does not document
share tokens, exchange rates, lock periods, token transfers, or other helpers
that exist in SDK code but are not present on this contract.

Source of truth: `contracts/vault-contract/src/lib.rs`.

The matching SDK wrapper is `VaultContract` in
[axionvera-sdk](https://github.com/Axionvera/axionvera-sdk)
(`packages/core/src/contracts/VaultContract.ts`). Mapping notes name the helper
the SDK should use and call out current mismatches so the two repos do not
drift.

## Conventions

| Item | Contract rule |
| --- | --- |
| Host env | Every function receives `env: Env` first. SDK invocations omit it. |
| Amounts | `i128`. SDK helpers should send `bigint` encoded as Soroban `i128`. |
| Addresses | Soroban `Address`. SDK helpers should send Stellar account / contract IDs. |
| Auth | Write methods listed below call `require_auth` on the acting address. Queries do not. |
| Initialization | All write methods and all queries except `is_initialized` return `VaultError::NotInitialized` until `initialize` succeeds. |
| Results | Public methods return `Result<T, VaultError>` unless noted. |

### `VaultError`

| Variant | Code | When it is returned |
| --- | --- | --- |
| `AlreadyInitialized` | 1 | `initialize` is called after the vault is already initialized |
| `NotInitialized` | 2 | A gated method is called before `initialize` |
| `InvalidAmount` | 3 | Amount is `<= 0`, or checked arithmetic overflows |
| `InsufficientBalance` | 4 | Withdraw amount is greater than the user's stored balance |
| `InvalidRewardState` | 5 | Defined on the contract; not returned by the functions in this document |

Deposit, withdraw, and claim currently update vault accounting only. They do
not transfer `deposit_token` or `reward_token`.

---

## Write functions

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

On success the contract also stores `TotalDeposits = 0` and `RewardBalance = 0`.

**SDK mapping**

`VaultContract` construction (`new VaultContract({ client, contractId, wallet })`)
only binds the SDK to a deployed contract ID. It does not call this function.

SDK contributors should invoke contract symbol `initialize` with arguments in
this order: `admin`, `deposit_token`, `reward_token`. There is no
`VaultContract.initialize(...)` helper on the current SDK wrapper.

---

### `deposit`

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

Maps to `vault.deposit({ amount, from? })`.

- If `from` is omitted, the SDK should use the connected wallet public key.
- Encode arguments in **contract order**: `from` (`Address`), then `amount`
  (`i128`).
- Decode the return value as `i128` (updated user balance).
- The current SDK encoder sends `[amount, from]`. That order does not match
  this Network function and should be aligned to `[from, amount]`.
- Do not assume share minting. This contract has no share token and no
  `preview_deposit` method.

---

### `withdraw`

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

Maps to `vault.withdraw({ amount, to? })`.

- If `to` is omitted, the SDK should use the connected wallet public key.
- Encode arguments in **contract order**: `to` (`Address`), then `amount`
  (`i128`).
- Decode the return value as `i128` (updated user balance).
- The current SDK encoder sends `[amount, to]`. That order does not match this
  Network function and should be aligned to `[to, amount]`.
- Do not assume share redemption or `preview_withdraw`. Those methods are not
  on this contract.

---

### `claim_rewards`

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
separate from vault-wide `RewardBalance` (see `pending_rewards` below). This
contract does not transfer `reward_token` on claim.

**SDK mapping**

Maps to `vault.claimRewards()`.

- Invoke contract symbol `claim_rewards` with one argument: `user` (`Address`).
  Use the connected wallet public key when the SDK helper takes no address.
- The current SDK helper sends **no** contract arguments. This Network
  function requires `user`.
- Decode the return value as `i128` (claimed amount). A `0` result is success,
  not an error.
- Do not preflight this call against `pending_rewards()`; that query is not
  per-user.

---

## Query functions

All queries except `is_initialized` require a successful `initialize`. They
are read-only and need no auth.

### `total_deposits`

**Inputs**

None.

**Output**

`Result<i128, VaultError>` — sum of all credited user balances.

**SDK mapping**

There is no `VaultContract` helper for this query yet. SDK contributors should
add a read helper that simulates `total_deposits` with an empty argument list
and decodes `i128`. Do not map this to `totalAssets` / `totalSupply`; those
symbols are not on this contract.

---

### Balance query (`user_balance`)

**Inputs**

| Argument | Type | Notes |
| --- | --- | --- |
| `user` | `Address` | Account whose deposited balance is returned |

**Output**

`Result<i128, VaultError>` — that user's stored deposit balance, or `0` if
they have never deposited.

**SDK mapping**

Maps to `vault.getBalance(account?)`.

- Invoke contract symbol `user_balance` with `user` (`Address`).
- If `account` is omitted, the SDK should use the connected wallet public key.
- Decode as `i128`.
- The current SDK helper queries `balance`, not `user_balance`. Align the
  symbol to `user_balance`.
- `getVaultShares` / `shares_of` are not backed by this contract. User
  deposit state is this balance query only.

---

### Pending rewards query (`pending_rewards`)

**Inputs**

None.

**Output**

`Result<i128, VaultError>` — vault-wide `RewardBalance`.

After `initialize` this value is `0`. No write function in this contract
updates `RewardBalance`. This is **not** a per-user unclaimed-rewards query.
Per-user claimable amounts live in `ClaimableReward` and are not exposed by a
dedicated public query today.

**SDK mapping**

Maps to a vault-wide pending-rewards read helper, for example
`vault.getPendingRewards()` with **no** address argument.

- Invoke contract symbol `pending_rewards` with an empty argument list.
- Decode as `i128`.
- Do not call `pending_rewards(user)`. That signature does not exist here.
- Do not treat this value as the amount `claim_rewards` will return for a
  given user.

---

### Configuration query

There is no single `configuration` method. Configuration is four independent
reads:

| Function | Inputs | Output | Meaning |
| --- | --- | --- | --- |
| `is_initialized` | none | `bool` | `true` after a successful `initialize` |
| `admin` | none | `Result<Address, VaultError>` | Address passed to `initialize` |
| `deposit_token` | none | `Result<Address, VaultError>` | Deposit token address |
| `reward_token` | none | `Result<Address, VaultError>` | Reward token address |

`is_initialized` is safe to call before setup. The other three return
`NotInitialized` until `initialize` succeeds.

**SDK mapping**

Maps to a configuration helper that simulates these four symbols with empty
argument lists (no `getVaultInfo()` fields).

`VaultContract.getVaultInfo()` in the SDK reads `totalAssets`, `totalSupply`,
`apy`, and `lockPeriod`. Those methods are not on this Network vault. SDK
contributors should wrap `is_initialized`, `admin`, `deposit_token`, and
`reward_token` instead.

---

## SDK helper checklist

| Contract function | Expected SDK helper | Contract args (after `env`) | Contract return |
| --- | --- | --- | --- |
| `initialize` | none yet (deployment-time invoke) | `admin`, `deposit_token`, `reward_token` | `()` |
| `deposit` | `vault.deposit({ amount, from? })` | `from`, `amount` | `i128` new balance |
| `withdraw` | `vault.withdraw({ amount, to? })` | `to`, `amount` | `i128` new balance |
| `claim_rewards` | `vault.claimRewards()` | `user` | `i128` claimed amount |
| `total_deposits` | none yet | _(none)_ | `i128` |
| `user_balance` | `vault.getBalance(account?)` | `user` | `i128` |
| `pending_rewards` | pending-rewards read helper | _(none)_ | `i128` vault-wide |
| `is_initialized` | config helper | _(none)_ | `bool` |
| `admin` | config helper | _(none)_ | `Address` |
| `deposit_token` | config helper | _(none)_ | `Address` |
| `reward_token` | config helper | _(none)_ | `Address` |

Functions that appear in SDK wrappers but are **not** part of this Network
interface: `preview_deposit`, `preview_withdraw`, `get_shares`,
`get_exchange_rate`, `shares_of`, `exchange_rate`, `balance`, `totalAssets`,
`totalSupply`, `apy`, `lockPeriod`.
