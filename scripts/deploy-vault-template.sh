#!/bin/bash
set -e

# Dry-run deployment script template for Axionvera Vault Contract
# This script validates the required environment variables and prints
# the intended deployment steps. It does not perform a real deployment.
# Real deployment is only performed by the maintainer using explicitly loaded keys.

echo "=================================================="
echo " Axionvera Vault Contract - Dry Run Deployment"
echo "=================================================="
echo ""
echo "NOTE: This is a template script. It performs a dry-run by default."
echo "Real deployment should only be performed by the maintainer."
echo ""

# Load environment variables if .env exists
if [ -f .env ]; then
    source .env
fi

MISSING_VARS=0

check_var() {
    if [ -z "${!1}" ]; then
        echo "Error: Required environment variable $1 is not set."
        MISSING_VARS=1
    fi
}

check_var "AXIONVERA_NETWORK_NAME"
check_var "AXIONVERA_DEPLOYER_SOURCE"
check_var "AXIONVERA_ADMIN_ADDRESS"
check_var "AXIONVERA_DEPOSIT_TOKEN"
check_var "AXIONVERA_REWARD_TOKEN"

if [ $MISSING_VARS -eq 1 ]; then
    echo ""
    echo "Please set the required variables in your .env file or environment."
    exit 1
fi

WASM_PATH="target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm"

echo "Configuration Validated:"
echo "  Network:       $AXIONVERA_NETWORK_NAME"
echo "  Deployer:      $AXIONVERA_DEPLOYER_SOURCE"
echo "  Admin:         $AXIONVERA_ADMIN_ADDRESS"
echo "  Deposit Token: $AXIONVERA_DEPOSIT_TOKEN"
echo "  Reward Token:  $AXIONVERA_REWARD_TOKEN"
echo "  WASM Path:     $WASM_PATH"
echo ""

echo "Intended Deployment Steps:"
echo ""
echo "1. Deploy WASM:"
echo "   stellar contract deploy \\"
echo "     --wasm $WASM_PATH \\"
echo "     --source $AXIONVERA_DEPLOYER_SOURCE \\"
echo "     --network $AXIONVERA_NETWORK_NAME \\"
echo "     --alias axionvera-vault-$AXIONVERA_NETWORK_NAME"
echo ""
echo "2. Initialize Contract (using the returned <CONTRACT_ID>):"
echo "   stellar contract invoke \\"
echo "     --id <CONTRACT_ID> \\"
echo "     --source $AXIONVERA_DEPLOYER_SOURCE \\"
echo "     --network $AXIONVERA_NETWORK_NAME \\"
echo "     -- initialize \\"
echo "     --admin $AXIONVERA_ADMIN_ADDRESS \\"
echo "     --deposit_token $AXIONVERA_DEPOSIT_TOKEN \\"
echo "     --reward_token $AXIONVERA_REWARD_TOKEN"
echo ""
echo "Dry run completed successfully. No real deployment was performed."
