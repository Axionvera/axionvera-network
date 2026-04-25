# Testnet Deployment Guide

This guide explains how to use the `deploy_testnet.sh` script to deploy the Axionvera vault contract to Stellar Testnet.

## Overview

The `deploy_testnet.sh` script automates the entire deployment workflow:

1. **Build**: Compiles the Soroban contract to WASM (WebAssembly)
2. **Optimize**: Strips unnecessary bloat from the WASM binary to reduce deployment costs
3. **Deploy**: Uploads the contract to Stellar Testnet using the Soroban CLI
4. **Configure**: Saves the deployed Contract ID to `.env` for the network-node to consume

## Prerequisites

Before running the deployment script, ensure you have:

### 1. Rust Toolchain
Install Rust and the `wasm32-unknown-unknown` target:

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add the wasm32-unknown-unknown target
rustup target add wasm32-unknown-unknown
```

### 2. Soroban CLI
Install the Soroban CLI from the official repository:

```bash
# Using cargo (recommended)
cargo install soroban-cli

# Or download pre-built binaries from:
# https://github.com/stellar/rs-soroban-cli/releases
```

Verify installation:
```bash
soroban --version
```

### 3. Funded Stellar Testnet Account
You need a funded account on Stellar Testnet to pay for contract deployment.

**Create and fund an account:**

```bash
# Generate a new keypair
soroban config identity generate --name my-account

# Get your public key
soroban config identity show my-account

# Fund your account using the Stellar Testnet Friendbot:
# https://developers.stellar.org/docs/build/apps/test-data#testnet-friendbot
```

### 4. Configure Soroban CLI for Testnet
Add the Testnet network configuration:

```bash
soroban network add \
  --name testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"
```

Verify the network is configured:
```bash
soroban network ls
```

## Quick Start

### Basic Deployment

Deploy to Testnet using default settings:

```bash
./scripts/deploy_testnet.sh
```

This will:
- Use the `default` Soroban identity
- Deploy to the `testnet` network
- Save the Contract ID to `.env`

### Custom Deployment

Deploy with a specific account and network:

```bash
./scripts/deploy_testnet.sh --network testnet --source my-account
```

### Using Environment Variables

Alternatively, set environment variables before running:

```bash
export SOROBAN_NETWORK=testnet
export SOROBAN_SOURCE=my-account
./scripts/deploy_testnet.sh
```

## Script Output

The script provides detailed output at each stage:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Axionvera Network - Testnet Deployment
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ℹ️  Starting deployment process...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Checking Prerequisites
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✅ Rust toolchain found: cargo 1.75.0
✅ wasm32-unknown-unknown target found
✅ Soroban CLI found: soroban 21.0.0
✅ Soroban identity 'default' is configured
✅ Network 'testnet' is configured

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Building WASM Contract
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ℹ️  Building contract for wasm32-unknown-unknown target...
✅ WASM contract built successfully
ℹ️  Location: /path/to/target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm
ℹ️  Size: 256K

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Optimizing WASM Contract
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ℹ️  Running soroban contract optimize...
✅ WASM contract optimized successfully
ℹ️  Original size: 256K
ℹ️  Optimized size: 128K
ℹ️  Optimized location: /path/to/target/wasm32-unknown-unknown/release/axionvera_vault_contract.optimized.wasm

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Deploying Contract to testnet
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ℹ️  Deploying WASM contract using Soroban CLI...
✅ Contract deployed successfully
ℹ️  Contract ID: CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Saving Configuration
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

ℹ️  Saving Contract ID to .env...
✅ Contract ID saved to .env
ℹ️  File: /path/to/.env

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Deployment Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Deployment completed successfully!

📋 Deployment Details:
  Network:      testnet
  Source:       default
  Contract ID:  CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
  Config File:  /path/to/.env

🔧 Next Steps:
  1. Set the required environment variables in .env:
     - VAULT_ADMIN: Your admin account ID
     - VAULT_DEPOSIT_TOKEN: Deposit token contract ID
     - VAULT_REWARD_TOKEN: Reward token contract ID

  2. Initialize the contract:
     npm run initialize

  3. Test the contract:
     npm run test:integration
```

## After Deployment

### 1. Update Environment Variables

The script creates or updates `.env` with the Contract ID. You still need to set:

```bash
# Edit .env and add:
VAULT_ADMIN=GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
VAULT_DEPOSIT_TOKEN=CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
VAULT_REWARD_TOKEN=CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
```

### 2. Initialize the Contract

Once the environment variables are set, initialize the contract:

```bash
npm run initialize
```

This calls the contract's `initialize` function with your admin account and token addresses.

### 3. Test the Deployment

Run integration tests to verify the contract works:

```bash
npm run test:integration
```

## Troubleshooting

### "Soroban CLI not found"

Install Soroban CLI:
```bash
cargo install soroban-cli
```

### "wasm32-unknown-unknown target not installed"

The script will attempt to install it automatically. If it fails:
```bash
rustup target add wasm32-unknown-unknown
```

### "Soroban identity not configured"

Create and configure an identity:
```bash
soroban config identity generate --name my-account
```

### "Network 'testnet' not configured"

Add the Testnet network:
```bash
soroban network add \
  --name testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"
```

### "Contract deployment failed"

Common causes:
- **Insufficient funds**: Fund your account using Friendbot
- **Network issues**: Check your internet connection and RPC endpoint
- **Invalid WASM**: Ensure the contract builds successfully with `cargo build`

### "Invalid Contract ID format"

The script validates the Contract ID format. If you see this error:
- Check the Soroban CLI output for error messages
- Verify your account has sufficient funds
- Try deploying again

## Understanding the Script

### Build Stage

```bash
cargo build -p axionvera-vault-contract --target wasm32-unknown-unknown --release
```

This compiles the Rust contract to WebAssembly. The `--release` flag enables optimizations.

### Optimization Stage

```bash
soroban contract optimize --wasm <path-to-wasm>
```

This uses the Soroban CLI to optimize the WASM binary by:
- Removing unused code
- Stripping debug symbols
- Applying compression techniques

This reduces deployment costs on Stellar.

### Deployment Stage

```bash
soroban contract deploy --wasm <path-to-wasm> --source <account> --network <network>
```

This uploads the contract to Stellar Testnet. The Soroban CLI:
1. Reads the WASM binary
2. Creates a deployment transaction
3. Signs it with your account
4. Submits it to the Stellar network
5. Returns the Contract ID

### Configuration Stage

The script saves the Contract ID to `.env` for the network-node to consume:

```bash
VAULT_CONTRACT_ID=CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
```

## Advanced Usage

### Custom WASM Path

Override the WASM path:
```bash
VAULT_WASM_PATH=/custom/path/contract.wasm ./scripts/deploy_testnet.sh
```

### Custom .env File

Save configuration to a different file:
```bash
./scripts/deploy_testnet.sh --env-file .env.testnet
```

### Dry Run (Check Prerequisites Only)

To verify everything is set up without deploying:
```bash
./scripts/deploy_testnet.sh --help
# Then manually run each stage
```

## Integration with CI/CD

The script is designed to work in CI/CD pipelines:

```yaml
# Example GitHub Actions workflow
- name: Deploy to Testnet
  env:
    SOROBAN_NETWORK: testnet
    SOROBAN_SOURCE: ci-account
  run: ./scripts/deploy_testnet.sh
```

## Security Considerations

- **Private Keys**: Never commit `.env` files with private keys to version control
- **Account Funding**: Use a separate account for testing, not your production account
- **Network Passphrase**: Always verify you're deploying to the correct network
- **Contract Verification**: After deployment, verify the contract on Stellar Expert

## Additional Resources

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar Testnet](https://developers.stellar.org/docs/build/apps/test-data#testnet)
- [Soroban CLI Reference](https://github.com/stellar/rs-soroban-cli)
- [Contract Specification](../docs/contract-spec.md)
- [Architecture Guide](../ARCHITECTURE.md)

## Support

For issues or questions:
1. Check the [Troubleshooting](#troubleshooting) section
2. Review the [Stellar Developer Discord](https://discord.gg/stellardev)
3. Open an issue on the [GitHub repository](https://github.com/axionvera/axionvera-network)
