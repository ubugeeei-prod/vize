#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-scaling/record-exact-source-pair.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$(git -C base rev-parse HEAD)" == "$BASE_SHA" ]]
[[ "$(git -C head rev-parse HEAD)" == "$HEAD_SHA" ]]
[[ "$(git -C driver rev-parse HEAD)" == "$DRIVER_SHA" ]]
mkdir -p paired-scaling/build
