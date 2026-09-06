#!/usr/bin/env python3
"""Regression tests for mocked post-deployment smoke test flow runner and schemas."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
RUNNER_PY = ROOT / "scripts/run-mocked-smoke-test.py"
RUNNER_SH = ROOT / "scripts/run-mocked-smoke-test.sh"
SCHEMA_FILE = ROOT / "schemas/mock-post-deployment-smoke.schema.json"
INPUT_EXAMPLE = ROOT / "examples/mock-post-deployment-smoke-input.json"
OUTPUT_EXAMPLE = ROOT / "examples/mock-post-deployment-smoke-output.json"


def run_cmd(*args: str, env: dict[str, str] = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(args),
        cwd=ROOT,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


def test_schema_validity() -> None:
    assert SCHEMA_FILE.is_file(), f"Schema not found: {SCHEMA_FILE}"
    schema = json.loads(SCHEMA_FILE.read_text(encoding="utf-8"))
    assert schema.get("title") == "Axionvera Mocked Post-Deployment Smoke Test Report"
    assert "required" in schema
    assert "lifecycle_stages" in schema["properties"]


def test_python_runner() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        # 1. Default run
        res = run_cmd(sys.executable, str(RUNNER_PY))
        assert res.returncode == 0, f"Runner failed: {res.stdout} {res.stderr}"
        assert "PASSED: mocked post-deployment smoke test flow completed successfully" in res.stdout

        # 2. JSON output run
        out_json_path = Path(tmpdir) / "report.json"
        res_json = run_cmd(sys.executable, str(RUNNER_PY), "--json", "--output", str(out_json_path))
        assert res_json.returncode == 0, f"JSON runner failed: {res_json.stderr}"
        assert out_json_path.is_file()
        report = json.loads(out_json_path.read_text(encoding="utf-8"))
        assert report["overall_status"] == "PASSED"
        assert len(report["lifecycle_stages"]) == 5
        assert report["maintainer_smoke_boundary"]["no_live_rpc_used"] is True

        # 3. Running with explicit input file
        res_inp = run_cmd(sys.executable, str(RUNNER_PY), "--input", str(INPUT_EXAMPLE))
        assert res_inp.returncode == 0, f"Input run failed: {res_inp.stderr}"

        # 4. Negative mutations on input
        orig_inp = json.loads(INPUT_EXAMPLE.read_text(encoding="utf-8"))
        mutations = {
            "bad-schema": {**orig_inp, "schema_version": "2"},
            "bad-net": {**orig_inp, "network": "invalid-net"},
            "bad-cid": {**orig_inp, "contract_id_source": "nonexistent.json", "fallback_contract_id": "INVALID_CONTRACT_FORMAT"},
        }
        for label, mut in mutations.items():
            mut_file = Path(tmpdir) / f"{label}.json"
            mut_file.write_text(json.dumps(mut), encoding="utf-8")
            res_mut = run_cmd(sys.executable, str(RUNNER_PY), "--input", str(mut_file))
            assert res_mut.returncode != 0, f"Mutation {label} should have failed"
            assert "FAILED:" in res_mut.stdout

        # 5. Missing init input file
        res_missing = run_cmd(sys.executable, str(RUNNER_PY), "--init-input", str(Path(tmpdir) / "nonexistent.json"))
        assert res_missing.returncode != 0
        assert "FAILED:" in res_missing.stdout


def test_shell_runner() -> None:
    res = run_cmd(str(RUNNER_SH))
    assert res.returncode == 0, f"Shell runner failed: {res.stdout} {res.stderr}"
    assert "PASSED: mocked post-deployment smoke test flow completed successfully" in res.stdout


def main() -> int:
    test_schema_validity()
    test_python_runner()
    test_shell_runner()
    print("PASS mocked post-deployment smoke test flow validation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
