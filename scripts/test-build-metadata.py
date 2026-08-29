#!/usr/bin/env python3
"""Validate WASM build metadata."""

import argparse
import json
import re
from pathlib import Path

CANONICAL_WASM_PATH = "target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm"
COMMIT_RE = re.compile(r"^(UNCOMMITTED|[0-9a-fA-F]{40})$")
SHA_RE = re.compile(r"^[0-9a-fA-F]{64}$")
TS_RE = re.compile(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$")


def validate(data: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["metadata must be a JSON object"]
    required = {
        "schema_version", "package", "target", "artifact_path", "sha256",
        "build_timestamp", "source_commit"
    }
    missing = sorted(required - data.keys())
    errors.extend(f"missing field: {key}" for key in missing)
    if missing:
        return errors
        
    if data["schema_version"] != "1": errors.append("schema_version must be '1'")
    if data["package"] != "axionvera-vault-contract": errors.append("package must be 'axionvera-vault-contract'")
    if data["target"] != "wasm32-unknown-unknown": errors.append("target must be 'wasm32-unknown-unknown'")
    if data["artifact_path"] != CANONICAL_WASM_PATH: errors.append("artifact_path is not canonical")
    
    if not isinstance(data["sha256"], str) or not SHA_RE.fullmatch(data["sha256"]): 
        errors.append("invalid sha256 format")
        
    if not isinstance(data["build_timestamp"], str) or not TS_RE.fullmatch(data["build_timestamp"]): 
        errors.append("invalid build_timestamp format")
        
    if not isinstance(data["source_commit"], str) or not COMMIT_RE.fullmatch(data["source_commit"]): 
        errors.append("invalid source_commit format")
        
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("metadata", type=Path)
    args = parser.parse_args()
    try:
        data = json.loads(args.metadata.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"INVALID: {exc}")
        return 1
    errors = validate(data)
    if errors:
        print("INVALID METADATA")
        for error in errors:
            print(f"- {error}")
        return 1
    print("VALID: build metadata format is correct")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
