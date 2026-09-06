#!/usr/bin/env python3
"""Tests for MVP demo scenario fixtures, schema, and validator."""

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts/validate-mvp-demo-scenario.py"
SCHEMA = ROOT / "schemas/mvp-demo-scenario.schema.json"
SCENARIO = ROOT / "examples/mvp-demo-scenario/scenario.json"
TRANSITIONS = ROOT / "examples/mvp-demo-scenario/state-transitions.json"
EVENTS = ROOT / "examples/mvp-demo-scenario/event-outputs.json"


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
    assert SCHEMA.is_file(), f"Schema not found: {SCHEMA}"
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    assert schema.get("title") == "Axionvera MVP Demo Scenario Schema"
    assert "steps" in schema["properties"]


def test_validator_default() -> None:
    res = run_cmd(sys.executable, str(VALIDATOR))
    assert res.returncode == 0, f"Default validator failed: {res.stdout} {res.stderr}"
    assert "VALID: MVP demo scenario fixtures are consistent and valid" in res.stdout


def test_validator_mutations() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        orig_scen = json.loads(SCENARIO.read_text(encoding="utf-8"))

        # 1. Missing step
        mut_scen1 = {**orig_scen, "steps": orig_scen["steps"][:4]}
        p1 = Path(tmpdir) / "scen-short.json"
        p1.write_text(json.dumps(mut_scen1), encoding="utf-8")
        res1 = run_cmd(sys.executable, str(VALIDATOR), "--scenario", str(p1))
        assert res1.returncode != 0
        assert "Scenario must contain at least 5 steps" in res1.stdout

        # 2. Accounting mismatch
        mut_scen2 = json.loads(SCENARIO.read_text(encoding="utf-8"))
        mut_scen2["steps"][4]["expected_state"]["user_1_balance"] = 9999
        p2 = Path(tmpdir) / "scen-math.json"
        p2.write_text(json.dumps(mut_scen2), encoding="utf-8")
        res2 = run_cmd(sys.executable, str(VALIDATOR), "--scenario", str(p2))
        assert res2.returncode != 0
        assert "Accounting error" in res2.stdout

        # 3. Step action mismatch
        mut_scen3 = json.loads(SCENARIO.read_text(encoding="utf-8"))
        mut_scen3["steps"][1]["action"] = "unknown_action"
        p3 = Path(tmpdir) / "scen-action.json"
        p3.write_text(json.dumps(mut_scen3), encoding="utf-8")
        res3 = run_cmd(sys.executable, str(VALIDATOR), "--scenario", str(p3))
        assert res3.returncode != 0
        assert "action mismatch" in res3.stdout


def main() -> int:
    test_schema_validity()
    test_validator_default()
    test_validator_mutations()
    print("PASS MVP demo scenario fixtures validation")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
