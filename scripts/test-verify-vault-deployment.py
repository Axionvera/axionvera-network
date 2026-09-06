#!/usr/bin/env python3
"""Tests for the post-deployment verification script template, input validator, and report validator."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SH_SCRIPT = ROOT / "scripts/verify-vault-deployment.sh"
PY_SCRIPT = ROOT / "scripts/verify-vault-deployment.py"
INPUT_EXAMPLE = ROOT / "examples/post-deployment-verification-input.json"
REPORT_EXAMPLE = ROOT / "examples/post-deployment-verification.json"
INPUT_SCHEMA = ROOT / "schemas/post-deployment-verification-input.schema.json"
REPORT_SCHEMA = ROOT / "schemas/post-deployment-verification.schema.json"


def run_cmd(*args: str, env: dict[str, str] = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(args),
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_shell_template() -> None:
    base_env = os.environ.copy()

    # 1. Missing env vars must fail
    res_missing = run_cmd(str(SH_SCRIPT), env=base_env)
    assert res_missing.returncode != 0, "Shell script should fail when required vars are missing"
    assert "Error: Required environment variable" in res_missing.stdout

    # 2. Valid env vars should succeed
    valid_env = base_env.copy()
    valid_env["AXIONVERA_NETWORK_NAME"] = "testnet"
    valid_env["AXIONVERA_VAULT_CONTRACT_ID"] = "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    valid_env["AXIONVERA_ADMIN_ADDRESS"] = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
    valid_env["AXIONVERA_DEPOSIT_TOKEN"] = "CBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB"
    valid_env["AXIONVERA_REWARD_TOKEN"] = "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC"

    res_valid = run_cmd(str(SH_SCRIPT), env=valid_env)
    assert res_valid.returncode == 0, f"Shell script failed: returncode={res_valid.returncode} {res_valid.stderr}"
    assert "Dry run verification completed successfully." in res_valid.stdout
    assert "stellar contract invoke" in res_valid.stdout
    assert "total_deposits" in res_valid.stdout


def test_python_validator() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        # 1. Validate canonical input example
        res_inp = run_cmd(sys.executable, str(PY_SCRIPT), str(INPUT_EXAMPLE))
        assert res_inp.returncode == 0, f"Input example failed: {res_inp.stdout} {res_inp.stderr}"
        assert "READY: post-deployment verification input valid" in res_inp.stdout

        # 2. Validate canonical report example
        res_ex = run_cmd(sys.executable, str(PY_SCRIPT), str(REPORT_EXAMPLE))
        assert res_ex.returncode == 0, f"Report example failed: {res_ex.stdout} {res_ex.stderr}"
        assert "VERIFIED: post-deployment verification checks passed" in res_ex.stdout

        # 3. Test report generation
        out_path = Path(tmpdir) / "generated.json"
        res_gen = run_cmd(sys.executable, str(PY_SCRIPT), "--generate", "--output", str(out_path))
        assert res_gen.returncode == 0, res_gen.stderr
        res_gen_val = run_cmd(sys.executable, str(PY_SCRIPT), str(out_path))
        assert res_gen_val.returncode == 0
        assert "VERIFIED: post-deployment verification checks passed" in res_gen_val.stdout

        # 4. Input negative mutations
        orig_inp = json.loads(INPUT_EXAMPLE.read_text(encoding="utf-8"))
        inp_mutations = {
            "missing field": {k: v for k, v in orig_inp.items() if k != "mode"},
            "bad schema version": {**orig_inp, "schema_version": "2"},
            "invalid network": {**orig_inp, "network": "unsupported-network"},
            "invalid contract_id": {**orig_inp, "contract_id": "NOT_A_VALID_CONTRACT_KEY"},
            "invalid admin_address": {**orig_inp, "admin_address": "bad_admin"},
            "invalid deposit_token": {**orig_inp, "deposit_token": "invalid_token_id"},
            "invalid reward_token": {**orig_inp, "reward_token": "invalid_token_id"},
            "invalid mode": {**orig_inp, "mode": "invalid_mode"},
        }
        for label, mut in inp_mutations.items():
            cand = Path(tmpdir) / f"inp-{label.replace(' ', '-')}.json"
            cand.write_text(json.dumps(mut), encoding="utf-8")
            res_mut = run_cmd(sys.executable, str(PY_SCRIPT), str(cand))
            assert res_mut.returncode != 0, f"Input mutation unexpectedly passed: {label}"

        # 5. Report negative mutations
        orig = json.loads(REPORT_EXAMPLE.read_text(encoding="utf-8"))
        mutations = {
            "missing field": {k: v for k, v in orig.items() if k != "mode"},
            "bad schema version": {**orig, "schema_version": "2"},
            "invalid network": {**orig, "network": "bad-net"},
            "invalid contract_id": {**orig, "contract_id": "NOT_A_CONTRACT"},
            "invalid mode": {**orig, "mode": "unsupported_mode"},
            "live secrets used": {
                **orig,
                "maintainer_verification_boundary": {"required": True, "live_secrets_used": True},
            },
            "malformed admin": {
                **orig,
                "verification_checks": {
                    **orig["verification_checks"],
                    "initialization_state": {
                        **orig["verification_checks"]["initialization_state"],
                        "admin_address": "bad_admin_addr",
                    },
                },
            },
            "read calls not boolean": {
                **orig,
                "verification_checks": {
                    **orig["verification_checks"],
                    "read_calls": {
                        "total_deposits_accessible": "yes",
                        "reward_balance_accessible": True,
                    },
                },
            },
        }

        for label, mut in mutations.items():
            cand = Path(tmpdir) / f"{label.replace(' ', '-')}.json"
            cand.write_text(json.dumps(mut), encoding="utf-8")
            res_mut = run_cmd(sys.executable, str(PY_SCRIPT), str(cand))
            assert res_mut.returncode != 0, f"Report mutation unexpectedly passed: {label}"


def test_schemas_validity() -> None:
    # Ensure all schemas parse cleanly
    for p in (INPUT_SCHEMA, REPORT_SCHEMA):
        assert p.is_file(), f"Schema file not found: {p}"
        data = json.loads(p.read_text(encoding="utf-8"))
        assert data.get("type") == "object"
        assert "required" in data


def main() -> int:
    test_schemas_validity()
    test_shell_template()
    test_python_validator()
    print("PASS post-deployment verification script template and report validation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
