#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$VIZE_ESLINT_EVIDENCE"
node "$VIZE_CI_FILES/preserve-exact-source-and-dependency-authority.ts"
