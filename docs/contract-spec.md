# Contract Specification (Vault)

## Initialization

### `initialize(admin, deposit_token, reward_token) -> Result<(), VaultError>`

One-time initialization that sets:
- `admin`: authorized caller for `distribute_rewards`
- `deposit_token`: token used for deposits and withdrawals
- `reward_token`: token used for reward distributions and user claims

Security:
- Fails with `AlreadyInitialized` if called twice.
- Fails with `InvalidTokenConfiguration` if `deposit_token == reward_token`.
- Requires `admin` authorization.

Emits:
- `init`

## Deposits

### `deposit(from, amount) -> Result<(), VaultError>`

Transfers `amount` of `deposit_token` from `from` to the contract and increases `from`’s recorded vault balance.

Validations:
- `amount > 0`
- Requires `from` authorization
- Fails with `InsufficientBalance` if `from` does not hold enough `deposit_token`

Accounting:
- Accrues any pending rewards for `from` before changing their balance.
- Rejects invalid transfers before mutating user reward snapshots or vault balances.

Emits:
- `deposit`

## Withdrawals

### `withdraw(to, amount) -> Result<(), VaultError>`

Transfers `amount` of `deposit_token` from the contract to `to` and decreases `to`’s recorded vault balance.

Validations:
- `amount > 0`
- Requires `to` authorization
- Fails with `InsufficientBalance` if `amount > balance(to)`
- Fails with `InsufficientContractBalance` if the vault cannot cover the token transfer

Accounting:
- Accrues any pending rewards for `to` before changing their balance.
- Final state is only written after token transfer pre-checks succeed.

Emits:
- `withdraw`

## Reward Distribution

### `distribute_rewards(amount) -> Result<i128, VaultError>`

Transfers `amount` of `reward_token` from `admin` to the contract and increases the global `reward_index`.

Validations:
- `amount > 0`
- Requires `admin` authorization
- Fails with `NoDeposits` if `total_deposits == 0`
- Fails with `InsufficientBalance` if `admin` does not hold enough `reward_token`

Emits:
- `distrib`

## Claiming Rewards

### `claim_rewards(user) -> Result<i128, VaultError>`

Accrues pending rewards for `user`, transfers the claimable amount of `reward_token` from the contract to `user`, and resets `user`’s accrued reward counter.

Validations:
- Requires `user` authorization
- Fails with `InsufficientContractBalance` if the vault reward pool is underfunded

Emits:
- `claim` (only when amount > 0)

## Views

- `balance(user) -> Result<i128, VaultError>`
- `total_deposits() -> Result<i128, VaultError>`
- `reward_index() -> Result<i128, VaultError>`
- `pending_rewards(user) -> Result<i128, VaultError>`
- `admin() -> Result<Address, VaultError>`
- `deposit_token() -> Result<Address, VaultError>`
- `reward_token() -> Result<Address, VaultError>`

## Errors

- `AlreadyInitialized`: vault initialization can only happen once.
- `NotInitialized`: the vault must be initialized before use.
- `InvalidAmount`: token amounts must be greater than zero.
- `InsufficientBalance`: the caller-facing token balance is lower than the requested amount.
- `NoDeposits`: rewards cannot be distributed while `total_deposits == 0`.
- `InvalidTokenConfiguration`: deposit and reward token addresses must be different.
- `InsufficientContractBalance`: the vault does not hold enough tokens to complete the transfer.
- `MathOverflow`: arithmetic overflow or underflow was detected while updating accounting.
