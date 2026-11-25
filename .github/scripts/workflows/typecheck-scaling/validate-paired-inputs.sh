#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-scaling/validate-paired-inputs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$BASE_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$HEAD_SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$SIZES" == "1000,5000,20000" ]]
