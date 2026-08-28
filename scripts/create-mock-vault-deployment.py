#!/usr/bin/env python3
"""Create a contributor-safe mocked vault deployment handoff."""

import argparse
import json
import re
from pathlib import Path

CANONICAL_WASM_PATH = "target/wasm32-unknown-unknown/release/axionvera_vault_contract.wasm"
SHA256_RE = re.compile(r"^[0-9a-fA-F]{64}$")
COMMIT_RE = re.compile(r"^[0-9a-fA-F]{40}$")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--source-commit", default="UNCOMMITTED")
    parser.add_argument("--sha256", default="SHA256_PLACEHOLDER")
    args = parser.parse_args()

    if args.source_commit != "UNCOMMITTED" and not COMMIT_RE.fullmatch(args.source_commit):
        parser.error("--source-commit must be a 40-character commit or UNCOMMITTED")
    if args.sha256 != "SHA256_PLACEHOLDER" and not SHA256_RE.fullmatch(args.sha256):
        parser.error("--sha256 must be 64 hexadecimal characters or SHA256_PLACEHOLDER")

    artifact = {
        "schema_version": "1",
        "mocked": True,
        "status": "ready_for_maintainer_deployment",
        "source_commit": args.source_commit,
        "network_mode": "testnet",
        "wasm": {"path": CANONICAL_WASM_PATH, "sha256": args.sha256},
        "contract_id": "CONTRACT_ID_PLACEHOLDER",
        "initialization": {
            "admin": "ADDRESS_PLACEHOLDER",
            "deposit_token": "ADDRESS_PLACEHOLDER",
            "reward_token": "ADDRESS_PLACEHOLDER",
        },
        "maintainer_deployment_boundary": {
            "required": True,
            "real_deployment_performed": False,
        },
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(artifact, indent=2) + "\n", encoding="utf-8")
    print(f"created mocked deployment artifact: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
