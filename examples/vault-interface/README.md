# Vault interface example

`schemas/vault-interface-v0.1.json` is the complete machine-readable interface document for the
currently implemented vault contract. It conforms to
`schemas/vault-interface.schema.json` and is intended for SDK compatibility
tests, fixture mirroring, and binding metadata.

From the repository root, inspect the ordered invocation metadata for a method:

```bash
node examples/vault-interface/consume-schema.js deposit
node examples/vault-interface/consume-schema.js claim_rewards
```

The example only reads interface metadata. A production SDK remains responsible
for converting declared values into Soroban `ScVal` arguments, simulating or
submitting the invocation, collecting required signatures, and decoding the
result.

See `docs/vault-interface-schema.md` for versioning, authorization, event, and
non-feature guidance.
