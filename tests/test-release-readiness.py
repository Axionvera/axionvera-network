#!/usr/bin/env python3
"""
Practical dry-run test for the release readiness checklist script.
Runs scripts/release-readiness-check.sh and verifies clean output
with no missing or failed items when the repository is intact.
No secrets or privileged actions are used.
"""

import os
import subprocess
import sys

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SCRIPT = os.path.join(PROJECT_ROOT, "scripts", "release-readiness-check.sh")


def main() -> int:
    if not os.path.isfile(SCRIPT):
        print(f"FAIL: script not found at {SCRIPT}")
        return 1

    print("Running release readiness checklist (quick mode)...")
    result = subprocess.run(
        ["bash", SCRIPT],
        capture_output=True,
        text=True,
        cwd=PROJECT_ROOT,
    )

    print("Exit code:", result.returncode)
    print("--- stdout (first 30 lines) ---")
    for line in result.stdout.splitlines()[:30]:
        print(line)
    if len(result.stdout.splitlines()) > 30:
        print("... (truncated)")

    if result.returncode != 0:
        print("FAIL: script exited with non-zero code")
        if result.stderr:
            print("stderr:", result.stderr[:500])
        return 1

    if "[MISSING]" in result.stdout:
        print("FAIL: missing items reported")
        return 1

    if "[FAIL]" in result.stdout:
        print("FAIL: command failures reported")
        return 1

    if "RELEASE READINESS: ALL CHECKS PASSED" not in result.stdout:
        print("FAIL: expected success message not found")
        return 1

    print("PASS: release readiness check completed cleanly")
    return 0


if __name__ == "__main__":
    sys.exit(main())
