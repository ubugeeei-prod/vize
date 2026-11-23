#!/usr/bin/env bash
set -euo pipefail
rust_script_version="0.34.0"
download_defer_status=75
if command -v rust-script >/dev/null 2>&1; then
  installed_version="$(rust-script --version | sed -E 's/^rust-script[[:space:]]+([^[:space:]]+).*/\1/')"
  if [ "$installed_version" = "$rust_script_version" ]; then
    rust-script --version
    exit 0
  fi
  echo "Found rust-script ${installed_version}; installing ${rust_script_version}" >&2
fi
cargo_bin="${CARGO_HOME:-$HOME/.cargo}/bin"
rust_script_installed=false
if [ "${RUNNER_OS:-}" = "Linux" ] && [ "${RUNNER_ARCH:-}" = "X64" ]; then
  rust_script_temp="$(mktemp -d "$RUNNER_TEMP/rust-script.XXXXXX")"
  trap 'rm -rf "$rust_script_temp"' EXIT
  archive="$rust_script_temp/rust-script.tar.gz"
  download_status=0
  curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
    --output "$archive" \
    "https://github.com/fornwall/rust-script/releases/download/0.34.0/rust-script-x86_64-unknown-linux-gnu.tar.gz" || download_status=$?
  if [ "$download_status" -eq 0 ]; then
    bash "$GITHUB_ACTION_PATH/extract-prebuilt.sh" "$archive" "$rust_script_temp/runner" \
      0700ecfe6c768c007f3e5a7919f4fedbf0a6727d275a196791f134a1742d4d13
    mkdir -p "$cargo_bin"
    install -m 0755 "$rust_script_temp/runner/rust-script" "$cargo_bin/rust-script"
    rust_script_installed=true
  elif [ "$download_status" -ne "$download_defer_status" ]; then
    exit "$download_status"
  else
    echo "::warning::rust-script release assets are unavailable; building the pinned version from crates.io"
  fi
fi
if [ "$rust_script_installed" != true ]; then
  cargo install rust-script --version "$rust_script_version" --locked --force
fi
export PATH="$cargo_bin:$PATH"
echo "$cargo_bin" >> "$GITHUB_PATH"
installed_version="$(rust-script --version | sed -E 's/^rust-script[[:space:]]+([^[:space:]]+).*/\1/')"
if [ "$installed_version" != "$rust_script_version" ]; then
  echo "::error::expected rust-script ${rust_script_version}, got $(rust-script --version)" >&2
  exit 1
fi
rust-script --version
