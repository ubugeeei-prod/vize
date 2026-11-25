#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-scaling/validate-inputs.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
[[ "$SHA" =~ ^[0-9a-f]{40}$ ]]
[[ "$SIZES" =~ ^([0-9]+(,[0-9]+)*)?$ ]]
[[ "$RUNS" =~ ^[1-9][0-9]?$ ]]
[[ "$SCALING_TOOLS" =~ ^([a-z0-9-]+(,[a-z0-9-]+)*)?$ ]]
[[ "$MONOREPO_SFCS" =~ ^[0-9]+$ ]]
[[ "$GIANT_BLOCKS" =~ ^[0-9]+$ ]]
[[ "$NUXT_SIZES" =~ ^([0-9]+(,[0-9]+)*)?$ ]]
