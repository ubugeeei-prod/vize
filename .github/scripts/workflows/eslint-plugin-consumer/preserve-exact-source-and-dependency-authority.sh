#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/eslint-plugin-consumer/preserve-exact-source-and-dependency-authority.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p "$VIZE_ESLINT_EVIDENCE"
node "$VIZE_CI_FILES/preserve-exact-source-and-dependency-authority.ts"
