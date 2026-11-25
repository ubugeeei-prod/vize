#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/compile-bench/fetch-latest-published-native.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
v="${COMPILE_BENCH_VERSIONS##* }"
mkdir -p "$RUNNER_TEMP/native/$v"
curl -fsSL "https://registry.npmjs.org/@vizejs/native-linux-x64-gnu/-/native-linux-x64-gnu-$v.tgz" | tar -xz -C "$RUNNER_TEMP/native/$v"
echo "LATEST_VERSION=$v" >> "$GITHUB_ENV"
