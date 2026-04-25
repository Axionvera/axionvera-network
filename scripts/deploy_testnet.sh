#!/bin/bash

################################################################################
# Axionvera Network - Testnet Deployment Script
#
# This script automates the complete process of building, optimizing, and
# deploying the Soroban vault contract to Stellar Testnet. It handles:
#   1. Building the WASM contract in release mode
#   2. Optimizing the WASM binary to reduce size
#   3. Deploying the contract to Testnet using Soroban CLI
#   4. Saving the deployed Contract ID to .env for network-node consumption
#
# Prerequisites:
#   - Rust toolchain with wasm32-unknown-unknown target
#   - Soroban CLI (https://github.com/stellar/rs-soroban-cli)
#   - A funded Stellar Testnet account configured as SOROBAN_SOURCE
#   - jq (for JSON parsing, optional but recommended)
#
# Usage:
#   ./scripts/deploy_testnet.sh [OPTIONS]
#
# Options:
#   --network NETWORK     Soroban network (default: testnet)
#   --source ACCOUNT      Funded CLI identity (default: default)
#   --env-file PATH       Path to .env file (default: .env)
#   --help                Show this help message
#
# Environment Variables:
#   SOROBAN_NETWORK       Override network (default: testnet)
#   SOROBAN_SOURCE        Override source account (default: default)
#   VAULT_WASM_PATH       Override WASM output path
#
# Example:
#   ./scripts/deploy_testnet.sh --network testnet --source my-account
#
################################################################################

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
NETWORK="${SOROBAN_NETWORK:-testnet}"
SOURCE="${SOROBAN_SOURCE:-default}"
ENV_FILE="${PROJECT_ROOT}/.env"
WASM_PATH="${VAULT_WASM_PATH:-${PROJECT_ROOT}/target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm}"
WASM_OPTIMIZED="${WASM_PATH%.wasm}.optimized.wasm"

################################################################################
# Utility Functions
################################################################################

# Print colored output
log_info() {
    echo -e "${BLUE}ℹ️  $*${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $*${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $*${NC}"
}

log_error() {
    echo -e "${RED}❌ $*${NC}"
}

# Print section header
print_header() {
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$*${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Show help message
show_help() {
    head -n 30 "$0" | tail -n +2 | sed 's/^# //'
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --network)
                NETWORK="$2"
                shift 2
                ;;
            --source)
                SOURCE="$2"
                shift 2
                ;;
            --env-file)
                ENV_FILE="$2"
                shift 2
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

################################################################################
# Prerequisite Checks
################################################################################

check_prerequisites() {
    print_header "Checking Prerequisites"

    # Check Rust toolchain
    if ! command_exists cargo; then
        log_error "Rust toolchain not found. Please install from https://rustup.rs/"
        exit 1
    fi
    log_success "Rust toolchain found: $(cargo --version)"

    # Check wasm32-unknown-unknown target
    if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
        log_warning "wasm32-unknown-unknown target not installed. Installing..."
        rustup target add wasm32-unknown-unknown
        log_success "wasm32-unknown-unknown target installed"
    else
        log_success "wasm32-unknown-unknown target found"
    fi

    # Check Soroban CLI
    if ! command_exists soroban; then
        log_error "Soroban CLI not found. Please install from https://github.com/stellar/rs-soroban-cli"
        exit 1
    fi
    log_success "Soroban CLI found: $(soroban --version)"

    # Check if source account is configured
    if ! soroban config identity show "$SOURCE" >/dev/null 2>&1; then
        log_error "Soroban identity '$SOURCE' not configured"
        log_info "Configure it with: soroban config identity generate --name $SOURCE"
        exit 1
    fi
    log_success "Soroban identity '$SOURCE' is configured"

    # Verify network connectivity
    log_info "Verifying network connectivity to $NETWORK..."
    if ! soroban network ls | grep -q "$NETWORK"; then
        log_error "Network '$NETWORK' not configured in Soroban CLI"
        log_info "Configure it with: soroban network add --name $NETWORK --rpc-url <RPC_URL> --network-passphrase <PASSPHRASE>"
        exit 1
    fi
    log_success "Network '$NETWORK' is configured"
}

################################################################################
# Build Stage
################################################################################

build_wasm() {
    print_header "Building WASM Contract"

    log_info "Building contract for wasm32-unknown-unknown target..."
    log_info "Command: cargo build -p axionvera-vault-contract --target wasm32-unknown-unknown --release"

    cd "$PROJECT_ROOT"
    cargo build -p axionvera-vault-contract --target wasm32-unknown-unknown --release

    if [[ ! -f "$WASM_PATH" ]]; then
        log_error "WASM build failed. Contract not found at: $WASM_PATH"
        exit 1
    fi

    local wasm_size
    wasm_size=$(du -h "$WASM_PATH" | cut -f1)
    log_success "WASM contract built successfully"
    log_info "Location: $WASM_PATH"
    log_info "Size: $wasm_size"
}

################################################################################
# Optimization Stage
################################################################################

optimize_wasm() {
    print_header "Optimizing WASM Contract"

    log_info "Running soroban contract optimize..."
    log_info "This removes unnecessary bloat and reduces deployment costs"

    # Run optimization
    soroban contract optimize --wasm "$WASM_PATH"

    if [[ ! -f "$WASM_OPTIMIZED" ]]; then
        log_error "WASM optimization failed. Optimized contract not found at: $WASM_OPTIMIZED"
        exit 1
    fi

    # Compare sizes
    local original_size
    local optimized_size
    original_size=$(du -h "$WASM_PATH" | cut -f1)
    optimized_size=$(du -h "$WASM_OPTIMIZED" | cut -f1)

    log_success "WASM contract optimized successfully"
    log_info "Original size: $original_size"
    log_info "Optimized size: $optimized_size"
    log_info "Optimized location: $WASM_OPTIMIZED"

    # Use optimized WASM for deployment
    WASM_PATH="$WASM_OPTIMIZED"
}

################################################################################
# Deployment Stage
################################################################################

deploy_contract() {
    print_header "Deploying Contract to $NETWORK"

    log_info "Deploying WASM contract using Soroban CLI..."
    log_info "Network: $NETWORK"
    log_info "Source Account: $SOURCE"
    log_info "WASM Path: $WASM_PATH"

    # Deploy and capture output
    local deploy_output
    deploy_output=$(soroban contract deploy \
        --wasm "$WASM_PATH" \
        --source "$SOURCE" \
        --network "$NETWORK" 2>&1) || {
        log_error "Contract deployment failed"
        log_error "Output: $deploy_output"
        exit 1
    }

    # Extract Contract ID from output
    # The Soroban CLI returns the contract ID as the last line
    local contract_id
    contract_id=$(echo "$deploy_output" | tail -n 1 | xargs)

    # Validate Contract ID format (Stellar contract IDs are 56 characters)
    if [[ ! $contract_id =~ ^C[A-Z0-9]{55}$ ]]; then
        log_error "Invalid Contract ID format: $contract_id"
        log_error "Expected format: C followed by 55 alphanumeric characters"
        exit 1
    fi

    log_success "Contract deployed successfully"
    log_info "Contract ID: $contract_id"

    echo "$contract_id"
}

################################################################################
# Environment Configuration
################################################################################

save_contract_id() {
    local contract_id=$1

    print_header "Saving Configuration"

    log_info "Saving Contract ID to $ENV_FILE..."

    # Create or update .env file
    if [[ -f "$ENV_FILE" ]]; then
        # Backup existing .env
        cp "$ENV_FILE" "${ENV_FILE}.backup"
        log_info "Backed up existing .env to ${ENV_FILE}.backup"

        # Update or add VAULT_CONTRACT_ID
        if grep -q "^VAULT_CONTRACT_ID=" "$ENV_FILE"; then
            sed -i.bak "s/^VAULT_CONTRACT_ID=.*/VAULT_CONTRACT_ID=$contract_id/" "$ENV_FILE"
            rm -f "${ENV_FILE}.bak"
        else
            echo "VAULT_CONTRACT_ID=$contract_id" >> "$ENV_FILE"
        fi
    else
        # Create new .env file
        cat > "$ENV_FILE" << EOF
# Axionvera Network Configuration
# Generated by deploy_testnet.sh on $(date)

# Soroban Network Configuration
SOROBAN_NETWORK=$NETWORK
SOROBAN_SOURCE=$SOURCE

# Vault Contract Configuration
VAULT_CONTRACT_ID=$contract_id

# Token Configuration (set these before running initialize)
# VAULT_ADMIN=<admin-account-id>
# VAULT_DEPOSIT_TOKEN=<deposit-token-contract-id>
# VAULT_REWARD_TOKEN=<reward-token-contract-id>
EOF
    fi

    log_success "Contract ID saved to .env"
    log_info "File: $ENV_FILE"
}

################################################################################
# Post-Deployment Verification
################################################################################

verify_deployment() {
    local contract_id=$1

    print_header "Verifying Deployment"

    log_info "Querying contract on $NETWORK..."

    # Attempt to query contract info
    if soroban contract info --id "$contract_id" --network "$NETWORK" >/dev/null 2>&1; then
        log_success "Contract verified on $NETWORK"
    else
        log_warning "Could not verify contract immediately (may take a few seconds to finalize)"
    fi
}

################################################################################
# Summary and Next Steps
################################################################################

print_summary() {
    local contract_id=$1

    print_header "Deployment Summary"

    echo ""
    echo -e "${GREEN}Deployment completed successfully!${NC}"
    echo ""
    echo "📋 Deployment Details:"
    echo "  Network:      $NETWORK"
    echo "  Source:       $SOURCE"
    echo "  Contract ID:  $contract_id"
    echo "  Config File:  $ENV_FILE"
    echo ""
    echo "🔧 Next Steps:"
    echo "  1. Set the required environment variables in $ENV_FILE:"
    echo "     - VAULT_ADMIN: Your admin account ID"
    echo "     - VAULT_DEPOSIT_TOKEN: Deposit token contract ID"
    echo "     - VAULT_REWARD_TOKEN: Reward token contract ID"
    echo ""
    echo "  2. Initialize the contract:"
    echo "     npm run initialize"
    echo ""
    echo "  3. Test the contract:"
    echo "     npm run test:integration"
    echo ""
    echo "📚 Documentation:"
    echo "  - Contract Spec: docs/contract-spec.md"
    echo "  - Storage Layout: docs/contract-storage.md"
    echo "  - Architecture: ARCHITECTURE.md"
    echo ""
    echo "🔗 Useful Commands:"
    echo "  - Query contract: soroban contract info --id $contract_id --network $NETWORK"
    echo "  - Invoke function: soroban contract invoke --id $contract_id --network $NETWORK -- <function> <args>"
    echo ""
}

################################################################################
# Main Execution
################################################################################

main() {
    parse_args "$@"

    print_header "Axionvera Network - Testnet Deployment"
    log_info "Starting deployment process..."
    echo ""

    # Execute deployment stages
    check_prerequisites
    build_wasm
    optimize_wasm
    local contract_id
    contract_id=$(deploy_contract)
    save_contract_id "$contract_id"
    verify_deployment "$contract_id"
    print_summary "$contract_id"

    log_success "All done! Happy deploying! 🚀"
}

# Run main function
main "$@"
