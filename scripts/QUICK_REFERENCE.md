# Testnet Deployment - Quick Reference

## One-Command Deployment

```bash
npm run deploy:testnet
```

Or directly:
```bash
./scripts/deploy_testnet.sh
```

## Common Scenarios

### First Time Setup

```bash
# 1. Install Soroban CLI
cargo install soroban-cli

# 2. Create and fund an account
soroban config identity generate --name my-account
# Fund at: https://developers.stellar.org/docs/build/apps/test-data#testnet-friendbot

# 3. Configure Testnet
soroban network add \
  --name testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

# 4. Deploy
npm run deploy:testnet
```

### Deploy with Custom Account

```bash
./scripts/deploy_testnet.sh --source my-account
```

### Deploy to Different Network

```bash
./scripts/deploy_testnet.sh --network futurenet
```

### View Help

```bash
./scripts/deploy_testnet.sh --help
```

## What the Script Does

| Stage | Command | Output |
|-------|---------|--------|
| **Build** | `cargo build --target wasm32-unknown-unknown --release` | WASM binary |
| **Optimize** | `soroban contract optimize` | Optimized WASM (50% smaller) |
| **Deploy** | `soroban contract deploy` | Contract ID |
| **Configure** | Save to `.env` | `VAULT_CONTRACT_ID=C...` |

## After Deployment

```bash
# 1. Edit .env with token addresses
VAULT_ADMIN=GAAAA...
VAULT_DEPOSIT_TOKEN=CAAA...
VAULT_REWARD_TOKEN=CAAA...

# 2. Initialize contract
npm run initialize

# 3. Test
npm run test:integration
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `soroban: command not found` | `cargo install soroban-cli` |
| `wasm32-unknown-unknown not found` | `rustup target add wasm32-unknown-unknown` |
| `identity not configured` | `soroban config identity generate --name default` |
| `network not configured` | See "First Time Setup" above |
| `insufficient funds` | Fund account at Friendbot |

## Environment Variables

```bash
# Override defaults
export SOROBAN_NETWORK=testnet
export SOROBAN_SOURCE=my-account
export VAULT_WASM_PATH=/custom/path/contract.wasm

npm run deploy:testnet
```

## Useful Commands

```bash
# List configured identities
soroban config identity ls

# List configured networks
soroban network ls

# Query deployed contract
soroban contract info --id CAAAA... --network testnet

# Invoke contract function
soroban contract invoke \
  --id CAAAA... \
  --network testnet \
  --source my-account \
  -- balance --user GAAAA...
```

## File Locations

- **Script**: `scripts/deploy_testnet.sh`
- **Guide**: `scripts/DEPLOYMENT_GUIDE.md`
- **Config**: `.env` (created after deployment)
- **Contract**: `contracts/vault-contract/`
- **WASM Output**: `target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm`
- **Optimized WASM**: `target/wasm32-unknown-unknown/release/axionvera_vault_contract.optimized.wasm`

## Documentation

- [Full Deployment Guide](./DEPLOYMENT_GUIDE.md)
- [Contract Specification](../docs/contract-spec.md)
- [Architecture](../ARCHITECTURE.md)
- [Soroban Docs](https://developers.stellar.org/docs/build/smart-contracts)
