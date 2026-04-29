#!/usr/bin/env bash
# Helper to create a feature branch for the mTLS enforcement changes and commit local modifications.
BRANCH_NAME=${1:-feature/mtls-enforcement}
set -euo pipefail
if [ -n "$(git status --porcelain)" ]; then
  echo "Working tree is dirty; please commit or stash changes before creating the branch." >&2
  exit 1
fi
git checkout -b "$BRANCH_NAME"
# Optionally stage and commit all changes
if [ "$#" -ge 2 ] && [ "$2" = "--commit-all" ]; then
  git add -A
  git commit -m "feat: enforce mTLS, add CA rotation tooling"
  echo "Committed changes on $BRANCH_NAME"
else
  echo "Created branch $BRANCH_NAME. Use --commit-all to commit current changes." 
fi
