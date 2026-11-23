#!/usr/bin/env bash
set -euo pipefail
{
  echo "CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=rust-lld"
  echo "CARGO_TARGET_AARCH64_PC_WINDOWS_MSVC_LINKER=rust-lld"
} >> "$GITHUB_ENV"
