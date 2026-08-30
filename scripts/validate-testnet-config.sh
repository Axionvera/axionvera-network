#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIG_PATH="examples/testnet-config.json"

cd "$PROJECT_ROOT"

echo "Validating the network-node testnet JSON example..."
cargo run --locked --quiet \
  --package axionvera-network-node \
  --example validate_config \
  -- "$CONFIG_PATH"

echo "Validating the complete testnet example set..."
cargo test --locked --quiet \
  --package axionvera-network-node \
  testnet_example

echo "VALID: testnet JSON and .env examples are complete, consistent, and safe."
