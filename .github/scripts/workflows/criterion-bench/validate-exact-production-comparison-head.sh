#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/criterion-bench/validate-exact-production-comparison-head.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$HEAD_SHA" == "$DISPATCH_SHA" ]]
[[ "$(git rev-parse HEAD)" == "$HEAD_SHA" ]]
git merge-base --is-ancestor "$BASE_SHA" "$HEAD_SHA"
