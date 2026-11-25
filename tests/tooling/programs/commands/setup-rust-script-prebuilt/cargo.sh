#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/setup-rust-script-prebuilt/cargo.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
printf '%s\n' "$*" > "$CARGO_LOG"
mkdir -p "$CARGO_HOME/bin"
cp "$FIXTURE_RUNNER" "$CARGO_HOME/bin/rust-script"
chmod +x "$CARGO_HOME/bin/rust-script"
