#!/usr/bin/env bash
# rotate_ca.sh - Fetch a new client CA bundle, validate, atomically replace local CA, and optionally restart service.
# Usage: rotate_ca.sh <url> <dest_path> [--restart-cmd "systemctl restart axionvera-node"]
set -euo pipefail
if [ "$#" -lt 2 ]; then
  echo "Usage: $0 <ca_pem_url> <dest_path> [--restart-cmd \"command\"]"
  exit 2
fi
CA_URL="$1"
DEST_PATH="$2"
RESTART_CMD=""
if [ "$#" -ge 3 ]; then
  RESTART_CMD="$3"
fi
TMPFILE=$(mktemp /tmp/ca_pem.XXXXXX)
trap 'rm -f "$TMPFILE"' EXIT
curl -fsSL "$CA_URL" -o "$TMPFILE"
# Basic validation: check for PEM header
if ! grep -q "BEGIN CERTIFICATE" "$TMPFILE"; then
  echo "Fetched file does not look like a PEM certificate" >&2
  exit 3
fi
# Atomically replace
mkdir -p "$(dirname "$DEST_PATH")"
chmod 644 "$TMPFILE"
mv "$TMPFILE" "${DEST_PATH}.tmp"
mv -f "${DEST_PATH}.tmp" "$DEST_PATH"
echo "Replaced CA at $DEST_PATH"
if [ -n "$RESTART_CMD" ]; then
  echo "Running restart command: $RESTART_CMD"
  eval "$RESTART_CMD"
fi
echo "CA rotation complete"
