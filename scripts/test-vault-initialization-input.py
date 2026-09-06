#!/usr/bin/env python3
"""Regression tests for vault initialization input validation."""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALIDATE = ROOT / "scripts/validate-vault-initialization-input.py"
EXAMPLE = ROOT / "examples/vault-initialization-input.json"


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, *args], cwd=ROOT, text=True, capture_output=True, check=False
    )


def main() -> int:
    with tempfile.TemporaryDirectory() as directory:
        # 1. Test canonical example
        valid = run(str(VALIDATE), str(EXAMPLE))
        assert valid.returncode == 0, f"Example validation failed: {valid.stdout} {valid.stderr}"
        assert "READY: ready_for_maintainer_initialization" in valid.stdout

        original = json.loads(EXAMPLE.read_text(encoding="utf-8"))

        # 2. Test valid real addresses
        real_stellar = {
            **original,
            "contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "admin_public_key": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "deposit_token_contract_id": "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
            "reward_token_contract_id": "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC",
        }
        real_file = Path(directory) / "real_valid.json"
        real_file.write_text(json.dumps(real_stellar), encoding="utf-8")
        real_valid = run(str(VALIDATE), str(real_file))
        assert real_valid.returncode == 0, f"Real address validation failed: {real_valid.stdout}"

        # 3. Negative mutations
        mutations = {
            "missing field": {k: v for k, v in original.items() if k != "admin_public_key"},
            "bad schema version": {**original, "schema_version": "2"},
            "invalid network": {**original, "network": "unsupported-net"},
            "malformed contract_id": {**original, "contract_id": "INVALID_ID"},
            "contract_id wrong prefix": {**original, "contract_id": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"},
            "malformed admin pubkey": {**original, "admin_public_key": "not-an-address"},
            "admin pubkey wrong prefix": {**original, "admin_public_key": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"},
            "malformed deposit token": {**original, "deposit_token_contract_id": "bad_token"},
            "malformed reward token": {**original, "reward_token_contract_id": "bad_token"},
            "claimed already executed": {
                **original,
                "maintainer_initialization_boundary": {"required": True, "executed": True},
            },
            "boundary not required": {
                **original,
                "maintainer_initialization_boundary": {"required": False, "executed": False},
            },
            "root array": [original],
        }

        for label, mutation in mutations.items():
            candidate = Path(directory) / f"{label.replace(' ', '-')}.json"
            candidate.write_text(json.dumps(mutation), encoding="utf-8")
            result = run(str(VALIDATE), str(candidate))
            assert result.returncode != 0, f"mutation unexpectedly passed: {label}"

        print("PASS vault initialization input validation and negative mutations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
