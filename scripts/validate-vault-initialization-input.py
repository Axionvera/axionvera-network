#!/usr/bin/env python3
"""Validate vault initialization input parameters without network or secrets."""

import argparse
import json
import re
from pathlib import Path

CONTRACT_RE = re.compile(r"^(CONTRACT_ID_PLACEHOLDER|C[A-Z2-7]{55})$")
ADMIN_RE = re.compile(r"^(ADMIN_PUBKEY_PLACEHOLDER|ADMIN_PUBLIC_KEY_PLACEHOLDER|G[A-Z2-7]{55})$")
TOKEN_RE = re.compile(r"^(TOKEN_CONTRACT_ID_PLACEHOLDER|DEPOSIT_TOKEN_CONTRACT_ID_PLACEHOLDER|REWARD_TOKEN_CONTRACT_ID_PLACEHOLDER|C[A-Z2-7]{55})$")
VALID_NETWORKS = {"local", "testnet", "futurenet", "mainnet"}


def validate(data: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["artifact must be a JSON object"]
    required = {
        "schema_version",
        "network",
        "contract_id",
        "admin_public_key",
        "deposit_token_contract_id",
        "reward_token_contract_id",
        "maintainer_initialization_boundary",
    }
    missing = sorted(required - data.keys())
    errors.extend(f"missing field: {key}" for key in missing)
    if missing:
        return errors

    if data["schema_version"] != "1":
        errors.append("schema_version must be 1")

    if data["network"] not in VALID_NETWORKS:
        errors.append(f"invalid network: {data.get(network)}; expected one of {sorted(VALID_NETWORKS)}")

    if not isinstance(data["contract_id"], str) or not CONTRACT_RE.fullmatch(data["contract_id"]):
        errors.append("invalid contract_id")

    if not isinstance(data["admin_public_key"], str) or not ADMIN_RE.fullmatch(data["admin_public_key"]):
        errors.append("invalid admin_public_key")

    if not isinstance(data["deposit_token_contract_id"], str) or not TOKEN_RE.fullmatch(data["deposit_token_contract_id"]):
        errors.append("invalid deposit_token_contract_id")

    if not isinstance(data["reward_token_contract_id"], str) or not TOKEN_RE.fullmatch(data["reward_token_contract_id"]):
        errors.append("invalid reward_token_contract_id")

    boundary = data["maintainer_initialization_boundary"]
    if not isinstance(boundary, dict):
        errors.append("maintainer_initialization_boundary must be an object")
    else:
        if boundary.get("required") is not True:
            errors.append("maintainer initialization must be required")
        if boundary.get("executed") is not False:
            errors.append("mock input artifact cannot claim already executed initialization")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifact", type=Path, help="Path to vault initialization input JSON file")
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
    print("READY: ready_for_maintainer_initialization")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
