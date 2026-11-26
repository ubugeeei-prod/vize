#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/01-pr-source/scripts/require-every-selected-tooling-worker.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
echo "selected=$SELECTED cli=$CLI workers=$WORKERS"
[[ "$CLI" == success ]]
if [[ "$SELECTED" == true ]]; then [[ "$WORKERS" == success ]]
else [[ "$SELECTED" == false && "$WORKERS" == skipped ]]; fi
