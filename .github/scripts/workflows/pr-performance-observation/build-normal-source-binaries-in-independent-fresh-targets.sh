#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/pr-performance-observation/build-normal-source-binaries-in-independent-fresh-targets.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
for role in base head; do
  target="$RUNNER_TEMP/pr-performance-$role-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
  node driver/tools/benchmarks/scripts/pr-performance-build.ts \
    --source-root "$GITHUB_WORKSPACE/$role" --target-dir "$target" \
    --binary "$target/ci-opt/vize" --role "$role" \
    --receipt "$GITHUB_WORKSPACE/performance-observation/build/$role.json"
done
