#!/usr/bin/env python3
"""Regression tests for the contributor-safe mocked deployment flow."""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CREATE = ROOT / "scripts/create-mock-vault-deployment.py"
VALIDATE = ROOT / "scripts/validate-mock-vault-deployment.py"


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, *args], cwd=ROOT, text=True, capture_output=True, check=False
    )


def main() -> int:
    with tempfile.TemporaryDirectory() as directory:
        artifact = Path(directory) / "artifact.json"
        created = run(str(CREATE), "--output", str(artifact))
        assert created.returncode == 0, created.stderr
        valid = run(str(VALIDATE), str(artifact))
        assert valid.returncode == 0, valid.stdout
        assert "READY: ready_for_maintainer_deployment" in valid.stdout

        original = json.loads(artifact.read_text(encoding="utf-8"))
        mutations = {
            "missing field": {key: value for key, value in original.items() if key != "wasm"},
            "malformed initialization": {
                **original,
                "initialization": {**original["initialization"], "admin": "not-an-address"},
            },
            "real deployment claim": {
                **original,
                "maintainer_deployment_boundary": {"required": True, "real_deployment_performed": True},
            },
            "non-canonical path": {
                **original,
                "wasm": {**original["wasm"], "path": "contracts/vault-contract/target/output.wasm"},
            },
        }
        for label, mutation in mutations.items():
            candidate = Path(directory) / f"{label.replace(' ', '-')}.json"
            candidate.write_text(json.dumps(mutation), encoding="utf-8")
            result = run(str(VALIDATE), str(candidate))
            assert result.returncode != 0, f"mutation unexpectedly passed: {label}"
        print("PASS mocked deployment artifact validation and negative mutations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
