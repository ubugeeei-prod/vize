#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/04-typechecker-native-phases/scripts/check-exact-neutral-inputs-strict-phase-parser-and-projection-probe.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/benchmarks/scripts/typechecker-native-phases.mjs --self-test; node --test tools/benchmarks/scripts/typechecker-native-*.test.mjs
node tools/benchmarks/scripts/typechecker-native-source-custody.mjs "$RUNNER_TEMP/typechecker-native-source-custody.json"; cd "$NATIVE_PHASE_SOURCE_ROOT"
cargo clippy --locked -p vize_canon --example native_phase_projection -- -D warnings
