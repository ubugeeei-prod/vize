#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-cold-pair/check-planted-diagnostics-in-the-planted-corpora.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/benchmarks/scripts/typecheck-corpus-mutations-run.ts \
  --base "$RUNNER_TEMP/pair-base" \
  --head "$RUNNER_TEMP/pair-head" \
  --work "$RUNNER_TEMP/pair-work" \
  --out "$RUNNER_TEMP/pair-out" \
  --corpora "$CORPORA"
