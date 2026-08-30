#!/usr/bin/env bash
# Axionvera Network Release Readiness Checklist
# Usage:
#   ./scripts/release-readiness-check.sh
#   ./scripts/release-readiness-check.sh --full
#
# Verifies required docs, schemas, examples, scripts, repo files,
# and runs quick local quality commands. No secrets or privileged
# actions are performed.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FULL_CHECK=0
MISSING=0

for arg in "$@"; do
  case "$arg" in
    --full) FULL_CHECK=1 ;;
  esac
done

echo "=== Axionvera Network Release Readiness Checklist ==="
echo "Project root: $PROJECT_ROOT"
echo "Mode: $([ $FULL_CHECK -eq 1 ] && echo 'FULL' || echo 'QUICK')"
echo ""

check_file() {
  local rel_path="$1"
  local label="${2:-$rel_path}"
  if [[ -e "$PROJECT_ROOT/$rel_path" ]]; then
    echo "[OK] $label"
  else
    echo "[MISSING] $label ($rel_path)"
    MISSING=$((MISSING + 1))
  fi
}

echo "--- Required Documentation ---"
check_file "docs/maintainer-handoff-guide.md" "Maintainer Handoff Guide"
check_file "docs/testnet-deployment-checklist.md" "Testnet Deployment Checklist"
check_file "docs/ci-and-local-checks.md" "CI and Local Checks"
check_file "docs/testnet-configuration.md" "Testnet Configuration"
check_file "README.md" "README"
check_file "LICENSE" "License"
check_file "CONTRIBUTING.md" "Contributing Guide"
check_file "SECURITY.md" "Security Policy"

echo ""
echo "--- Required Schemas ---"
check_file "schemas/build-metadata.schema.json" "Build Metadata Schema"
check_file "schemas/mock-vault-deployment.schema.json" "Mock Vault Deployment Schema"
check_file "schemas/vault-event.schema.json" "Vault Event Schema"
check_file "schemas/sdk-handoff.schema.json" "SDK Handoff Schema"
check_file "schemas/contract-id-registry.schema.json" "Contract ID Registry Schema"

echo ""
echo "--- Required Examples ---"
check_file "examples/build-metadata.json" "Build Metadata Example"
check_file "examples/testnet-config.json" "Testnet Config Example"
check_file "examples/vault-deployment" "Vault Deployment Examples"
check_file "examples/vault-events" "Vault Event Examples"
check_file "examples/sdk-handoff.json" "SDK Handoff Example"
check_file "examples/contract-id-registry.json" "Contract ID Registry Example"

echo ""
echo "--- Required Scripts ---"
check_file "scripts/build-vault-wasm.sh" "Build Vault WASM"
check_file "scripts/deploy-vault-template.sh" "Deploy Vault Template"
check_file "scripts/validate-testnet-config.sh" "Validate Testnet Config"
check_file "scripts/validate-mock-vault-deployment.py" "Validate Mock Vault Deployment"
check_file "scripts/validate-sdk-handoff.py" "Validate SDK Handoff"
check_file "scripts/validate-contract-id-registry.py" "Validate Contract ID Registry"

echo ""
echo "--- Required Project Files ---"
check_file ".env.example" ".env Example"
check_file "Cargo.toml" "Workspace Cargo Manifest"
check_file "Cargo.lock" "Locked Dependencies"

echo ""
echo "--- Local Command Checks ---"

run_cmd() {
  local cmd="$1"
  local label="$2"
  local log_file="/tmp/release_readiness_$(echo "$label" | tr ' /' '_' | tr -cd '[:alnum:]_').log"
  echo "Checking $label ..."
  if eval "$cmd" > "$log_file" 2>&1; then
    echo "[OK] $label"
    rm -f "$log_file"
  else
    echo "[FAIL] $label (see $log_file)"
    MISSING=$((MISSING + 1))
  fi
}

run_cmd "cargo fmt --all -- --check" "cargo fmt --check"
run_cmd "cargo check --workspace --all-targets" "cargo check"
run_cmd "python3 scripts/test-sdk-handoff.py" "SDK handoff validation test"
run_cmd "python3 scripts/test-contract-id-registry.py" "Contract ID registry validation test"

if [[ $FULL_CHECK -eq 1 ]]; then
  run_cmd "cargo test --workspace --all-targets" "cargo test"
  run_cmd "cargo clippy --workspace --all-targets --all-features -- -D warnings" "cargo clippy"
else
  echo "[SKIPPED] cargo test (run with --full)"
  echo "[SKIPPED] cargo clippy (run with --full)"
fi

echo ""
if [[ $MISSING -eq 0 ]]; then
  echo "=== RELEASE READINESS: ALL CHECKS PASSED ==="
  exit 0
else
  echo "=== RELEASE READINESS: $MISSING MISSING / FAILED ITEMS ==="
  exit 1
fi
