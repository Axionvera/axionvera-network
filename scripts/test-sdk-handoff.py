#!/usr/bin/env python3
"""Regression tests for SDK handoff artifact package validation."""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXAMPLE = ROOT / "examples/sdk-handoff.json"
VALIDATE = ROOT / "scripts/validate-sdk-handoff.py"


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, *args], cwd=ROOT, text=True, capture_output=True, check=False
    )


def main() -> int:
    with tempfile.TemporaryDirectory() as directory:
        # Test base example validity
        valid = run(str(VALIDATE), str(EXAMPLE))
        assert valid.returncode == 0, f"Example failed validation: {valid.stderr or valid.stdout}"
        assert "VALID: placeholder_for_maintainer_deployment" in valid.stdout

        original = json.loads(EXAMPLE.read_text(encoding="utf-8"))

        # Test valid real maintainer deployment artifact
        real_deployment = {
            **original,
            "status": "ready_for_sdk_consumption",
            "contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "initialization": {
                "admin": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                "deposit_token": "CDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                "reward_token": "CRAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            },
            "maintainer_deployment_boundary": {
                "no_secrets_included": True,
                "real_deployment_performed": True,
            },
        }
        real_path = Path(directory) / "real_deployment.json"
        real_path.write_text(json.dumps(real_deployment), encoding="utf-8")
        valid_real = run(str(VALIDATE), str(real_path))
        assert valid_real.returncode == 0, f"Real artifact failed validation: {valid_real.stderr or valid_real.stdout}"
        assert "VALID: ready_for_sdk_consumption" in valid_real.stdout

        # Negative mutation tests
        mutations = {
            "missing field": {key: value for key, value in original.items() if key != "network"},
            "invalid schema_version": {**original, "schema_version": "2"},
            "invalid status": {**original, "status": "unknown_status"},
            "invalid network name": {
                **original,
                "network": {**original["network"], "name": "invalid_net"},
            },
            "invalid contract_id": {**original, "contract_id": "NOT_A_CONTRACT_ID"},
            "invalid address in initialization": {
                **original,
                "initialization": {**original["initialization"], "admin": "bad_address"},
            },
            "inclusion of secret key": {
                **original,
                "secret_key_leak": "SDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            },
            "forbidden field name": {
                **original,
                "private_key": "some_value",
            },
            "ready status with placeholders": {
                **original,
                "status": "ready_for_sdk_consumption",
                "maintainer_deployment_boundary": {
                    "no_secrets_included": True,
                    "real_deployment_performed": True,
                },
            },
            "secrets included flag false": {
                **original,
                "maintainer_deployment_boundary": {
                    "no_secrets_included": False,
                    "real_deployment_performed": False,
                },
            },
        }

        for label, mutation in mutations.items():
            candidate = Path(directory) / f"{label.replace(' ', '-')}.json"
            candidate.write_text(json.dumps(mutation), encoding="utf-8")
            result = run(str(VALIDATE), str(candidate))
            assert result.returncode != 0, f"mutation unexpectedly passed: {label}"

        print("PASS SDK handoff artifact validation and negative mutations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
