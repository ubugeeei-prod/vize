#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/fresco-dimension-bench/record-exact-source-of-both-parser-implementations.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$(git rev-parse HEAD)" == "$HEAD_SHA" ]]
echo "BASE_SHA=$HEAD_SHA" >> "$GITHUB_ENV"
