#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/lsp-vs-volar/plan.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
args=()
for id in $FIXTURES; do args+=(--fixture "$id"); done
# Assigned first, so an unknown fixture fails this step instead of planning nothing.
jobs=$(node tools/benchmarks/scripts/lsp-vs-volar.ts --plan "${args[@]}")
echo "jobs=$jobs" >> "$GITHUB_OUTPUT"
