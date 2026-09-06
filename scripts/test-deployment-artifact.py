#!/usr/bin/env python3
"""Unit tests for deployment artifact validation script."""

import sys
from pathlib import Path

# Add scripts directory to path to import validator
sys.path.insert(0, str(Path(__file__).parent))
from importlib import import_module

validator = import_module("validate-deployment-artifact")


def run_tests() -> None:
    # Test 1: Valid example artifact
    valid_artifact = {
        "schema_version": "1",
        "contract_id": "CONTRACT_ID_PLACEHOLDER",
        "network": "testnet",
        "deployer_public_key": "DEPLOYER_PUBLIC_KEY_PLACEHOLDER",
        "wasm_hash": "WASM_HASH_PLACEHOLDER",
        "deployment_timestamp": "2026-09-06T15:00:00Z",
        "initialization_status": "pending_initialization",
    }
    errors = validator.validate(valid_artifact)
    assert not errors, f"Expected valid artifact to pass, got: {errors}"

    # Test 2: Valid real Stellar keys
    valid_stellar = {
        "schema_version": "1",
        "contract_id": "CBCEJ2T6QZ6B2Q2K4PGL4ZJUPTX23DXZQ33A7C7KDFMGB2P2M4W4U2A3",
        "network": "testnet",
        "deployer_public_key": "GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFDAGORAc5X7A6Y66KBOB",
        "wasm_hash": "a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef0",
        "deployment_timestamp": "TIMESTAMP_PLACEHOLDER",
        "initialization_status": "initialized",
    }
    # Adjust valid pub key to uppercase 56 chars if regex checks standard Stellar
    valid_stellar["deployer_public_key"] = "GBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFDAGORAC5X7A6Y66KBOB"
    errors = validator.validate(valid_stellar)
    assert not errors, f"Expected stellar format to pass, got: {errors}"

    # Test 3: Missing required fields
    incomplete = {"schema_version": "1", "contract_id": "CONTRACT_ID_PLACEHOLDER"}
    errors = validator.validate(incomplete)
    assert any("missing field" in err for err in errors), "Expected missing field error"

    # Test 4: Invalid network
    invalid_net = dict(valid_artifact, network="ethereum")
    errors = validator.validate(invalid_net)
    assert any("network must be one of" in err for err in errors), "Expected invalid network error"

    # Test 5: Reject secrets / private keys
    secret_leak = dict(valid_artifact, deployer_public_key="SBRPYHIL2CI3FNQ4BXLFMNDLFJUNPU2HY3ZMFDAGORAC5X7A6Y66KBOB")
    errors = validator.validate(secret_leak)
    assert any("secret or private key detected" in err or "invalid deployer_public_key" in err for err in errors)

    print("All deployment artifact validator tests passed successfully!")


if __name__ == "__main__":
    run_tests()