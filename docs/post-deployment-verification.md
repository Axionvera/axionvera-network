# Post-Deployment Verification Template

## Overview
After a maintainer executes a testnet or production deployment of the Axionvera Vault Soroban contract (`axionvera-vault-contract`), post-deployment verification must be conducted to ensure that:
1. The contract ID is present and formatted correctly on the target network.
2. The network configuration matches the expected deployment target.
3. The contract is properly initialized with valid admin and token addresses.
4. Basic read calls (`total_deposits`, `reward_balance`, `is_initialized`) return expected initial states.

To ensure safety across contributors and CI pipelines, this repository provides contributor-safe script templates and schemas that operate in **dry-run / mocked mode by default**, with zero live secrets or private keys required.

## Artifacts & Components
- **Input Schema**: [`schemas/post-deployment-verification-input.schema.json`](../schemas/post-deployment-verification-input.schema.json)
- **Output Report Schema**: [`schemas/post-deployment-verification.schema.json`](../schemas/post-deployment-verification.schema.json)
- **Example Input**: [`examples/post-deployment-verification-input.json`](../examples/post-deployment-verification-input.json)
- **Example Output**: [`examples/post-deployment-verification.json`](../examples/post-deployment-verification.json)
- **Verification CLI**: [`scripts/verify-vault-deployment.py`](../scripts/verify-vault-deployment.py)
- **Verification Shell Template**: [`scripts/verify-vault-deployment.sh`](../scripts/verify-vault-deployment.sh)
- **Automated Tests**: [`scripts/test-verify-vault-deployment.py`](../scripts/test-verify-vault-deployment.py)

## Schema Definitions

### Verification Input
| Field | Type | Description | Values / Pattern |
|---|---|---|---|
| `schema_version` | String | Schema version | `"1"` |
| `contract_id` | String | Deployed vault contract ID | `C[A-Z2-7]{55}` or `CONTRACT_ID_PLACEHOLDER` |
| `network` | String | Stellar network | `local`, `testnet`, `futurenet`, `mainnet` |
| `admin_address` | String | Vault admin Stellar address | `G[A-Z2-7]{55}` or `ADMIN_PUBKEY_PLACEHOLDER` |
| `deposit_token` | String | Accepted deposit token contract | `C[A-Z2-7]{55}` or `TOKEN_CONTRACT_ID_PLACEHOLDER` |
| `reward_token` | String | Reward distribution token contract | `C[A-Z2-7]{55}` or `TOKEN_CONTRACT_ID_PLACEHOLDER` |
| `mode` | String | Execution mode | `dry_run`, `mocked`, `live` |

### Verification Report Output
| Field | Type | Description |
|---|---|---|
| `schema_version` | String | Must be `"1"` |
| `contract_id` | String | Target contract ID verified |
| `network` | String | Verified network |
| `mode` | String | `dry_run`, `mocked`, or `live` |
| `verification_checks` | Object | Results of contract ID, network, initialization, and read checks |
| `maintainer_verification_boundary` | Object | Enforces `required: true` and `live_secrets_used: false` |

## Contributor Usage (Dry-Run / Mocked)

### 1. Validate Input Configuration
```bash
python3 scripts/verify-vault-deployment.py --input examples/post-deployment-verification-input.json
```
Output:
```text
READY: post-deployment verification input valid
```

### 2. Validate Verification Report
```bash
python3 scripts/verify-vault-deployment.py examples/post-deployment-verification.json
```
Output:
```text
VERIFIED: post-deployment verification checks passed
```

### 3. Generate Mock Verification Report
```bash
python3 scripts/verify-vault-deployment.py --generate --output /tmp/mock-report.json
```

### 4. Run Shell Verification Template (Dry-Run)
```bash
AXIONVERA_NETWORK_NAME="testnet" AXIONVERA_VAULT_CONTRACT_ID="CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" AXIONVERA_DEPOSIT_TOKEN="CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB" AXIONVERA_REWARD_TOKEN="CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC" ./scripts/verify-vault-deployment.sh
```

### 5. Run Validation Test Suite
```bash
python3 scripts/test-verify-vault-deployment.py
```
Output:
```text
PASS post-deployment verification script template and report validation
```

## Maintainer Live Verification Guide

Following a live deployment and contract initialization on Stellar Testnet, the maintainer executes live read calls against the deployed contract:

1. **Verify Contract ID and Network**:
   ```bash
   stellar contract inspect --id <DEPLOYED_CONTRACT_ID> --network testnet
   ```

2. **Verify Initialization State**:
   ```bash
   stellar contract invoke --id <DEPLOYED_CONTRACT_ID> --network testnet -- is_initialized
   ```

3. **Verify Vault Admin and Tokens**:
   ```bash
   stellar contract invoke --id <DEPLOYED_CONTRACT_ID> --network testnet -- owner
   stellar contract invoke --id <DEPLOYED_CONTRACT_ID> --network testnet -- deposit_token
   stellar contract invoke --id <DEPLOYED_CONTRACT_ID> --network testnet -- reward_token
   ```

4. **Verify Initial Balances**:
   ```bash
   stellar contract invoke --id <DEPLOYED_CONTRACT_ID> --network testnet -- total_deposits
   stellar contract invoke --id <DEPLOYED_CONTRACT_ID> --network testnet -- reward_balance
   ```

All read calls succeed without consuming gas from user balances and confirm the contract is ready for integration testing.
