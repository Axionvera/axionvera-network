# Vault Initialization Input Validator

## Overview
After a maintainer deploys the Axionvera Vault Soroban contract (axionvera-vault-contract), the contract must be initialized before any gated operations (such as deposits, withdrawals, or rewards) can be executed.

To prevent misconfiguration, invalid public keys, or malformed contract IDs during maintainer operations, this repository provides:
1. A formal JSON schema (schemas/vault-initialization-input.schema.json).
2. A validator CLI (scripts/validate-vault-initialization-input.py).
3. Canonical offline sample inputs (examples/vault-initialization-input.json).
4. Automated regression tests (scripts/test-vault-initialization-input.py).

## Schema Fields

| Field | Type | Description | Allowed Values / Pattern |
|---|---|---|---|
| schema_version | String | Version of the initialization schema | 1 |
| network | String | Target Stellar network | local, testnet, futurenet, mainnet |
| contract_id | String | Deployed vault contract address | C[A-Z2-7]{55} or CONTRACT_ID_PLACEHOLDER |
| admin_public_key | String | Stellar account address for admin | G[A-Z2-7]{55} or ADMIN_PUBKEY_PLACEHOLDER |
| deposit_token_contract_id | String | Accepted deposit token contract | C[A-Z2-7]{55} or TOKEN_CONTRACT_ID_PLACEHOLDER |
| reward_token_contract_id | String | Distribution reward token contract | C[A-Z2-7]{55} or TOKEN_CONTRACT_ID_PLACEHOLDER |
| maintainer_initialization_boundary | Object | Safety boundary guard | required: true, executed: false |

## Usage

### 1. Validate an Input File
```bash
python3 scripts/validate-vault-initialization-input.py examples/vault-initialization-input.json
```

### 2. Run Regression Tests
```bash
python3 scripts/test-vault-initialization-input.py
```
Output:
```text
PASS vault initialization input validation and negative mutations
```

## Maintainer Live Execution Guide
Once inputs have passed validation, the maintainer executes initialization on-chain via the Soroban CLI:
```bash
soroban contract invoke   --id ""   --source ""   --network ""   -- initialize   --admin ""   --deposit_token ""   --reward_token ""
```
