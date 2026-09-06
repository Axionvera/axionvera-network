# Deployment Artifact Specification

## Overview
This document specifies the schema and usage of non-secret contract deployment artifacts within the `axionvera-network` ecosystem.

Contributors prepare these artifacts to decouple repository test flows from live chain deployments, ensuring zero credentials or private keys are checked into version control.

## Schema
The JSON schema is located at `schemas/deployment-artifact.schema.json`.

### Required Fields
* `schema_version`: String (`"1"` or `"1.0"`).
* `contract_id`: Canonical Soroban contract identifier (`C...`) or `CONTRACT_ID_PLACEHOLDER`.
* `network`: Target network (`testnet`, `futurenet`, `standalone`, `mainnet`).
* `deployer_public_key`: Deployer public key (`G...`) or `DEPLOYER_PUBLIC_KEY_PLACEHOLDER`.
* `wasm_hash`: SHA-256 hash of compiled contract wasm (`64 hex chars`) or `WASM_HASH_PLACEHOLDER`.
* `deployment_timestamp`: ISO-8601 UTC timestamp or `TIMESTAMP_PLACEHOLDER`.
* `initialization_status`: Status (`uninitialized`, `initialized`, `pending_initialization`).

## Validation
Validate any artifact file using the standalone script:

```bash
python3 scripts/validate-deployment-artifact.py examples/deployment-artifact.json