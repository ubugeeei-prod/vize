#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/tests/tooling/programs/commands/setup-rust-script-action/cargo.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
printf '%s\n' "$*" >> "$CARGO_LOG"
if [ "${1:-}" != "install" ] || [ "${2:-}" != "rust-script" ]; then
  exit 65
fi
printf '0.34.0' > "$VERSION_STATE"
mkdir -p "$CARGO_HOME/bin"
cp "$FIXTURE_RUST_SCRIPT" "$CARGO_HOME/bin/rust-script"
chmod +x "$CARGO_HOME/bin/rust-script"
