#!/usr/bin/env python3
"""Validate or generate post-deployment verification inputs and reports for Axionvera Vault contracts."""

import argparse
import json
import os
import re
import sys
from pathlib import Path

CONTRACT_RE = re.compile(r"^(CONTRACT_ID_PLACEHOLDER|C[A-Z2-7]{55})$")
ADDRESS_RE = re.compile(r"^(ADDRESS_PLACEHOLDER|ADMIN_PUBKEY_PLACEHOLDER|G[A-Z2-7]{55})$")
TOKEN_RE = re.compile(r"^(ADDRESS_PLACEHOLDER|TOKEN_CONTRACT_ID_PLACEHOLDER|C[A-Z2-7]{55})$")
VALID_NETWORKS = {"local", "testnet", "futurenet", "mainnet"}
VALID_MODES = {"dry_run", "mocked", "live"}


def validate_input(data: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["input artifact must be a JSON object"]
    required = {
        "schema_version",
        "contract_id",
        "network",
        "admin_address",
        "deposit_token",
        "reward_token",
        "mode",
    }
    missing = sorted(required - data.keys())
    errors.extend(f"missing field: {k}" for k in missing)
    if missing:
        return errors

    if data.get("schema_version") != "1":
        errors.append("schema_version must be '1'")
    if data.get("network") not in VALID_NETWORKS:
        errors.append(f"invalid network: {data.get('network')}")
    if not isinstance(data.get("contract_id"), str) or not CONTRACT_RE.fullmatch(data["contract_id"]):
        errors.append("invalid contract_id")
    if not isinstance(data.get("admin_address"), str) or not ADDRESS_RE.fullmatch(data["admin_address"]):
        errors.append("invalid admin_address")
    if not isinstance(data.get("deposit_token"), str) or not TOKEN_RE.fullmatch(data["deposit_token"]):
        errors.append("invalid deposit_token")
    if not isinstance(data.get("reward_token"), str) or not TOKEN_RE.fullmatch(data["reward_token"]):
        errors.append("invalid reward_token")
    if data.get("mode") not in VALID_MODES:
        errors.append(f"invalid mode: {data.get('mode')}")

    return errors


def validate_report(data: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["report artifact must be a JSON object"]
    required = {
        "schema_version",
        "contract_id",
        "network",
        "mode",
        "verification_checks",
        "maintainer_verification_boundary",
    }
    missing = sorted(required - data.keys())
    errors.extend(f"missing field: {k}" for k in missing)
    if missing:
        return errors

    if data.get("schema_version") != "1":
        errors.append("schema_version must be '1'")
    if data.get("network") not in VALID_NETWORKS:
        errors.append(f"invalid network: {data.get('network')}")
    if not isinstance(data.get("contract_id"), str) or not CONTRACT_RE.fullmatch(data["contract_id"]):
        errors.append("invalid contract_id")
    if data.get("mode") not in VALID_MODES:
        errors.append(f"invalid mode: {data.get('mode')}")

    checks = data.get("verification_checks")
    if not isinstance(checks, dict):
        errors.append("verification_checks must be an object")
    else:
        for b in ("contract_id_presence", "network_config_valid"):
            if not isinstance(checks.get(b), bool):
                errors.append(f"verification_checks.{b} must be a boolean")

        init = checks.get("initialization_state")
        if not isinstance(init, dict):
            errors.append("verification_checks.initialization_state must be an object")
        else:
            if not isinstance(init.get("verified"), bool):
                errors.append("initialization_state.verified must be a boolean")
            if not isinstance(init.get("admin_address"), str) or not ADDRESS_RE.fullmatch(init["admin_address"]):
                errors.append("invalid initialization_state.admin_address")
            if not isinstance(init.get("deposit_token"), str) or not TOKEN_RE.fullmatch(init["deposit_token"]):
                errors.append("invalid initialization_state.deposit_token")
            if not isinstance(init.get("reward_token"), str) or not TOKEN_RE.fullmatch(init["reward_token"]):
                errors.append("invalid initialization_state.reward_token")

        reads = checks.get("read_calls")
        if not isinstance(reads, dict):
            errors.append("verification_checks.read_calls must be an object")
        else:
            for r in ("total_deposits_accessible", "reward_balance_accessible"):
                if not isinstance(reads.get(r), bool):
                    errors.append(f"verification_checks.read_calls.{r} must be a boolean")

    boundary = data.get("maintainer_verification_boundary")
    if not isinstance(boundary, dict):
        errors.append("maintainer_verification_boundary must be an object")
    else:
        if boundary.get("required") is not True:
            errors.append("maintainer verification boundary required must be true")
        if boundary.get("live_secrets_used") is not False:
            errors.append("contributor verification report cannot use live secrets")

    return errors


def generate_mock_report(
    contract_id: str = "CONTRACT_ID_PLACEHOLDER",
    network: str = "testnet",
    admin: str = "ADMIN_PUBKEY_PLACEHOLDER",
    deposit_token: str = "TOKEN_CONTRACT_ID_PLACEHOLDER",
    reward_token: str = "TOKEN_CONTRACT_ID_PLACEHOLDER",
    mode: str = "dry_run",
) -> dict:
    return {
        "schema_version": "1",
        "contract_id": contract_id,
        "network": network,
        "mode": mode,
        "verification_checks": {
            "contract_id_presence": True,
            "network_config_valid": True,
            "initialization_state": {
                "verified": True,
                "admin_address": admin,
                "deposit_token": deposit_token,
                "reward_token": reward_token,
            },
            "read_calls": {
                "total_deposits_accessible": True,
                "reward_balance_accessible": True,
            },
        },
        "maintainer_verification_boundary": {
            "required": True,
            "live_secrets_used": False,
        },
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", nargs="?", type=Path, help="Path to verification report or input JSON")
    parser.add_argument("--input", type=Path, dest="input_path", help="Path to verification input JSON")
    parser.add_argument("--generate", action="store_true", help="Generate mocked verification report")
    parser.add_argument("--output", type=Path, help="Output path when generating")
    args = parser.parse_args()

    if args.generate:
        report = generate_mock_report()
        rendered = json.dumps(report, indent=2) + "\n"
        if args.output:
            args.output.write_text(rendered, encoding="utf-8")
            print(f"Generated mock verification report to {args.output}")
        else:
            print(rendered)
        return 0

    target = args.input_path or args.report
    if not target:
        parser.error("Either report path, --input, or --generate must be specified.")

    try:
        data = json.loads(target.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"INVALID: {exc}")
        return 1

    # Check if target is an input configuration
    if "verification_checks" not in data and "admin_address" in data:
        errors = validate_input(data)
        if errors:
            print("INVALID")
            for err in errors:
                print(f"- {err}")
            return 1
        print("READY: post-deployment verification input valid")
        return 0

    # Otherwise validate as report
    errors = validate_report(data)
    if errors:
        print("INVALID")
        for err in errors:
            print(f"- {err}")
        return 1

    print("VERIFIED: post-deployment verification checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
