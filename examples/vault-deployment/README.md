# Mocked vault deployment handoff

This example is a contributor-safe, deterministic handoff. It does **not** deploy a contract, contact Stellar RPC, load an identity, or use a secret.

## Flow

```text
build output
  -> deployment artifact
  -> placeholder contract identity
  -> initialization inputs
  -> readiness validation
  -> maintainer-only deployment boundary
```

The canonical WASM path is:

```text
target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm
```

The initialization fields mirror the contract exactly:

- `admin`
- `deposit_token`
- `reward_token`

The example deliberately uses placeholders. `mocked: true`, `status`, and
`real_deployment_performed: false` prevent this fixture from being presented as
on-chain deployment evidence.

## Generate and validate

From the repository root:

```bash
python3 scripts/create-mock-vault-deployment.py \
  --output /tmp/axionvera-vault-deployment.json
python3 scripts/validate-mock-vault-deployment.py \
  /tmp/axionvera-vault-deployment.json
```

The generator accepts an optional source commit and SHA-256 value. It never
accepts or loads signing keys. A real contract ID and real public addresses may
be entered only by the maintainer during the actual deployment handoff.

## Maintainer boundary

After this validator reports `ready_for_maintainer_deployment`, the contributor
flow ends. The maintainer-controlled checklist in
`docs/testnet-deployment-checklist.md` is responsible for:

1. building the locked WASM artifact;
2. deploying it to Stellar testnet;
3. recording the returned contract ID and transaction;
4. initializing exactly once with the three addresses; and
5. validating the live contract state.

This fixture is not evidence that any of those steps occurred.
