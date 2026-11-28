#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/04-typechecker-native-phases/scripts/capture-every-actual-shard-with-byte-map-diagnostic-and-freshness-gates.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/benchmarks/scripts/typechecker-native-phases.mjs \
  "$GITHUB_WORKSPACE/target/ci-opt/vize" \
  "$GITHUB_WORKSPACE/target/ci-opt/examples/native_phase_projection" \
  "$NATIVE_PHASE_OUTPUT"
