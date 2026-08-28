#!/usr/bin/env python3
"""Validate a mocked vault deployment handoff without network or secrets."""

import argparse
import json
import re
from pathlib import Path

CANONICAL_WASM_PATH = "target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm"
ADDRESS_RE = re.compile(r"^(ADDRESS_PLACEHOLDER|G[A-Z2-7]{55}|C[A-Z2-7]{55})$")
CONTRACT_RE = re.compile(r"^(CONTRACT_ID_PLACEHOLDER|C[A-Z2-7]{55})$")
COMMIT_RE = re.compile(r"^(UNCOMMITTED|[0-9a-fA-F]{40})$")
SHA_RE = re.compile(r"^(SHA256_PLACEHOLDER|[0-9a-fA-F]{64})$")


def validate(data: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["artifact must be a JSON object"]
    required = {
        "schema_version", "mocked", "status", "source_commit", "network_mode",
        "wasm", "contract_id", "initialization", "maintainer_deployment_boundary",
    }
    missing = sorted(required - data.keys())
    errors.extend(f"missing field: {key}" for key in missing)
    if missing:
        return errors
    if data["schema_version"] != "1": errors.append("schema_version must be '1'")
    if data["mocked"] is not True: errors.append("mocked must be true")
    if data["status"] != "ready_for_maintainer_deployment": errors.append("invalid status")
    if not isinstance(data["source_commit"], str) or not COMMIT_RE.fullmatch(data["source_commit"]): errors.append("invalid source_commit")
    if data["network_mode"] != "testnet": errors.append("network_mode must be testnet")
    wasm = data["wasm"]
    if not isinstance(wasm, dict):
        errors.append("wasm must be an object")
    else:
        if wasm.get("path") != CANONICAL_WASM_PATH: errors.append("wasm.path is not canonical")
        if not isinstance(wasm.get("sha256"), str) or not SHA_RE.fullmatch(wasm["sha256"]): errors.append("invalid wasm.sha256")
    if not isinstance(data["contract_id"], str) or not CONTRACT_RE.fullmatch(data["contract_id"]): errors.append("invalid contract_id")
    init = data["initialization"]
    if not isinstance(init, dict):
        errors.append("initialization must be an object")
    else:
        for name in ("admin", "deposit_token", "reward_token"):
            if not isinstance(init.get(name), str) or not ADDRESS_RE.fullmatch(init[name]): errors.append(f"invalid initialization.{name}")
    boundary = data["maintainer_deployment_boundary"]
    if not isinstance(boundary, dict):
        errors.append("maintainer_deployment_boundary must be an object")
    else:
        if boundary.get("required") is not True: errors.append("maintainer deployment must be required")
        if boundary.get("real_deployment_performed") is not False: errors.append("mock artifact cannot claim real deployment")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifact", type=Path)
    args = parser.parse_args()
    try:
        data = json.loads(args.artifact.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"INVALID: {exc}")
        return 1
    errors = validate(data)
    if errors:
        print("INVALID")
        for error in errors:
            print(f"- {error}")
        return 1
    print("READY: ready_for_maintainer_deployment")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
