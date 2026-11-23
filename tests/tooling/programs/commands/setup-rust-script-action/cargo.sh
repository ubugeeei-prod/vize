#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >> "$CARGO_LOG"
if [ "${1:-}" != "install" ] || [ "${2:-}" != "rust-script" ]; then
  exit 65
fi
printf '0.34.0' > "$VERSION_STATE"
mkdir -p "$CARGO_HOME/bin"
cp "$FIXTURE_RUST_SCRIPT" "$CARGO_HOME/bin/rust-script"
chmod +x "$CARGO_HOME/bin/rust-script"
