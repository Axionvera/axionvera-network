#!/bin/bash
set -e

# Contributor-safe post-deployment smoke test runner template
# Validates contract ID resolution, initialization input loading,
# and mocked lifecycle behavior without live secrets or RPC calls.

echo "======================================================"
echo " Axionvera - Mocked Post-Deployment Smoke Test Runner"
echo "======================================================"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

python3 "$ROOT_DIR/scripts/run-mocked-smoke-test.py" "$@"
