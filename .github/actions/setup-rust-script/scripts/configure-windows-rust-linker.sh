#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/setup-rust-script/scripts/configure-windows-rust-linker.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
{
  echo "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=rust-lld"
  echo "CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER=rust-lld"
} >> "$GITHUB_ENV"
