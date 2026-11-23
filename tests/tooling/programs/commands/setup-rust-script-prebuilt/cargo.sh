#!/bin/bash
set -euo pipefail
printf '%s\n' "$*" > "$CARGO_LOG"
mkdir -p "$CARGO_HOME/bin"
cp "$FIXTURE_RUNNER" "$CARGO_HOME/bin/rust-script"
chmod +x "$CARGO_HOME/bin/rust-script"
