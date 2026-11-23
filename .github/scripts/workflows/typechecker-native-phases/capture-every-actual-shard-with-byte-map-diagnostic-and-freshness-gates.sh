#!/usr/bin/env bash
node tools/benchmarks/scripts/typechecker-native-phases.mjs \
  "$GITHUB_WORKSPACE/target/ci-opt/vize" \
  "$GITHUB_WORKSPACE/target/ci-opt/examples/native_phase_projection" \
  "$NATIVE_PHASE_OUTPUT"
