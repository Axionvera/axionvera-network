# Axionvera Network Scripts

This directory contains deployment and utility scripts for the Axionvera Network project.

## Scripts Overview

### 🚀 deploy_testnet.sh
**One-click Testnet deployment script**

Automates the complete workflow for deploying the Soroban vault contract to Stellar Testnet:
1. Builds the WASM contract in release mode
2. Optimizes the WASM binary to reduce size
3. Deploys the contract to Testnet
4. Saves the Contract ID to `.env`

**Usage**:
```bash
npm run deploy:testnet
# or
./scripts/deploy_testnet.sh
```

**Documentation**: See [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md) and [QUICK_REFERENCE.md](./QUICK_REFERENCE.md)

---

### 📋 deploy.ts
**TypeScript deployment script**

Deploys a pre-built WASM contract to Soroban network using the Soroban CLI.

**Usage**:
```bash
npm run deploy
```

**Environment Variables**:
- `SOROBAN_NETWORK`: Network to deploy to (default: testnet)
- `SOROBAN_SOURCE`: Funded CLI identity (default: default)
- `VAULT_WASM`: Path to WASM file

---

### ⚙️ initialize.ts
**Contract initialization script**

Initializes the deployed vault contract with admin account and token addresses.

**Usage**:
```bash
npm run initialize
```

**Required Environment Variables**:
- `VAULT_CONTRACT_ID`: Deployed contract ID
- `VAULT_ADMIN`: Admin account ID
- `VAULT_DEPOSIT_TOKEN`: Deposit token contract ID
- `VAULT_REWARD_TOKEN`: Reward token contract ID
- `SOROBAN_NETWORK`: Network (default: testnet)
- `SOROBAN_SOURCE`: Funded CLI identity (default: default)

---

### 🏗️ deploy-infrastructure.sh
**Terraform infrastructure deployment**

Deploys the complete Axionvera Network infrastructure to AWS using Terraform.

**Usage**:
```bash
./scripts/deploy-infrastructure.sh
```

**Prerequisites**:
- Terraform >= 1.5.0
- AWS CLI configured
- AWS credentials with appropriate permissions

---

### 🔒 security-scan.sh
**Security scanning script**

Runs security scans on the Docker image using Trivy.

**Usage**:
```bash
./scripts/security-scan.sh
```

---

## Deployment Workflow

### Quick Start (Testnet)

```bash
# 1. One-command deployment
npm run deploy:testnet

# 2. Update .env with token addresses
# Edit .env and set:
#   VAULT_ADMIN=<your-admin-account>
#   VAULT_DEPOSIT_TOKEN=<deposit-token-id>
#   VAULT_REWARD_TOKEN=<reward-token-id>

# 3. Initialize contract
npm run initialize

# 4. Test
npm run test:integration
```

### Full Workflow (Build → Deploy → Initialize)

```bash
# 1. Build contract
npm run build:contracts

# 2. Deploy to Testnet (builds, optimizes, deploys)
npm run deploy:testnet

# 3. Initialize contract
npm run initialize

# 4. Run tests
npm run test:integration
```

### Production Deployment (AWS)

```bash
# 1. Deploy infrastructure
./scripts/deploy-infrastructure.sh

# 2. Deploy contract to mainnet
SOROBAN_NETWORK=public npm run deploy:testnet

# 3. Initialize contract
npm run initialize
```

---

## Documentation

### For New Contributors
Start here:
1. [QUICK_REFERENCE.md](./QUICK_REFERENCE.md) - Quick lookup guide
2. [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md) - Comprehensive guide
3. [../TESTNET_DEPLOYMENT_IMPLEMENTATION.md](../TESTNET_DEPLOYMENT_IMPLEMENTATION.md) - Implementation details

### For Experienced Developers
- [QUICK_REFERENCE.md](./QUICK_REFERENCE.md) - Common scenarios and commands

### For Maintainers
- [../TESTNET_DEPLOYMENT_IMPLEMENTATION.md](../TESTNET_DEPLOYMENT_IMPLEMENTATION.md) - Architecture and maintenance

---

## Environment Configuration

### .env File
The deployment scripts use a `.env` file for configuration:

```bash
# Soroban Network Configuration
SOROBAN_NETWORK=testnet
SOROBAN_SOURCE=default

# Vault Contract Configuration
VAULT_CONTRACT_ID=CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA

# Token Configuration
VAULT_ADMIN=GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
VAULT_DEPOSIT_TOKEN=CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
VAULT_REWARD_TOKEN=CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
```

### Environment Variables
Override defaults using environment variables:

```bash
export SOROBAN_NETWORK=testnet
export SOROBAN_SOURCE=my-account
export VAULT_WASM_PATH=/custom/path/contract.wasm
npm run deploy:testnet
```

---

## Prerequisites

### For Testnet Deployment
- Rust toolchain with `wasm32-unknown-unknown` target
- Soroban CLI
- Funded Stellar Testnet account
- Node.js 18+

### For Infrastructure Deployment
- Terraform >= 1.5.0
- AWS CLI configured
- AWS credentials with appropriate permissions

### For Security Scanning
- Docker
- Trivy (installed automatically)

---

## Troubleshooting

### Common Issues

**"soroban: command not found"**
```bash
cargo install soroban-cli
```

**"wasm32-unknown-unknown target not installed"**
```bash
rustup target add wasm32-unknown-unknown
```

**"Soroban identity not configured"**
```bash
soroban config identity generate --name default
```

**"Network not configured"**
```bash
soroban network add \
  --name testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"
```

**"Insufficient funds"**
Fund your account at: https://developers.stellar.org/docs/build/apps/test-data#testnet-friendbot

For more troubleshooting, see [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md#troubleshooting)

---

## Integration with npm

All scripts are integrated with npm for easy access:

```bash
npm run build:contracts      # Build WASM contract
npm run deploy:testnet       # Deploy to Testnet (build + optimize + deploy)
npm run deploy               # Deploy pre-built WASM
npm run initialize           # Initialize contract
npm run test:integration     # Run integration tests
npm run lint                 # Lint and format code
npm run format               # Format code
```

---

## CI/CD Integration

The scripts are designed for CI/CD pipelines:

```yaml
# GitHub Actions example
- name: Deploy to Testnet
  env:
    SOROBAN_NETWORK: testnet
    SOROBAN_SOURCE: ci-account
  run: npm run deploy:testnet
```

---

## Security Considerations

- Never commit `.env` files with private keys to version control
- Use separate accounts for testing and production
- Always verify you're deploying to the correct network
- Review contract code before deployment
- Monitor contract activity after deployment

---

## Additional Resources

- [Soroban Documentation](https://developers.stellar.org/docs/build/smart-contracts)
- [Stellar Testnet](https://developers.stellar.org/docs/build/apps/test-data#testnet)
- [Soroban CLI Reference](https://github.com/stellar/rs-soroban-cli)
- [Contract Specification](../docs/contract-spec.md)
- [Architecture Guide](../ARCHITECTURE.md)

---

## Support

For issues or questions:
1. Check the [Troubleshooting](#troubleshooting) section
2. Review the [DEPLOYMENT_GUIDE.md](./DEPLOYMENT_GUIDE.md)
3. Check the [Stellar Developer Discord](https://discord.gg/stellardev)
4. Open an issue on the [GitHub repository](https://github.com/axionvera/axionvera-network)

---

## Contributing

When adding new scripts:
1. Follow the existing naming conventions
2. Add comprehensive comments
3. Include error handling
4. Update this README
5. Add documentation for new scripts
6. Test thoroughly before committing

---

## License

All scripts are licensed under Apache-2.0. See [../LICENSE](../LICENSE) for details.
