# Protocol Contract Dependency Report

✅ No circular dependencies detected.

## Dependency List
- **accounting** depends on: events
- **assets** depends on: events
- **auth** has no internal dependencies.
- **config** depends on: events
- **core** depends on: events, state, storage
- **events** has no internal dependencies.
- **interfaces** depends on: events
- **lifecycle** has no internal dependencies.
- **metrics** has no internal dependencies.
- **monitoring** depends on: events
- **orchestrator** depends on: events, interfaces
- **registry** depends on: events, interfaces
- **rewards** has no internal dependencies.
- **security** has no internal dependencies.
- **state** depends on: events
- **storage** depends on: events, state
- **treasury** depends on: events, interfaces
- **upgrades** has no internal dependencies.
- **vault-contract** depends on: auth, events, accounting, core, interfaces, security, vault-contract-v2
- **vault-contract-v2** has no internal dependencies.
