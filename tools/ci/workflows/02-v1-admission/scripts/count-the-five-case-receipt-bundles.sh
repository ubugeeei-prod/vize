#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/02-v1-admission/scripts/count-the-five-case-receipt-bundles.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
mkdir -p "$FIVE_CASE_BUNDLES"
count=$(( $(find "$FIVE_CASE_BUNDLES" -maxdepth 1 -type f -name '*.json' | wc -l) ))
echo "Found $count five-case receipt bundles"
echo "count=$count" >> "$GITHUB_OUTPUT"
