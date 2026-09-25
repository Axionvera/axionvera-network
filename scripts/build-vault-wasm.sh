#!/bin/bash
set -e

# Build script for Axionvera Vault Contract WASM target
# This script builds the vault contract as a wasm32v1-none target

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTRACT_DIR="$PROJECT_ROOT/contracts/vault-contract"
TARGET_DIR="$PROJECT_ROOT/target"

echo "Building Axionvera Vault Contract for WASM..."
echo "Project root: $PROJECT_ROOT"
echo "Contract directory: $CONTRACT_DIR"
echo "WASM target directory: $TARGET_DIR"
echo ""

# Check if contract directory exists
if [ ! -d "$CONTRACT_DIR" ]; then
    echo "Error: Contract directory not found at $CONTRACT_DIR"
    exit 1
fi

# Add wasm32 target if not already installed
echo "Checking for wasm32v1-none target..."
if ! rustup target list --installed | grep -q "wasm32v1-none"; then
    echo "Adding wasm32v1-none target..."
    rustup target add wasm32v1-none
else
    echo "wasm32v1-none target already installed"
fi
echo ""

# Build the contract using the Stellar CLI so the supported Soroban
# target and deployment optimisation settings are applied.
echo "Building vault contract with stellar contract build..."
(
    cd "$PROJECT_ROOT"
    stellar contract build \
        --package axionvera-vault-contract \
        --locked
)

# Check if build succeeded
if [ $? -eq 0 ]; then
    WASM_PATH="target/wasm32v1-none/release/axionvera_vault_contract.wasm"
    METADATA_PATH="target/wasm32v1-none/release/axionvera_vault_contract.metadata.json"
    
    echo "Generating build metadata..."
    
    # Get git commit or default to UNCOMMITTED
    if git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
        COMMIT=$(git rev-parse HEAD 2>/dev/null || echo "UNCOMMITTED")
        # Check for uncommitted changes
        if ! git diff-index --quiet HEAD -- 2>/dev/null; then
            COMMIT="UNCOMMITTED"
        fi
    else
        COMMIT="UNCOMMITTED"
    fi
    
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    
    # Cross-platform sha256
    if command -v sha256sum > /dev/null; then
        SHA256=$(sha256sum "$PROJECT_ROOT/$WASM_PATH" | awk '{print $1}')
    elif command -v shasum > /dev/null; then
        SHA256=$(shasum -a 256 "$PROJECT_ROOT/$WASM_PATH" | awk '{print $1}')
    else
        echo "Warning: No sha256sum or shasum found, using placeholder"
        SHA256="0000000000000000000000000000000000000000000000000000000000000000"
    fi
    
    cat > "$PROJECT_ROOT/$METADATA_PATH" << EOF
{
  "schema_version": "1",
  "package": "axionvera-vault-contract",
  "target": "wasm32v1-none",
  "artifact_path": "$WASM_PATH",
  "sha256": "$SHA256",
  "build_timestamp": "$TIMESTAMP",
  "source_commit": "$COMMIT"
}
EOF
    
    echo ""
    echo "Build successful!"
    echo "WASM file location: $TARGET_DIR/wasm32v1-none/release/axionvera_vault_contract.wasm"
    echo "Metadata file location: $TARGET_DIR/wasm32v1-none/release/axionvera_vault_contract.metadata.json"
else
    echo ""
    echo "Build failed!"
    exit 1
fi
