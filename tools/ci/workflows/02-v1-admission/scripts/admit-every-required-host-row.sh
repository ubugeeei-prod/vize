#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/02-v1-admission/scripts/admit-every-required-host-row.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
args=(
  admit-run
  --candidate "$CANDIDATE_SHA"
  --run "$GITHUB_RUN_ID"
  --run-attempt "$GITHUB_RUN_ATTEMPT"
  --receipts "$HOST_RECEIPTS"
  --output .qualification/v1-exact-candidate-manifest.json
)
if [ "$ENFORCE_COMPLETE" = true ]; then args+=(--enforce-complete); fi
node tools/commands/release/v1-exact-candidate.ts "${args[@]}"
