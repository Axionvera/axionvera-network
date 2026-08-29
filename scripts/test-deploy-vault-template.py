#!/usr/bin/env python3
"""Tests for the dry-run deployment template script."""

import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/deploy-vault-template.sh"

def run(env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(SCRIPT)],
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False
    )

def main() -> int:
    base_env = os.environ.copy()
    
    # Test 1: Missing environment variables should fail
    result_missing = run(base_env)
    assert result_missing.returncode != 0, "Script should fail when variables are missing"
    assert "Error: Required environment variable" in result_missing.stdout, result_missing.stdout
    
    # Test 2: Full environment variables should succeed and print dry-run output
    valid_env = base_env.copy()
    valid_env["AXIONVERA_NETWORK_NAME"] = "testnet"
    valid_env["AXIONVERA_DEPLOYER_SOURCE"] = "deployer"
    valid_env["AXIONVERA_ADMIN_ADDRESS"] = "GADMIN..."
    valid_env["AXIONVERA_DEPOSIT_TOKEN"] = "CDEPOSIT..."
    valid_env["AXIONVERA_REWARD_TOKEN"] = "CREWARD..."
    
    result_valid = run(valid_env)
    assert result_valid.returncode == 0, f"Script failed with output:\n{result_valid.stdout}\n{result_valid.stderr}"
    assert "Dry run completed successfully" in result_valid.stdout, "Script did not confirm dry-run success"
    assert "stellar contract deploy" in result_valid.stdout, "Script did not output deploy command"
    assert "stellar contract invoke" in result_valid.stdout, "Script did not output invoke command"
    
    # Check that variables were correctly substituted
    assert "GADMIN..." in result_valid.stdout
    assert "CDEPOSIT..." in result_valid.stdout
    assert "CREWARD..." in result_valid.stdout
    
    print("PASS dry-run deployment template script validation")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
