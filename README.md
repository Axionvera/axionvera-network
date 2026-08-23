# Axionvera Network

Axionvera Network is a clean Rust and Soroban workspace for vault deposits, withdrawals, rewards, and supporting network-node services.

## Current Scope

- Soroban vault contract
- Reward distribution engine
- Rust network node
- Tests and contributor documentation

## Planned Modules

- `contracts/vault-contract` - Soroban vault contract
- `contracts/rewards` - reward simulation and distribution logic
- `network-node` - off-chain network service
- `docs` - architecture and contributor docs
- `tests` - integration tests

## Documentation

- [SDK v2 to vault contract interface](docs/sdk-contract-interface.md) — method, argument, response, and event mapping between Axionvera SDK v2 and this network vault.

## Development

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
