#!/usr/bin/env python3
"""
Axionvera Network Release Packet Validator

Validates a release packet manifest against the schema and verifies
that no secrets or private keys are included.

Usage:
    python3 scripts/validate-release-packet.py <manifest_path>
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Dict, Any


# Secret patterns to detect
SECRET_PATTERNS = [
    "private_key",
    "privatekey",
    "secret_key",
    "secretkey",
    "seed_phrase",
    "seedphrase",
    "mnemonic",
    "password",
    "api_key",
    "apikey",
    "access_token",
    "accesstoken",
    "skey",
    "s_key",
    "sk_",
    "pk_",
    "pkey",
]


def load_manifest(manifest_path: Path) -> Dict[str, Any]:
    """Load and parse the manifest JSON file."""
    try:
        with open(manifest_path, "r") as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"Error: Manifest file not found: {manifest_path}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in manifest file: {e}")
        sys.exit(1)


def load_schema(project_root: Path) -> Dict[str, Any]:
    """Load the release packet schema."""
    schema_path = project_root / "schemas" / "release-packet.schema.json"
    try:
        with open(schema_path, "r") as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"Error: Schema file not found: {schema_path}")
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in schema file: {e}")
        sys.exit(1)


def validate_schema(manifest: Dict[str, Any], schema: Dict[str, Any]) -> bool:
    """Validate manifest against the JSON schema."""
    try:
        import jsonschema
        jsonschema.validate(manifest, schema)
        print("✓ Schema validation passed")
        return True
    except ImportError:
        print("⚠ jsonschema not installed, skipping schema validation")
        print("  Install with: pip install jsonschema")
        return True  # Continue without schema validation
    except jsonschema.ValidationError as e:
        print(f"✗ Schema validation failed: {e.message}")
        print(f"  Path: {' -> '.join(str(p) for p in e.path)}")
        return False


def check_for_secrets(manifest: Dict[str, Any]) -> bool:
    """Check manifest for potential secrets or private keys."""
    found_secrets = []

    def check_value(value: Any, path: str = ""):
        """Recursively check values for secret patterns."""
        if isinstance(value, dict):
            for key, val in value.items():
                check_value(val, f"{path}.{key}" if path else key)
        elif isinstance(value, list):
            for i, item in enumerate(value):
                check_value(item, f"{path}[{i}]")
        elif isinstance(value, str):
            value_lower = value.lower()
            for pattern in SECRET_PATTERNS:
                if pattern in value_lower:
                    found_secrets.append((path, pattern, value[:50]))

    check_value(manifest)

    if found_secrets:
        print("✗ Potential secrets detected in manifest:")
        for path, pattern, value in found_secrets:
            print(f"  - Path: {path}")
            print(f"    Pattern: {pattern}")
            print(f"    Value: {value}...")
        return False
    else:
        print("✓ No secrets detected in manifest")
        return True


def validate_security_boundary(manifest: Dict[str, Any]) -> bool:
    """Validate the security boundary section."""
    if "security_boundary" not in manifest:
        print("✗ Missing security_boundary section")
        return False

    sb = manifest["security_boundary"]

    if not sb.get("no_secrets_included", False):
        print("✗ security_boundary.no_secrets_included is not True")
        return False

    if not sb.get("no_private_keys_included", False):
        print("✗ security_boundary.no_private_keys_included is not True")
        return False

    print("✓ Security boundary validation passed")
    return True


def validate_required_fields(manifest: Dict[str, Any]) -> bool:
    """Validate that all required fields are present."""
    required_fields = [
        "schema_version",
        "packet_id",
        "generated_at",
        "source_commit",
        "artifacts",
        "security_boundary",
    ]

    missing_fields = [field for field in required_fields if field not in manifest]

    if missing_fields:
        print(f"✗ Missing required fields: {', '.join(missing_fields)}")
        return False

    print("✓ Required fields validation passed")
    return True


def validate_artifacts(manifest: Dict[str, Any]) -> bool:
    """Validate the artifacts section."""
    if "artifacts" not in manifest:
        print("✗ Missing artifacts section")
        return False

    artifacts = manifest["artifacts"]
    required_artifact_types = ["documentation", "schemas", "examples", "scripts"]

    for artifact_type in required_artifact_types:
        if artifact_type not in artifacts:
            print(f"✗ Missing artifact type: {artifact_type}")
            return False

        if not isinstance(artifacts[artifact_type], list):
            print(f"✗ {artifact_type} is not a list")
            return False

        if len(artifacts[artifact_type]) == 0:
            print(f"⚠ Warning: {artifact_type} is empty")

    print("✓ Artifacts validation passed")
    return True


def validate_checksums(manifest: Dict[str, Any]) -> bool:
    """Validate that checksums are present where expected."""
    artifacts = manifest.get("artifacts", {})
    issues = []

    for doc in artifacts.get("documentation", []):
        if "checksum" not in doc or not doc["checksum"]:
            issues.append(f"Missing checksum for documentation: {doc.get('name', 'unknown')}")

    for schema in artifacts.get("schemas", []):
        if "checksum" not in schema or not schema["checksum"]:
            issues.append(f"Missing checksum for schema: {schema.get('name', 'unknown')}")

    for script in artifacts.get("scripts", []):
        if "checksum" not in script or not script["checksum"]:
            issues.append(f"Missing checksum for script: {script.get('name', 'unknown')}")

    if issues:
        print("✗ Checksum validation issues:")
        for issue in issues:
            print(f"  - {issue}")
        return False

    print("✓ Checksum validation passed")
    return True


def main():
    parser = argparse.ArgumentParser(
        description="Validate a release packet manifest"
    )
    parser.add_argument(
        "manifest_path",
        type=str,
        help="Path to the release packet manifest JSON file",
    )
    parser.add_argument(
        "--schema-path",
        type=str,
        default=None,
        help="Path to the schema file (default: schemas/release-packet.schema.json)",
    )

    args = parser.parse_args()

    manifest_path = Path(args.manifest_path)
    if not manifest_path.exists():
        print(f"Error: Manifest file not found: {manifest_path}")
        sys.exit(1)

    # Load manifest
    print(f"Loading manifest from: {manifest_path}")
    manifest = load_manifest(manifest_path)

    # Determine project root
    if args.schema_path:
        project_root = Path(args.schema_path).parent.parent
    else:
        project_root = manifest_path.parent.parent

    # Load schema
    schema = load_schema(project_root)

    # Run validations
    print("\nRunning validations...\n")

    validations = [
        ("Required fields", lambda: validate_required_fields(manifest)),
        ("Schema", lambda: validate_schema(manifest, schema)),
        ("Security boundary", lambda: validate_security_boundary(manifest)),
        ("Artifacts", lambda: validate_artifacts(manifest)),
        ("Checksums", lambda: validate_checksums(manifest)),
        ("Secret detection", lambda: check_for_secrets(manifest)),
    ]

    results = []
    for name, validation_func in validations:
        print(f"\n{name}:")
        results.append(validation_func())

    # Summary
    print("\n" + "=" * 50)
    if all(results):
        print("✓ All validations passed")
        print(f"\nPacket ID: {manifest.get('packet_id', 'unknown')}")
        print(f"Generated at: {manifest.get('generated_at', 'unknown')}")
        print(f"Source commit: {manifest.get('source_commit', 'unknown')}")
        print(f"Network: {manifest.get('network', {}).get('name', 'unknown')}")
        sys.exit(0)
    else:
        print("✗ Some validations failed")
        failed = [name for (name, _) in zip([v[0] for v in validations], results) if not results[list([v[0] for v in validations]).index(name)]]
        print(f"Failed validations: {', '.join(failed)}")
        sys.exit(1)


if __name__ == "__main__":
    main()
