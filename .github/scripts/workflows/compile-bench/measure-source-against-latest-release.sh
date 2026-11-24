#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/compile-bench/measure-source-against-latest-release.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
node tools/benchmarks/scripts/compile-history.ts \
  --input tools/benchmarks/scripts/__in__ \
  --vue "$PWD/tools/benchmarks/scripts/node_modules/@vue/compiler-sfc" \
  --native "$LATEST_VERSION=$(ls "$RUNNER_TEMP/native/$LATEST_VERSION"/package/*.node)" \
  --native "source-${GITHUB_SHA::10}=$(ls npm/native/*.node)" \
  --runs 5 --warmups 2 --rounds 2 \
  --out compile-source.md --json compile-source.json
cat compile-source.md >> "$GITHUB_STEP_SUMMARY"
