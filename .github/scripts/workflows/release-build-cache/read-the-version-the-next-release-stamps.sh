#!/usr/bin/env bash
set -euo pipefail
version=$(node tools/support/release/release/main.ts next)
printf 'version=%s\n' "$version" >> "$GITHUB_OUTPUT"
