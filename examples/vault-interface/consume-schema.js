#!/usr/bin/env node

const fs = require("node:fs");
const path = require("node:path");

const SUPPORTED_SCHEMA_VERSION = "1";
const SUPPORTED_INTERFACE_VERSION = "0.1";

function loadInterface() {
  const file = path.join(__dirname, "..", "..", "schemas", "vault-interface-v0.1.json");
  const manifest = JSON.parse(fs.readFileSync(file, "utf8"));

  if (manifest.schema_version !== SUPPORTED_SCHEMA_VERSION) {
    throw new Error(`Unsupported schema version: ${manifest.schema_version}`);
  }
  if (manifest.interface_version !== SUPPORTED_INTERFACE_VERSION) {
    throw new Error(`Unsupported interface version: ${manifest.interface_version}`);
  }

  return manifest;
}

function invocationMetadata(manifest, methodName) {
  const method = manifest.methods.find(({ name }) => name === methodName);
  if (!method) {
    throw new Error(`Unknown vault method: ${methodName}`);
  }

  const argumentsInContractOrder = [...method.arguments]
    .sort((left, right) => left.position - right.position)
    .map(({ name, type }) => ({ name, type }));
  const events = method.emits.map((eventName) => {
    const event = manifest.events.find(({ name }) => name === eventName);
    if (!event) {
      throw new Error(`Missing event definition: ${eventName}`);
    }
    return {
      name: event.name,
      topics: [...event.topics]
        .sort((left, right) => left.position - right.position)
        .map(({ type, value }) => ({ type, value })),
    };
  });

  return {
    contractSymbol: method.name,
    arguments: argumentsInContractOrder,
    returns: method.returns,
    authorization: method.authorization,
    events,
  };
}

if (require.main === module) {
  const manifest = loadInterface();
  const methodName = process.argv[2] || "deposit";
  console.log(JSON.stringify(invocationMetadata(manifest, methodName), null, 2));
}

module.exports = { invocationMetadata, loadInterface };
