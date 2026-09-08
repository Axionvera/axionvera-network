# Deployment Failure Examples and Mock Logs

This directory contains representative failure logs illustrating common breakdown scenarios during testnet deployment and initialization:

- `failed-build.log`: Toolchain target mismatch (`wasm32-unknown-unknown` missing or compilation errors).
- `failed-deploy-rpc-timeout.log`: Soroban testnet RPC gateway timeout, connection desync, or rate limiting.
- `failed-initialization-auth.log`: Contract initialization invocation failure (re-initialization attempt or invalid auth signature).
- `wrong-contract-id.log`: Mismatched contract ID or referencing contract from a different network or reset ledger.
- `failed-smoke-test.log`: Smoke test simulation failure where post-deployment state assertions do not match expectations.

Refer to [Deployment Failure and Recovery Runbook](../../docs/deployment-failure-and-recovery-runbook.md) for full diagnosis and step-by-step maintainer recovery instructions.
