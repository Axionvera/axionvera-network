# Vault Event Fixtures

These JSON files are compatibility fixtures for SDK and dashboard event
indexing work. They are mocked, testnet-ready examples of the event shape
emitted by `axionvera-vault-contract` v0.1.

Each file conforms to `schemas/vault-event.schema.json` and documents:

- the stable contract topic pair emitted on-chain;
- the SDK-facing event name used by downstream consumers;
- the typed payload field order;
- the rule that failed contract calls emit no vault event.

The fixtures are intentionally not an indexer. They provide stable examples for
parsers, dashboards, and SDK tests before live indexing is implemented.
`catalog.json` lists the full fixture set for tools that prefer discovering the
examples from one manifest.

| Fixture | Contract flow | On-chain topics | SDK event type |
| --- | --- | --- | --- |
| `initialize.json` | `initialize` | `["vault", "init"]` | `initialized` |
| `deposit.json` | `deposit` | `["vault", "deposit"]` | `deposit` |
| `withdraw.json` | `withdraw` | `["vault", "withdraw"]` | `withdraw` |
| `claim.json` | `claim_rewards` | `["vault", "claim"]` | `claim_rewards` |

Replace `ADDRESS_PLACEHOLDER` with a `G...` account id or `C...` contract id
when using the fixtures in a concrete testnet replay.
