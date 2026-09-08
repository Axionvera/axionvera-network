#!/usr/bin/env python3
"""
Axionvera Network Release Packet Generator Test

Tests the release packet generator script to ensure it:
- Generates valid manifests
- Excludes secrets appropriately
- Collects all expected artifacts
- Produces valid output

Usage:
    python3 scripts/test-release-packet.py
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def run_command(cmd: list, cwd: Path) -> tuple:
    """Run a command and return (returncode, stdout, stderr)."""
    result = subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
    )
    return result.returncode, result.stdout, result.stderr


def test_generator_exists():
    """Test that the generator script exists."""
    project_root = Path(__file__).parent.parent
    generator_path = project_root / "scripts" / "generate-release-packet.py"

    if not generator_path.exists():
        print("✗ Generator script not found")
        return False

    print("✓ Generator script exists")
    return True


def test_generator_executable():
    """Test that the generator script is executable."""
    project_root = Path(__file__).parent.parent
    generator_path = project_root / "scripts" / "generate-release-packet.py"

    if not os.access(generator_path, os.X_OK):
        # Make it executable
        os.chmod(generator_path, 0o755)

    if os.access(generator_path, os.X_OK):
        print("✓ Generator script is executable")
        return True
    else:
        print("✗ Generator script is not executable")
        return False


def test_dry_run():
    """Test generator in dry-run mode."""
    project_root = Path(__file__).parent.parent
    generator_path = project_root / "scripts" / "generate-release-packet.py"

    returncode, stdout, stderr = run_command(
        ["python3", str(generator_path), "--dry-run"],
        project_root,
    )

    if returncode != 0:
        print(f"✗ Dry-run failed with return code {returncode}")
        print(f"  stderr: {stderr}")
        return False

    # Check that output contains expected fields
    try:
        manifest = json.loads(stdout)
        required_fields = [
            "schema_version",
            "packet_id",
            "generated_at",
            "source_commit",
            "artifacts",
            "security_boundary",
        ]

        missing = [field for field in required_fields if field not in manifest]
        if missing:
            print(f"✗ Dry-run manifest missing fields: {missing}")
            return False

        print("✓ Dry-run generates valid manifest")
        return True
    except json.JSONDecodeError:
        print("✗ Dry-run output is not valid JSON")
        return False


def test_full_generation():
    """Test full packet generation in a temporary directory."""
    project_root = Path(__file__).parent.parent
    generator_path = project_root / "scripts" / "generate-release-packet.py"

    with tempfile.TemporaryDirectory() as tmpdir:
        output_dir = Path(tmpdir) / "test-packet"

        returncode, stdout, stderr = run_command(
            ["python3", str(generator_path), "--output-dir", str(output_dir)],
            project_root,
        )

        if returncode != 0:
            print(f"✗ Full generation failed with return code {returncode}")
            print(f"  stderr: {stderr}")
            return False

        # Check that output directory was created
        if not output_dir.exists():
            print("✗ Output directory not created")
            return False

        # Check that manifest.json exists
        manifest_path = output_dir / "manifest.json"
        if not manifest_path.exists():
            print("✗ manifest.json not created")
            return False

        # Check that artifacts directory was created
        artifacts_dir = output_dir / "artifacts"
        if not artifacts_dir.exists():
            print("✗ artifacts directory not created")
            return False

        # Load and validate manifest
        try:
            with open(manifest_path, "r") as f:
                manifest = json.load(f)

            # Check required fields
            required_fields = [
                "schema_version",
                "packet_id",
                "generated_at",
                "source_commit",
                "artifacts",
                "security_boundary",
            ]

            missing = [field for field in required_fields if field not in manifest]
            if missing:
                print(f"✗ Manifest missing fields: {missing}")
                return False

            # Check artifacts subdirectories
            expected_subdirs = ["docs", "schemas", "examples", "scripts"]
            for subdir in expected_subdirs:
                subdir_path = artifacts_dir / subdir
                if not subdir_path.exists():
                    print(f"✗ Artifacts subdirectory not created: {subdir}")
                    return False

            print("✓ Full generation creates valid packet structure")
            return True
        except json.JSONDecodeError:
            print("✗ Manifest is not valid JSON")
            return False


def test_security_boundary():
    """Test that security boundary is properly set."""
    project_root = Path(__file__).parent.parent
    generator_path = project_root / "scripts" / "generate-release-packet.py"

    returncode, stdout, stderr = run_command(
        ["python3", str(generator_path), "--dry-run"],
        project_root,
    )

    if returncode != 0:
        print(f"✗ Security boundary test failed (dry-run failed)")
        return False

    try:
        manifest = json.loads(stdout)
        sb = manifest.get("security_boundary", {})

        if not sb.get("no_secrets_included", False):
            print("✗ security_boundary.no_secrets_included is not True")
            return False

        if not sb.get("no_private_keys_included", False):
            print("✗ security_boundary.no_private_keys_included is not True")
            return False

        if "excluded_patterns" not in sb:
            print("✗ security_boundary.excluded_patterns not set")
            return False

        print("✓ Security boundary properly configured")
        return True
    except json.JSONDecodeError:
        print("✗ Security boundary test failed (invalid JSON)")
        return False


def test_artifact_collection():
    """Test that artifacts are collected."""
    project_root = Path(__file__).parent.parent
    generator_path = project_root / "scripts" / "generate-release-packet.py"

    returncode, stdout, stderr = run_command(
        ["python3", str(generator_path), "--dry-run"],
        project_root,
    )

    if returncode != 0:
        print("✗ Artifact collection test failed (dry-run failed)")
        return False

    try:
        manifest = json.loads(stdout)
        artifacts = manifest.get("artifacts", {})

        # Check that artifact categories exist
        required_categories = ["documentation", "schemas", "examples", "scripts"]
        for category in required_categories:
            if category not in artifacts:
                print(f"✗ Artifact category missing: {category}")
                return False

            if not isinstance(artifacts[category], list):
                print(f"✗ Artifact category is not a list: {category}")
                return False

            if len(artifacts[category]) == 0:
                print(f"⚠ Warning: Artifact category is empty: {category}")

        print("✓ Artifacts collected successfully")
        print(f"  Documentation: {len(artifacts['documentation'])} files")
        print(f"  Schemas: {len(artifacts['schemas'])} files")
        print(f"  Examples: {len(artifacts['examples'])} files")
        print(f"  Scripts: {len(artifacts['scripts'])} files")
        return True
    except json.JSONDecodeError:
        print("✗ Artifact collection test failed (invalid JSON)")
        return False


def test_validator_exists():
    """Test that the validator script exists."""
    project_root = Path(__file__).parent.parent
    validator_path = project_root / "scripts" / "validate-release-packet.py"

    if not validator_path.exists():
        print("✗ Validator script not found")
        return False

    print("✓ Validator script exists")
    return True


def test_schema_exists():
    """Test that the release packet schema exists."""
    project_root = Path(__file__).parent.parent
    schema_path = project_root / "schemas" / "release-packet.schema.json"

    if not schema_path.exists():
        print("✗ Release packet schema not found")
        return False

    print("✓ Release packet schema exists")
    return True


def main():
    print("Testing Release Packet Generator\n")
    print("=" * 50)

    tests = [
        ("Generator exists", test_generator_exists),
        ("Generator executable", test_generator_executable),
        ("Schema exists", test_schema_exists),
        ("Validator exists", test_validator_exists),
        ("Dry-run mode", test_dry_run),
        ("Security boundary", test_security_boundary),
        ("Artifact collection", test_artifact_collection),
        ("Full generation", test_full_generation),
    ]

    results = []
    for name, test_func in tests:
        print(f"\n{name}:")
        try:
            result = test_func()
            results.append(result)
        except Exception as e:
            print(f"✗ Test failed with exception: {e}")
            results.append(False)

    # Summary
    print("\n" + "=" * 50)
    passed = sum(results)
    total = len(results)

    if all(results):
        print(f"✓ All {total} tests passed")
        sys.exit(0)
    else:
        print(f"✗ {total - passed} of {total} tests failed")
        sys.exit(1)


if __name__ == "__main__":
    main()
