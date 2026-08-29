# Testnet Configuration Examples

This repository provides a contributor-safe Stellar testnet configuration set
for the network node and the vault contract tooling:

- `.env.example` mirrors the network-node fields as environment values and
  contains the variables consumed by `scripts/deploy-vault-template.sh`.
- `examples/testnet-config.json` is a directly loadable `NodeConfig` example.

Both files use the public Stellar testnet RPC endpoint and contain no secret
seed, mnemonic, API key, credential, live contract ID, or private endpoint.
The contract values are deliberately invalid placeholders, so the committed
examples cannot authorize or perform a deployment.

## Network-node values

The following fields are supported by `network-node/src/config.rs`. The JSON
field and environment variable in each row represent the same setting.

| JSON field | Environment variable | Example value | Meaning |
| --- | --- | --- | --- |
| `network_name` | `AXIONVERA_NETWORK_NAME` | `testnet` | Selects Stellar testnet. Supported values are `local`, `testnet`, `mainnet`, and `futurenet`. |
| `rpc_url` | `AXIONVERA_RPC_URL` | `https://soroban-testnet.stellar.org` | Public Soroban JSON-RPC endpoint used for testnet requests. It is public infrastructure, not a secret or private project endpoint. |
| `environment` | `AXIONVERA_ENVIRONMENT` | `staging` | Separates testnet runtime state and logs from local development and production. Supported values are `development`, `staging`, and `production`. |

`NodeConfig::validate` checks that the network and environment names are
supported and that the RPC URL has a valid URL and port shape. The Stellar CLI
commands use its built-in `testnet` network alias, which supplies the public
Stellar testnet passphrase (`Test SDF Network ; September 2015`). A passphrase
is therefore not a `NodeConfig` or `.env.example` field.

## Contract-tooling values

The dry-run deployment template reads these additional variables from `.env`:

| Environment variable | Committed example | Meaning |
| --- | --- | --- |
| `AXIONVERA_DEPLOYER_SOURCE` | `DEPLOYER_IDENTITY_PLACEHOLDER` | Local Stellar CLI identity name selected and controlled by the maintainer. It is an alias, never a secret seed. |
| `AXIONVERA_ADMIN_ADDRESS` | `G_ADMIN_ADDRESS_PLACEHOLDER` | Public Stellar account address authorized as vault admin. |
| `AXIONVERA_DEPOSIT_TOKEN` | `C_DEPOSIT_TOKEN_PLACEHOLDER` | Public testnet contract ID for the accepted deposit token. |
| `AXIONVERA_REWARD_TOKEN` | `C_REWARD_TOKEN_PLACEHOLDER` | Public testnet contract ID for the reward token. |

The checked-in placeholders are intentionally not valid Stellar addresses.
Only a maintainer should replace them, and only in an ignored `.env` file or a
maintainer-controlled secret store. Public account and contract IDs may be
recorded according to project policy, but private keys and seed phrases must
never be written to these files.

## Prepare a private testnet configuration

From the repository root:

```bash
cp .env.example .env
```

In the ignored `.env`, replace the four `*_PLACEHOLDER` values with a funded
Stellar CLI identity alias and the required public testnet addresses. Keep the
three network-node values unchanged unless the maintainer has selected another
trusted testnet RPC provider.

The JSON example is ready to load without editing:

```rust
use axionvera_network_node::load_config;

let config = load_config("examples/testnet-config.json")?;
```

## Validate without secrets or network access

Validate the complete committed example set with one command:

```bash
./scripts/validate-testnet-config.sh
```

The script first loads and validates `examples/testnet-config.json` through the
real Rust `load_config` path. It then runs fixture tests that confirm the JSON
and `.env.example` values match, all required contract placeholders are
present, and the example uses the expected public RPC endpoint. Validation does
not contact the RPC service and does not require a Stellar identity.

The underlying commands can also be run separately:

```bash
cargo run --locked --package axionvera-network-node \
  --example validate_config -- examples/testnet-config.json
cargo test --locked --package axionvera-network-node testnet_example
```

After creating a private `.env`, preview the contract commands with:

```bash
./scripts/deploy-vault-template.sh
```

That script is a dry run: it prints the intended commands and never submits a
transaction. Actual deployment remains a maintainer-only step; follow the
[Vault Contract Testnet Deployment Checklist](./testnet-deployment-checklist.md).
