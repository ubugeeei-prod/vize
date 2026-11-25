#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-performance-observation/record-exact-comparison-sources.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ && "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$(git -C base rev-parse HEAD)" == "$BASE_SHA" ]]
[[ "$(git -C head rev-parse HEAD)" == "$HEAD_SHA" ]]
[[ "$(git -C driver rev-parse HEAD)" == "$DRIVER_SHA" ]]
git -C head fetch --no-tags "$GITHUB_WORKSPACE/base" "$BASE_SHA"
mkdir -p performance-observation/build
node "$VIZE_CI_FILES/record-exact-comparison-sources.ts"
