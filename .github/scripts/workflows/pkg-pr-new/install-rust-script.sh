#!/usr/bin/env bash
set -euo pipefail
rustup toolchain install 1.100.0 --profile minimal --no-self-update
rustup default 1.100.0
if command -v rust-script >/dev/null 2>&1; then
  rust_script_version="$(rust-script --version)"
  printf '%s\n' "$rust_script_version"
  if grep -Eq '(^|[[:space:]])0\.34\.0([[:space:]]|$)' <<<"$rust_script_version"; then
    exit 0
  fi
fi
cargo install rust-script --version 0.34.0 --locked
rust-script --version
