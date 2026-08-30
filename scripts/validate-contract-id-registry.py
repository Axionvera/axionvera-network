#!/usr/bin/env python3
"""Validate a contract ID registry file without network or secrets."""

import argparse
import json
import re
from pathlib import Path

ADDRESS_RE = re.compile(r"^(ADDRESS_PLACEHOLDER|G[A-Z2-7]{55}|C[A-Z2-7]{55})$")
CONTRACT_RE = re.compile(r"^(CONTRACT_ID_PLACEHOLDER|C[A-Z2-7]{55})$")
SHA256_RE = re.compile(r"^(SHA256_PLACEHOLDER|[0-9a-fA-F]{64})$")
TIMESTAMP_RE = re.compile(
    r"^(TIMESTAMP_PLACEHOLDER|[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z)$"
)
SECRET_KEY_RE = re.compile(r"S[A-Z2-7]{55}")


def validate(data: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["registry must be a JSON object"]

    raw_json = json.dumps(data)
    if SECRET_KEY_RE.search(raw_json):
        errors.append("registry contains a Stellar secret key pattern (S...)")

    forbidden_terms = {"secret", "private_key", "secret_key", "seed"}
    for key in data.keys():
        if any(term in key.lower() for term in forbidden_terms):
            errors.append(f"forbidden sensitive field in registry: {key}")

    required = {
        "schema_version",
        "updated_at",
        "environments",
        "maintainer_deployment_boundary",
    }
    missing = sorted(required - data.keys())
    errors.extend(f"missing field: {key}" for key in missing)
    if missing:
        return errors

    if data["schema_version"] != "1":
        errors.append("schema_version must be '1'")

    if not isinstance(data["updated_at"], str) or not TIMESTAMP_RE.fullmatch(
        data["updated_at"]
    ):
        errors.append("invalid updated_at timestamp format")

    envs = data["environments"]
    if not isinstance(envs, dict) or not envs:
        errors.append("environments must be a non-empty object")
    else:
        for env_name, env_data in envs.items():
            if not isinstance(env_data, dict):
                errors.append(f"environment '{env_name}' must be an object")
                continue
            env_required = {"network", "status", "contracts"}
            env_missing = sorted(env_required - env_data.keys())
            errors.extend(
                f"missing field in environment '{env_name}': {k}" for k in env_missing
            )
            if env_missing:
                continue

            if env_data["network"] not in {"local", "testnet", "mainnet", "futurenet"}:
                errors.append(f"invalid network in environment '{env_name}'")
            if env_data["status"] not in {"placeholder", "deployed"}:
                errors.append(f"invalid status in environment '{env_name}'")

            contracts = env_data["contracts"]
            if not isinstance(contracts, dict) or not contracts:
                errors.append(f"contracts in environment '{env_name}' must be a non-empty object")
                continue

            for c_name, c_data in contracts.items():
                if not isinstance(c_data, dict):
                    errors.append(
                        f"contract '{c_name}' in '{env_name}' must be an object"
                    )
                    continue
                c_required = {
                    "contract_id",
                    "wasm_sha256",
                    "deployed_at",
                    "deployer_address",
                }
                c_missing = sorted(c_required - c_data.keys())
                errors.extend(
                    f"missing field in contract '{env_name}.{c_name}': {k}"
                    for k in c_missing
                )
                if c_missing:
                    continue

                if not isinstance(c_data["contract_id"], str) or not CONTRACT_RE.fullmatch(
                    c_data["contract_id"]
                ):
                    errors.append(
                        f"invalid contract_id format in '{env_name}.{c_name}'"
                    )
                if not isinstance(c_data["wasm_sha256"], str) or not SHA256_RE.fullmatch(
                    c_data["wasm_sha256"]
                ):
                    errors.append(
                        f"invalid wasm_sha256 format in '{env_name}.{c_name}'"
                    )
                if not isinstance(c_data["deployed_at"], str) or not TIMESTAMP_RE.fullmatch(
                    c_data["deployed_at"]
                ):
                    errors.append(
                        f"invalid deployed_at format in '{env_name}.{c_name}'"
                    )
                if not isinstance(
                    c_data["deployer_address"], str
                ) or not ADDRESS_RE.fullmatch(c_data["deployer_address"]):
                    errors.append(
                        f"invalid deployer_address format in '{env_name}.{c_name}'"
                    )

                if env_data["status"] == "deployed":
                    if c_data["contract_id"] == "CONTRACT_ID_PLACEHOLDER":
                        errors.append(
                            f"deployed status in '{env_name}.{c_name}' cannot use CONTRACT_ID_PLACEHOLDER"
                        )
                    if c_data["deployer_address"] == "ADDRESS_PLACEHOLDER":
                        errors.append(
                            f"deployed status in '{env_name}.{c_name}' cannot use ADDRESS_PLACEHOLDER"
                        )

    boundary = data["maintainer_deployment_boundary"]
    if not isinstance(boundary, dict):
        errors.append("maintainer_deployment_boundary must be an object")
    else:
        if boundary.get("no_secrets_included") is not True:
            errors.append(
                "maintainer_deployment_boundary.no_secrets_included must be true"
            )
        if not isinstance(boundary.get("real_deployments_recorded"), bool):
            errors.append(
                "maintainer_deployment_boundary.real_deployments_recorded must be a boolean"
            )

    has_deployed = (
        isinstance(envs, dict)
        and any(
            isinstance(v, dict) and v.get("status") == "deployed"
            for v in envs.values()
        )
    )
    if has_deployed and boundary.get("real_deployments_recorded") is not True:
        errors.append(
            "environment with 'deployed' status requires real_deployments_recorded to be true"
        )

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("registry", type=Path)
    args = parser.parse_args()
    try:
        data = json.loads(args.registry.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"INVALID: {exc}")
        return 1
    errors = validate(data)
    if errors:
        print("INVALID CONTRACT ID REGISTRY")
        for error in errors:
            print(f"- {error}")
        return 1
    print("VALID: contract ID registry format is correct")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
