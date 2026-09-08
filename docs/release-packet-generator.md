# Release Packet Generator

The release packet generator collects non-secret Network readiness artifacts into a structured folder for maintainer review before testnet deployment. This ensures maintainers have all necessary documentation, schemas, examples, and scripts in one place for comprehensive review.

## Overview

The release packet generator:
- Collects documentation files (deployment guides, checklists, security reviews)
- Copies JSON schemas for validation
- Includes example configurations and data files
- Bundles build, deployment, and validation scripts
- Generates a manifest following `schemas/release-packet.schema.json`
- Excludes secrets and private keys using pattern-based filtering
- Provides SHA-256 checksums for all copied files

## Usage

### Generate a Release Packet

To generate a release packet for testnet:

```bash
python3 scripts/generate-release-packet.py
```

This creates a `release-packet/` directory in the project root containing:
- `manifest.json` - The release packet manifest
- `artifacts/docs/` - Documentation files
- `artifacts/schemas/` - JSON schema files
- `artifacts/examples/` - Example configurations
- `artifacts/scripts/` - Build and validation scripts

### Custom Output Directory

To specify a custom output directory:

```bash
python3 scripts/generate-release-packet.py --output-dir my-release-packet
```

### Target Network

To generate a packet for a specific network:

```bash
python3 scripts/generate-release-packet.py --network testnet
python3 scripts/generate-release-packet.py --network futurenet
python3 scripts/generate-release-packet.py --network mainnet
```

### Dry-Run Mode

To preview the manifest without copying artifacts:

```bash
python3 scripts/generate-release-packet.py --dry-run
```

This outputs the manifest JSON to stdout for review.

### Validate on Generation

To validate the generated manifest against the schema:

```bash
python3 scripts/generate-release-packet.py --validate
```

This requires the `jsonschema` Python package:
```bash
pip install jsonschema
```

## Manifest Structure

The generated manifest follows the schema defined in `schemas/release-packet.schema.json`. Key sections include:

### Metadata
- `schema_version` - Schema version (format: MAJOR.MINOR)
- `packet_id` - Unique UUID for this packet
- `generated_at` - ISO 8601 timestamp
- `source_commit` - Full Git commit SHA

### Network Information
- `network.name` - Target network (local, testnet, futurenet, mainnet)
- `network.rpc_url` - Public RPC endpoint
- `network.network_passphrase` - Network passphrase

### Artifacts
- `documentation` - Documentation files with categories (deployment, configuration, security, handoff, verification, checklist)
- `schemas` - JSON schema files
- `examples` - Example configurations and data files
- `scripts` - Build, deployment, and validation scripts
- `build_metadata` - WASM build metadata if available

### Checklists
- `release_readiness` - Status of release readiness check
- `testnet_deployment` - Status of testnet deployment checklist

### Security Boundary
- `no_secrets_included` - Verified that no secrets are included
- `no_private_keys_included` - Verified that no private keys are included
- `excluded_patterns` - Patterns that were excluded during generation
- `validation_method` - Method used to verify no secrets

## Security

The generator explicitly excludes files matching these patterns:
- `.env`, `.env.local`, `.env.*.local`
- `*.key`, `*.pem`, `*.p12`, `*.pfx`
- Files containing: `secret`, `private`, `password`, `seed`, `mnemonic`
- `.git/`, `target/`, `node_modules/`

The security boundary section in the manifest confirms that no secrets or private keys were included.

## Validation

To validate an existing release packet manifest:

```bash
python3 scripts/validate-release-packet.py release-packet/manifest.json
```

The validator checks:
- Schema compliance
- Required fields presence
- Security boundary configuration
- Artifact structure
- Checksum presence
- Secret pattern detection

## Testing

To test the release packet generator:

```bash
python3 scripts/test-release-packet.py
```

This runs comprehensive tests:
- Generator script existence and executability
- Schema file existence
- Dry-run mode
- Security boundary configuration
- Artifact collection
- Full packet generation

## Maintainer Review Workflow

1. **Generate Release Packet**
   ```bash
   python3 scripts/generate-release-packet.py --output-dir release-packet-<version>
   ```

2. **Review Documentation**
   - Check `artifacts/docs/` for deployment guides, checklists, and security reviews
   - Verify all required documentation is present
   - Review security review templates and examples

3. **Validate Schemas**
   - Check `artifacts/schemas/` for all required JSON schemas
   - Verify schema versions are consistent
   - Ensure schemas match current contract interface

4. **Review Examples**
   - Check `artifacts/examples/` for configuration examples
   - Verify example data matches expected formats
   - Ensure deployment examples are current

5. **Verify Scripts**
   - Check `artifacts/scripts/` for build and validation scripts
   - Verify scripts are executable
   - Ensure script purposes are correctly categorized

6. **Validate Manifest**
   ```bash
   python3 scripts/validate-release-packet.py release-packet-<version>/manifest.json
   ```

7. **Run Release Readiness Check**
   ```bash
   ./scripts/release-readiness-check.sh --full
   ```

8. **Proceed with Deployment**
   Once all artifacts are reviewed and validated, proceed with the testnet deployment checklist.

## Example Output

A generated release packet contains:

```
release-packet/
├── manifest.json
└── artifacts/
    ├── docs/
    │   ├── maintainer-handoff-guide.md
    │   ├── testnet-deployment-checklist.md
    │   ├── deployment-failure-and-recovery-runbook.md
    │   └── ...
    ├── schemas/
    │   ├── release-packet.schema.json
    │   ├── vault-interface.schema.json
    │   ├── sdk-handoff.schema.json
    │   └── ...
    ├── examples/
    │   ├── testnet-config.json
    │   ├── sdk-handoff.json
    │   ├── contract-id-registry.json
    │   └── ...
    └── scripts/
        ├── build-vault-wasm.sh
        ├── deploy-vault-template.sh
        ├── validate-release-packet.py
        └── ...
```

## Integration with CI/CD

The release packet generator can be integrated into CI/CD pipelines:

```yaml
- name: Generate Release Packet
  run: python3 scripts/generate-release-packet.py --output-dir release-packet-${{ github.sha }}

- name: Validate Release Packet
  run: python3 scripts/validate-release-packet.py release-packet-${{ github.sha }}/manifest.json

- name: Upload Release Packet
  uses: actions/upload-artifact@v3
  with:
    name: release-packet-${{ github.sha }}
    path: release-packet-${{ github.sha }}
```

## Troubleshooting

### Generator Fails to Find Files
Ensure you're running the script from the project root directory:
```bash
cd /path/to/axionvera-network
python3 scripts/generate-release-packet.py
```

### Validation Fails with Schema Error
Install the jsonschema package:
```bash
pip install jsonschema
```

### Missing Artifacts
Check that the required directories exist:
- `docs/` - Documentation files
- `schemas/` - JSON schema files
- `examples/` - Example configurations
- `scripts/` - Build and validation scripts

### Checksum Mismatch
If checksums don't match, ensure files haven't been modified after packet generation. Regenerate the packet if needed.

## Related Documentation

- [Maintainer Handoff Guide](./maintainer-handoff-guide.md) - Security boundaries and deployment workflow
- [Testnet Deployment Checklist](./testnet-deployment-checklist.md) - Step-by-step deployment process
- [Release Readiness](./release-readiness.md) - Pre-deployment checklist
- [Deployment Failure and Recovery Runbook](./deployment-failure-and-recovery-runbook.md) - Troubleshooting deployment issues
