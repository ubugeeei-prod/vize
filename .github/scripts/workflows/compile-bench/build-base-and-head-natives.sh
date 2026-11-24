#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/compile-bench/build-base-and-head-natives.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
for side in base head; do
  dir=$PWD
  if [[ $side == base ]]; then dir=$RUNNER_TEMP/base; fi
  (cd "$dir" && CARGO_TARGET_DIR="$RUNNER_TEMP/target-$side" cargo build --release -p vize_vitrine --features napi,legacy --lib)
  cp "$RUNNER_TEMP/target-$side/release/libvize_vitrine.so" "$RUNNER_TEMP/$side.node"
  rm -rf "$RUNNER_TEMP/target-$side"
done
