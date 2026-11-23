#!/usr/bin/env bash
set -euo pipefail
if [ "${1:-}" = "--version" ]; then
  printf 'rust-script %s\n' "$(cat "$VERSION_STATE")"
  exit 0
fi
exit 64
