#!/usr/bin/env python3
"""Validate an SDK handoff artifact package without network or secrets."""

import argparse
import json
import re
from pathlib import Path

ADDRESS_RE = re.compile(r"^(ADDRESS_PLACEHOLDER|G[A-Z2-7]{55}|C[A-Z2-7]{55})$")
CONTRACT_RE = re.compile(r"^(CONTRACT_ID_PLACEHOLDER|C[A-Z2-7]{55})$")
SECRET_KEY_RE = re.compile(r"S[A-Z2-7]{55}")
HTTP_URL_RE = re.compile(r"^https?://.+")


def validate(data: object) -> list[str]:
    errors: list[str] = []
    if not isinstance(data, dict):
        return ["artifact must be a JSON object"]

    raw_json = json.dumps(data)
    if SECRET_KEY_RE.search(raw_json):
        errors.append("artifact contains a Stellar secret key pattern (S...)")

    forbidden_terms = {"secret", "private_key", "secret_key", "seed"}
    for key in data.keys():
        if any(term in key.lower() for term in forbidden_terms):
            errors.append(f"forbidden sensitive field in artifact: {key}")

    required = {
        "schema_version",
        "status",
        "network",
        "contract_id",
        "interface_version",
        "interface_schema_ref",
        "event_schema_ref",
        "initialization",
        "maintainer_deployment_boundary",
    }
    missing = sorted(required - data.keys())
    errors.extend(f"missing field: {key}" for key in missing)
    if missing:
        return errors

    if data["schema_version"] != "1":
        errors.append("schema_version must be '1'")

    valid_statuses = {
        "placeholder_for_maintainer_deployment",
        "ready_for_sdk_consumption",
    }
    if data["status"] not in valid_statuses:
        errors.append(f"invalid status: {data['status']}")

    network = data["network"]
    if not isinstance(network, dict):
        errors.append("network must be an object")
    else:
        net_required = {"name", "rpc_url", "network_passphrase"}
        net_missing = sorted(net_required - network.keys())
        errors.extend(f"missing field in network: {k}" for k in net_missing)
        if not net_missing:
            if network["name"] not in {"local", "testnet", "mainnet", "futurenet"}:
                errors.append(f"invalid network.name: {network['name']}")
            if not isinstance(network["rpc_url"], str) or not HTTP_URL_RE.match(
                network["rpc_url"]
            ):
                errors.append("invalid network.rpc_url format")
            if (
                not isinstance(network["network_passphrase"], str)
                or not network["network_passphrase"]
            ):
                errors.append("network.network_passphrase must be a non-empty string")

    if not isinstance(data["contract_id"], str) or not CONTRACT_RE.fullmatch(
        data["contract_id"]
    ):
        errors.append("invalid contract_id format")

    if data["interface_version"] != "0.1":
        errors.append("interface_version must be '0.1'")

    if (
        not isinstance(data["interface_schema_ref"], str)
        or not data["interface_schema_ref"]
    ):
        errors.append("interface_schema_ref must be a non-empty string")

    if not isinstance(data["event_schema_ref"], str) or not data["event_schema_ref"]:
        errors.append("event_schema_ref must be a non-empty string")

    init = data["initialization"]
    if not isinstance(init, dict):
        errors.append("initialization must be an object")
    else:
        for name in ("admin", "deposit_token", "reward_token"):
            if not isinstance(init.get(name), str) or not ADDRESS_RE.fullmatch(
                init[name]
            ):
                errors.append(f"invalid initialization.{name}")

    boundary = data["maintainer_deployment_boundary"]
    if not isinstance(boundary, dict):
        errors.append("maintainer_deployment_boundary must be an object")
    else:
        if boundary.get("no_secrets_included") is not True:
            errors.append("maintainer_deployment_boundary.no_secrets_included must be true")
        if not isinstance(boundary.get("real_deployment_performed"), bool):
            errors.append(
                "maintainer_deployment_boundary.real_deployment_performed must be a boolean"
            )

    if data.get("status") == "ready_for_sdk_consumption":
        if boundary.get("real_deployment_performed") is not True:
            errors.append(
                "ready_for_sdk_consumption status requires real_deployment_performed to be true"
            )
        if data.get("contract_id") == "CONTRACT_ID_PLACEHOLDER":
            errors.append(
                "ready_for_sdk_consumption status cannot use CONTRACT_ID_PLACEHOLDER"
            )
        if isinstance(init, dict):
            for k, v in init.items():
                if v == "ADDRESS_PLACEHOLDER":
                    errors.append(
                        f"ready_for_sdk_consumption status cannot use ADDRESS_PLACEHOLDER for {k}"
                    )

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
        print("INVALID SDK HANDOFF ARTIFACT")
        for error in errors:
            print(f"- {error}")
        return 1
    print(f"VALID: {data['status']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
