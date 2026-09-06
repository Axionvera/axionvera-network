#!/usr/bin/env python3
"""Validator for MVP demo scenario fixtures, state transitions, and event outputs."""

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "schemas/mvp-demo-scenario.schema.json"
SCENARIO_PATH = ROOT / "examples/mvp-demo-scenario/scenario.json"
TRANSITIONS_PATH = ROOT / "examples/mvp-demo-scenario/state-transitions.json"
EVENTS_PATH = ROOT / "examples/mvp-demo-scenario/event-outputs.json"


def validate_fixtures(
    scenario_path: Path = SCENARIO_PATH,
    transitions_path: Path = TRANSITIONS_PATH,
    events_path: Path = EVENTS_PATH,
) -> list[str]:
    errors: list[str] = []

    # 1. Load scenario
    if not scenario_path.is_file():
        return [f"Scenario file not found: {scenario_path}"]
    try:
        scenario = json.loads(scenario_path.read_text(encoding="utf-8"))
    except Exception as e:
        return [f"Failed to parse scenario JSON: {e}"]

    # 2. Check basic scenario structure
    required_keys = {"schema_version", "demo_title", "network", "actors", "tokens", "steps", "maintainer_demo_boundary"}
    missing = sorted(required_keys - scenario.keys())
    if missing:
        errors.append(f"Missing required keys in scenario: {missing}")

    if scenario.get("schema_version") != "1":
        errors.append("scenario.json schema_version must be '1'")

    steps = scenario.get("steps", [])
    if len(steps) < 5:
        errors.append(f"Scenario must contain at least 5 steps, found {len(steps)}")

    # 3. Load transitions
    if not transitions_path.is_file():
        errors.append(f"Transitions file not found: {transitions_path}")
        transitions = []
    else:
        try:
            t_data = json.loads(transitions_path.read_text(encoding="utf-8"))
            transitions = t_data.get("transitions", [])
            if len(transitions) != len(steps):
                errors.append(f"Transitions count ({len(transitions)}) does not match steps count ({len(steps)})")
        except Exception as e:
            errors.append(f"Failed to parse transitions JSON: {e}")
            transitions = []

    # 4. Load events
    if not events_path.is_file():
        errors.append(f"Events file not found: {events_path}")
        events = []
    else:
        try:
            e_data = json.loads(events_path.read_text(encoding="utf-8"))
            events = e_data.get("events", [])
            if len(events) != len(steps):
                errors.append(f"Events count ({len(events)}) does not match steps count ({len(steps)})")
        except Exception as e:
            errors.append(f"Failed to parse events JSON: {e}")
            events = []

    # 5. Cross-step semantic alignment
    expected_actions = ["initialize", "deposit", "reward_setup", "claim", "withdraw"]
    for idx, (step, expected_act) in enumerate(zip(steps, expected_actions), 1):
        if step.get("action") != expected_act:
            errors.append(f"Step {idx} action mismatch: expected '{expected_act}', got '{step.get('action')}'")

    # 6. Mathematical accounting consistency
    if not errors:
        deposit_step = next((s for s in steps if s.get("action") == "deposit"), None)
        withdraw_step = next((s for s in steps if s.get("action") == "withdraw"), None)
        if deposit_step and withdraw_step:
            dep_amt = deposit_step["params"].get("amount", 0)
            with_amt = withdraw_step["params"].get("amount", 0)
            expected_rem = dep_amt - with_amt
            actual_rem = withdraw_step["expected_state"].get("user_1_balance", -1)
            if actual_rem != expected_rem:
                errors.append(f"Accounting error: deposit({dep_amt}) - withdraw({with_amt}) = {expected_rem}, but withdraw expected_state has {actual_rem}")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--scenario", type=Path, default=SCENARIO_PATH, help="Path to scenario.json")
    parser.add_argument("--transitions", type=Path, default=TRANSITIONS_PATH, help="Path to state-transitions.json")
    parser.add_argument("--events", type=Path, default=EVENTS_PATH, help="Path to event-outputs.json")
    args = parser.parse_args()

    errors = validate_fixtures(args.scenario, args.transitions, args.events)
    if errors:
        print("INVALID: MVP demo scenario validation failed:")
        for err in errors:
            print(f"  - {err}")
        return 1

    print("VALID: MVP demo scenario fixtures are consistent and valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
