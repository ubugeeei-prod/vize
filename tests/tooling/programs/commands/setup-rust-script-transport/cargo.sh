#!/bin/bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/setup-rust-script-transport/cargo.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
printf "%s\n" "$*" >> "$CARGO_LOG"
if [ "$CARGO_STATUS" -eq 0 ]; then
  mkdir -p "$CARGO_HOME/bin"
  cp "$FIXTURE_RUNNER" "$CARGO_HOME/bin/rust-script"
  chmod 0755 "$CARGO_HOME/bin/rust-script"
fi
exit "$CARGO_STATUS"
