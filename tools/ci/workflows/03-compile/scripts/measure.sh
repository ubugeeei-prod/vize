#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-compile/scripts/measure.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
args=()
for v in $COMPILE_BENCH_VERSIONS; do
  args+=(--native "$v=$(ls "$RUNNER_TEMP/native/$v"/package/*.node)")
done
node tools/benchmarks/scripts/compile-history.ts \
  --input tools/benchmarks/scripts/__in__ \
  --vue "$PWD/tools/benchmarks/scripts/node_modules/@vue/compiler-sfc" \
  "${args[@]}" --runs 5 --warmups 2 --rounds 2 \
  --out compile-history.md --json compile-history.json
node "$VIZE_CI_FILES/measure.cjs" >> compile-history.md
cat compile-history.md >> "$GITHUB_STEP_SUMMARY"
