#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/typecheck-scaling/build-the-native-binding-and-the-vize-package.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
# The whole workspace: vue-tsc and vue come from tools/benchmarks/scripts, tsgo from the root.
vp install --frozen-lockfile --prefer-offline
cargo --version
vp run --workspace-root build:native > "$RUNNER_TEMP/build-native.log" 2>&1 || { tail -n 80 "$RUNNER_TEMP/build-native.log"; exit 1; }
tail -n 20 "$RUNNER_TEMP/build-native.log"
if [[ -n "$NUXT_SIZES" ]]; then
  # The @vizejs/nuxt build lane loads the module and its Vite plugin as well.
  vp run --workspace-root build:vite-plugin
  vp run --workspace-root build:nuxt-stack
else
  vp run --filter './npm/cli' build
fi
git rev-parse HEAD
