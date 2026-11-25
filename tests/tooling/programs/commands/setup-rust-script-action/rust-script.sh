#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/setup-rust-script-action/rust-script.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
if [ "${1:-}" = "--version" ]; then
  printf 'rust-script %s\n' "$(cat "$VERSION_STATE")"
  exit 0
fi
exit 64
