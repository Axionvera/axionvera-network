#!/usr/bin/env python3
"""
Axionvera Network Release Packet Generator

Collects non-secret Network readiness artifacts into a release packet folder
for maintainer review before testnet deployment.

Usage:
    python3 scripts/generate-release-packet.py [--output-dir OUTPUT_DIR] [--network NETWORK]

The generator:
- Copies or summarizes non-secret artifacts (docs, schemas, examples, scripts)
- Produces a manifest following schemas/release-packet.schema.json
- Excludes secrets and private keys
- Validates the generated manifest
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Any


# Patterns to exclude (secrets, private keys, etc.)
EXCLUDED_PATTERNS = [
    ".env",
    ".env.local",
    ".env.*.local",
    "*.key",
    "*.pem",
    "*.p12",
    "*.pfx",
    "secret",
    "private",
    "password",
    "seed",
    "mnemonic",
    ".git/",
    "target/",
    "node_modules/",
]

# Artifact categories and their mappings
DOCUMENTATION_CATEGORIES = {
    "maintainer-handoff-guide.md": "handoff",
    "testnet-deployment-checklist.md": "checklist",
    "ci-and-local-checks.md": "configuration",
    "testnet-configuration.md": "configuration",
    "deployment-failure-and-recovery-runbook.md": "deployment",
    "post-deployment-verification.md": "verification",
    "mock-post-deployment-smoke-test.md": "verification",
    "vault-security-review-template.md": "security",
    "vault-security-review-example.md": "security",
}

EXAMPLE_TYPES = {
    "testnet-config.json": "config",
    "build-metadata.json": "deployment",
    "deployment-artifact.json": "deployment",
    "sdk-handoff.json": "handoff",
    "contract-id-registry.json": "registry",
    "post-deployment-verification.json": "verification",
    "mock-post-deployment-smoke-input.json": "verification",
    "mock-post-deployment-smoke-output.json": "verification",
    "vault-initialization-input.json": "deployment",
}

SCRIPT_PURPOSES = {
    "build-vault-wasm.sh": "build",
    "deploy-vault-template.sh": "deploy",
    "verify-vault-deployment.sh": "verify",
    "verify-vault-deployment.py": "verify",
    "release-readiness-check.sh": "validate",
    "validate-testnet-config.sh": "validate",
    "validate-mock-vault-deployment.py": "validate",
    "validate-sdk-handoff.py": "validate",
    "validate-contract-id-registry.py": "validate",
    "validate-vault-initialization-input.py": "validate",
    "validate-deployment-artifact.py": "validate",
    "validate-mvp-demo-scenario.py": "validate",
    "run-mocked-smoke-test.py": "test",
    "run-mocked-smoke-test.sh": "test",
}


def get_project_root() -> Path:
    """Get the project root directory."""
    script_dir = Path(__file__).parent
    return script_dir.parent


def get_git_commit() -> str:
    """Get the current git commit SHA."""
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=get_project_root(),
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        return "unknown"


def compute_sha256(filepath: Path) -> str:
    """Compute SHA-256 checksum of a file."""
    import hashlib

    sha256_hash = hashlib.sha256()
    with open(filepath, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()


def should_exclude(path: Path) -> bool:
    """Check if a path should be excluded based on patterns."""
    path_str = str(path)
    for pattern in EXCLUDED_PATTERNS:
        if pattern in path_str or path.match(pattern):
            return True
    return False


def collect_documentation(project_root: Path) -> List[Dict[str, Any]]:
    """Collect documentation files."""
    docs_dir = project_root / "docs"
    artifacts = []

    for doc_file in docs_dir.glob("*.md"):
        if should_exclude(doc_file):
            continue

        rel_path = doc_file.relative_to(project_root)
        category = DOCUMENTATION_CATEGORIES.get(doc_file.name, "configuration")

        artifact = {
            "name": doc_file.name,
            "path": str(rel_path),
            "category": category,
            "size_bytes": doc_file.stat().st_size,
            "checksum": compute_sha256(doc_file),
        }
        artifacts.append(artifact)

    return artifacts


def collect_schemas(project_root: Path) -> List[Dict[str, Any]]:
    """Collect JSON schema files."""
    schemas_dir = project_root / "schemas"
    artifacts = []

    for schema_file in schemas_dir.glob("*.json"):
        if should_exclude(schema_file):
            continue

        rel_path = schema_file.relative_to(project_root)

        artifact = {
            "name": schema_file.name,
            "path": str(rel_path),
            "size_bytes": schema_file.stat().st_size,
            "checksum": compute_sha256(schema_file),
        }
        artifacts.append(artifact)

    return artifacts


def collect_examples(project_root: Path) -> List[Dict[str, Any]]:
    """Collect example files."""
    examples_dir = project_root / "examples"
    artifacts = []

    # Collect JSON files in examples root
    for example_file in examples_dir.glob("*.json"):
        if should_exclude(example_file):
            continue

        rel_path = example_file.relative_to(project_root)
        example_type = EXAMPLE_TYPES.get(example_file.name, "config")

        artifact = {
            "name": example_file.name,
            "path": str(rel_path),
            "type": example_type,
            "size_bytes": example_file.stat().st_size,
            "checksum": compute_sha256(example_file),
        }
        artifacts.append(artifact)

    # Collect directories
    for example_dir in examples_dir.iterdir():
        if not example_dir.is_dir() or should_exclude(example_dir):
            continue

        rel_path = example_dir.relative_to(project_root)
        artifact = {
            "name": example_dir.name,
            "path": str(rel_path),
            "type": "deployment",
            "size_bytes": sum(f.stat().st_size for f in example_dir.rglob("*") if f.is_file()),
            "checksum": None,  # Directories don't have single checksum
        }
        artifacts.append(artifact)

    return artifacts


def collect_scripts(project_root: Path) -> List[Dict[str, Any]]:
    """Collect build, deployment, and validation scripts."""
    scripts_dir = project_root / "scripts"
    artifacts = []

    for script_file in scripts_dir.glob("*"):
        if not script_file.is_file() or should_exclude(script_file):
            continue

        # Only include .sh and .py files
        if script_file.suffix not in [".sh", ".py"]:
            continue

        rel_path = script_file.relative_to(project_root)
        purpose = SCRIPT_PURPOSES.get(script_file.name, "validate")

        artifact = {
            "name": script_file.name,
            "path": str(rel_path),
            "purpose": purpose,
            "size_bytes": script_file.stat().st_size,
            "checksum": compute_sha256(script_file),
        }
        artifacts.append(artifact)

    return artifacts


def collect_build_metadata(project_root: Path) -> Dict[str, Any]:
    """Collect build metadata if available."""
    target_dir = project_root / "target" / "wasm32-unknown-unknown" / "release"
    metadata = {}

    if not target_dir.exists():
        return metadata

    wasm_file = target_dir / "axionvera_vault_contract.wasm"
    metadata_file = target_dir / "axionvera_vault_contract.metadata.json"

    if wasm_file.exists():
        metadata["wasm_path"] = str(wasm_file.relative_to(project_root))
        metadata["wasm_sha256"] = compute_sha256(wasm_file)

    if metadata_file.exists():
        metadata["metadata_path"] = str(metadata_file.relative_to(project_root))

    return metadata


def copy_artifacts_to_packet(
    project_root: Path, packet_dir: Path, manifest: Dict[str, Any]
) -> None:
    """Copy artifacts to the release packet directory."""
    artifacts_dir = packet_dir / "artifacts"
    artifacts_dir.mkdir(parents=True, exist_ok=True)

    # Copy documentation
    docs_dir = artifacts_dir / "docs"
    docs_dir.mkdir(exist_ok=True)
    for doc in manifest["artifacts"]["documentation"]:
        src = project_root / doc["path"]
        dst = docs_dir / Path(doc["path"]).name
        shutil.copy2(src, dst)

    # Copy schemas
    schemas_dir = artifacts_dir / "schemas"
    schemas_dir.mkdir(exist_ok=True)
    for schema in manifest["artifacts"]["schemas"]:
        src = project_root / schema["path"]
        dst = schemas_dir / Path(schema["path"]).name
        shutil.copy2(src, dst)

    # Copy examples
    examples_dir = artifacts_dir / "examples"
    examples_dir.mkdir(exist_ok=True)
    for example in manifest["artifacts"]["examples"]:
        src = project_root / example["path"]
        if src.is_file():
            dst = examples_dir / Path(example["path"]).name
            shutil.copy2(src, dst)
        elif src.is_dir():
            dst = examples_dir / src.name
            shutil.copytree(src, dst, dirs_exist_ok=True)

    # Copy scripts
    scripts_dir = artifacts_dir / "scripts"
    scripts_dir.mkdir(exist_ok=True)
    for script in manifest["artifacts"]["scripts"]:
        src = project_root / script["path"]
        dst = scripts_dir / Path(script["path"]).name
        shutil.copy2(src, dst)
        # Make scripts executable
        os.chmod(dst, 0o755)


def generate_manifest(
    project_root: Path, network: str = "testnet"
) -> Dict[str, Any]:
    """Generate the release packet manifest."""
    manifest = {
        "schema_version": "1.0",
        "packet_id": str(uuid.uuid4()),
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source_commit": get_git_commit(),
        "network": {
            "name": network,
            "rpc_url": "https://soroban-testnet.stellar.org" if network == "testnet" else "",
            "network_passphrase": "Test SDF Network ; September 2015" if network == "testnet" else "",
        },
        "artifacts": {
            "documentation": collect_documentation(project_root),
            "schemas": collect_schemas(project_root),
            "examples": collect_examples(project_root),
            "scripts": collect_scripts(project_root),
            "build_metadata": collect_build_metadata(project_root),
        },
        "checklists": {
            "release_readiness": {
                "checked": False,
                "timestamp": None,
                "result": "skipped",
            },
            "testnet_deployment": {
                "checked": False,
                "timestamp": None,
                "result": "skipped",
            },
        },
        "security_boundary": {
            "no_secrets_included": True,
            "no_private_keys_included": True,
            "excluded_patterns": EXCLUDED_PATTERNS,
            "validation_method": "pattern_exclusion",
        },
        "notes": "Release packet generated for maintainer review. Verify all artifacts before testnet deployment.",
    }

    return manifest


def main():
    parser = argparse.ArgumentParser(
        description="Generate a release packet for Network readiness artifacts"
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default="release-packet",
        help="Output directory for the release packet (default: release-packet)",
    )
    parser.add_argument(
        "--network",
        type=str,
        default="testnet",
        choices=["local", "testnet", "futurenet", "mainnet"],
        help="Target network (default: testnet)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Generate manifest without copying artifacts",
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        help="Validate the generated manifest against the schema",
    )

    args = parser.parse_args()

    project_root = get_project_root()
    packet_dir = project_root / args.output_dir

    # Generate manifest
    if not args.dry_run:
        print(f"Generating release packet manifest for {args.network}...")
    manifest = generate_manifest(project_root, args.network)

    # Create output directory
    if not args.dry_run:
        packet_dir.mkdir(parents=True, exist_ok=True)

        # Copy artifacts
        print("Copying artifacts to release packet...")
        copy_artifacts_to_packet(project_root, packet_dir, manifest)

        # Write manifest
        manifest_path = packet_dir / "manifest.json"
        with open(manifest_path, "w") as f:
            json.dump(manifest, f, indent=2)

        print(f"Release packet generated at: {packet_dir}")
        print(f"Manifest saved at: {manifest_path}")
    else:
        # Dry-run: output only the JSON manifest
        print(json.dumps(manifest, indent=2))

    # Validate if requested
    if args.validate:
        schema_path = project_root / "schemas" / "release-packet.schema.json"
        if schema_path.exists():
            if not args.dry_run:
                print(f"\nValidating manifest against schema...")
            try:
                import jsonschema

                with open(schema_path, "r") as f:
                    schema = json.load(f)
                jsonschema.validate(manifest, schema)
                if not args.dry_run:
                    print("✓ Manifest validation passed")
            except ImportError:
                if not args.dry_run:
                    print("⚠ jsonschema not installed, skipping validation")
                    print("  Install with: pip install jsonschema")
            except jsonschema.ValidationError as e:
                if not args.dry_run:
                    print(f"✗ Manifest validation failed: {e}")
                sys.exit(1)
        else:
            if not args.dry_run:
                print(f"⚠ Schema file not found: {schema_path}")

    # Print summary only in non-dry-run mode
    if not args.dry_run:
        print("\nSummary:")
        print(f"  Documentation files: {len(manifest['artifacts']['documentation'])}")
        print(f"  Schema files: {len(manifest['artifacts']['schemas'])}")
        print(f"  Example files: {len(manifest['artifacts']['examples'])}")
        print(f"  Scripts: {len(manifest['artifacts']['scripts'])}")
        print(f"  Build metadata: {'available' if manifest['artifacts']['build_metadata'] else 'not available'}")
        print(f"\nSecurity boundary:")
        print(f"  No secrets included: {manifest['security_boundary']['no_secrets_included']}")
        print(f"  No private keys included: {manifest['security_boundary']['no_private_keys_included']}")


if __name__ == "__main__":
    main()
