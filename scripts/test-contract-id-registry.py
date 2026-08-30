#!/usr/bin/env python3
"""Regression tests for Contract ID Registry validation."""

import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXAMPLE = ROOT / "examples/contract-id-registry.json"
VALIDATE = ROOT / "scripts/validate-contract-id-registry.py"


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, *args], cwd=ROOT, text=True, capture_output=True, check=False
    )


def main() -> int:
    with tempfile.TemporaryDirectory() as directory:
        # Test placeholder example validity
        valid = run(str(VALIDATE), str(EXAMPLE))
        assert valid.returncode == 0, f"Example failed validation: {valid.stderr or valid.stdout}"
        assert "VALID: contract ID registry format is correct" in valid.stdout

        original = json.loads(EXAMPLE.read_text(encoding="utf-8"))

        # Test valid real maintainer deployment registry
        real_registry = {
            "schema_version": "1",
            "updated_at": "2026-08-30T18:00:00Z",
            "environments": {
                "testnet": {
                    "network": "testnet",
                    "status": "deployed",
                    "contracts": {
                        "axionvera_vault_contract": {
                            "contract_id": "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                            "wasm_sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                            "deployed_at": "2026-08-30T18:00:00Z",
                            "deployer_address": "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                        }
                    },
                }
            },
            "maintainer_deployment_boundary": {
                "no_secrets_included": True,
                "real_deployments_recorded": True,
            },
        }
        real_path = Path(directory) / "real_registry.json"
        real_path.write_text(json.dumps(real_registry), encoding="utf-8")
        valid_real = run(str(VALIDATE), str(real_path))
        assert valid_real.returncode == 0, f"Real registry failed validation: {valid_real.stderr or valid_real.stdout}"

        # Negative mutation tests
        mutations = {
            "missing field": {key: value for key, value in original.items() if key != "environments"},
            "invalid schema_version": {**original, "schema_version": "2"},
            "invalid timestamp format": {**original, "updated_at": "not-a-timestamp"},
            "invalid network name": {
                **original,
                "environments": {
                    "testnet": {**original["environments"]["testnet"], "network": "invalid_net"}
                },
            },
            "invalid contract_id": {
                **original,
                "environments": {
                    "testnet": {
                        "network": "testnet",
                        "status": "placeholder",
                        "contracts": {
                            "axionvera_vault_contract": {
                                **original["environments"]["testnet"]["contracts"]["axionvera_vault_contract"],
                                "contract_id": "INVALID_ID",
                            }
                        },
                    }
                },
            },
            "invalid address format": {
                **original,
                "environments": {
                    "testnet": {
                        "network": "testnet",
                        "status": "placeholder",
                        "contracts": {
                            "axionvera_vault_contract": {
                                **original["environments"]["testnet"]["contracts"]["axionvera_vault_contract"],
                                "deployer_address": "bad_address",
                            }
                        },
                    }
                },
            },
            "inclusion of secret key": {
                **original,
                "secret_key_leak": "SDAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            },
            "forbidden field name": {
                **original,
                "private_key": "some_value",
            },
            "deployed status with placeholders": {
                **original,
                "environments": {
                    "testnet": {
                        **original["environments"]["testnet"],
                        "status": "deployed",
                    }
                },
                "maintainer_deployment_boundary": {
                    "no_secrets_included": True,
                    "real_deployments_recorded": True,
                },
            },
            "secrets included flag false": {
                **original,
                "maintainer_deployment_boundary": {
                    "no_secrets_included": False,
                    "real_deployments_recorded": False,
                },
            },
        }

        for label, mutation in mutations.items():
            candidate = Path(directory) / f"{label.replace(' ', '-')}.json"
            candidate.write_text(json.dumps(mutation), encoding="utf-8")
            result = run(str(VALIDATE), str(candidate))
            assert result.returncode != 0, f"mutation unexpectedly passed: {label}"

        print("PASS contract ID registry validation and negative mutations")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
