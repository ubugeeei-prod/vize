#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-compile/scripts/fetch-published-natives.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
for v in $COMPILE_BENCH_VERSIONS; do
  dir="$RUNNER_TEMP/native/$v"
  mkdir -p "$dir"
  curl -fsSL "https://registry.npmjs.org/@vizejs/native-linux-x64-gnu/-/native-linux-x64-gnu-$v.tgz" | tar -xz -C "$dir"
  file "$dir"/package/*.node
done
