#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/release-build-cache/read-the-version-the-next-release-stamps.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
version=$(node tools/support/release/release/main.ts next)
printf 'version=%s\n' "$version" >> "$GITHUB_OUTPUT"
