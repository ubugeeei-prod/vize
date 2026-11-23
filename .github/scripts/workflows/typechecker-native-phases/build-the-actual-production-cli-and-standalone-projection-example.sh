#!/usr/bin/env bash
set -euo pipefail
node tools/benchmarks/scripts/typechecker-native-source-custody.mjs "$RUNNER_TEMP/typechecker-native-source-custody.json"
cd "$NATIVE_PHASE_SOURCE_ROOT"
cargo build --locked --profile ci-opt -p vize \
  --config 'profile.ci-opt.inherits="release"' \
  --config 'profile.ci-opt.lto="thin"' \
  --config 'profile.ci-opt.codegen-units=16' \
  --config 'profile.ci-opt.package.vize.strip="symbols"'
cargo build --locked --profile ci-opt -p vize_canon --example native_phase_projection \
  --config 'profile.ci-opt.inherits="release"' \
  --config 'profile.ci-opt.lto="thin"' \
  --config 'profile.ci-opt.codegen-units=16'
