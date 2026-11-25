#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/capture-cargo-status.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo() {
  printf '%s\n' '{"reason":"compiler-artifact","filenames":[]}'
  return "$MOCK_CARGO_STATUS"
}

source "$1"
