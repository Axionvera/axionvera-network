#!/usr/bin/env python3
"""Validate a non-secret vault contract deployment artifact against specifications."""

import argparse
import json
import re
import sys
from pathlib import Path

CONTRACT_RE = re.compile(r"^(CONTRACT_ID_PLACEHOLDER|C[A-Z2-7]{55})$")
PUBLIC_KEY_RE = re.compile(r"^(DEPLOYER_PUBLIC_KEY_PLACEHOLDER|G[A-Z2-7]{55})$")
WASM_HASH_RE = re.compile(r"^(WASM_HASH_PLACEHOLDER|[0-9a-fA-F]{64})$")
TIMESTAMP_RE = re.compile(r"^(TIMESTAMP_PLACEHOLDER|\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2}))$")
VALID_NETWORKS = {"testnet", "futurenet", "standalone", "mainnet"}
VALID_INIT_STATUSES = {"uninitialized", "initialized", "pending_initialization"}

SECRET_PATTERN_RE = re.compile(r"(secp256k1|private_key|secret_key|seed|S[A-Z2-7]{55})", re.IGNORECASE)


def validate(data: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["artifact must be a JSON object"]

    raw_text = json.dumps(data)
    if SECRET_PATTERN_RE.search(raw_text):
        errors.append("secret or private key detected in artifact payload")

    required = {
        "schema_version",
        "contract_id",
        "network",
        "deployer_public_key",
        "wasm_hash",
        "deployment_timestamp",
        "initialization_status",
    }
    missing = sorted(required - data.keys())
    errors.extend(f"missing field: {key}" for key in missing)
    if missing:
        return errors

    if data["schema_version"] not in {"1", "1.0"}:
        errors.append("schema_version must be '1' or '1.0'")

    if not isinstance(data["contract_id"], str) or not CONTRACT_RE.fullmatch(data["contract_id"]):
        errors.append("invalid contract_id format")

    if data["network"] not in VALID_NETWORKS:
        errors.append(f"network must be one of {sorted(VALID_NETWORKS)}")

    if not isinstance(data["deployer_public_key"], str) or not PUBLIC_KEY_RE.fullmatch(data["deployer_public_key"]):
        errors.append("invalid deployer_public_key format")

    if not isinstance(data["wasm_hash"], str) or not WASM_HASH_RE.fullmatch(data["wasm_hash"]):
        errors.append("invalid wasm_hash format")

    if not isinstance(data["deployment_timestamp"], str) or not TIMESTAMP_RE.fullmatch(data["deployment_timestamp"]):
        errors.append("invalid deployment_timestamp format (must be ISO-8601 or placeholder)")

    if data["initialization_status"] not in VALID_INIT_STATUSES:
        errors.append(f"initialization_status must be one of {sorted(VALID_INIT_STATUSES)}")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifact_path", type=Path, help="Path to deployment artifact JSON")
    args = parser.parse_args()

    if not args.artifact_path.exists():
        print(f"Error: file not found: {args.artifact_path}", file=sys.stderr)
        return 1

    try:
        data = json.loads(args.artifact_path.read_text(encoding="utf-8"))
    except Exception as err:
        print(f"Error: failed to parse JSON: {err}", file=sys.stderr)
        return 1

    errors = validate(data)
    if errors:
        print(f"Validation failed with {len(errors)} error(s):", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        return 1

    print(f"Validation successful for {args.artifact_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())