#!/usr/bin/env bash
set -euo pipefail
mkdir -p evidence
git rev-parse HEAD > evidence/source.sha
rustc +1.100.0 -vV > evidence/rustc.txt
cargo +1.100.0 test --locked --profile ci -p vize_l0 --test source_operations_profile 2>&1 | tee evidence/disabled-tests.log
cargo +1.100.0 test --locked --profile ci -p vize_l0 --test source_operations_profile --features micro-profiling 2>&1 | tee evidence/instrumented-tests.log
cargo +1.100.0 test --locked --profile ci -p vize_carton --features micro-profiling profile_export 2>&1 | tee evidence/export-tests.log
cargo +1.100.0 test --locked --profile ci -p vize_l0 --lib profiler::micro_hash --features micro-profiling 2>&1 | tee evidence/hash-digest-parity.log
cargo +1.100.0 test --locked --profile ci -p vize_carton --features micro-profiling --test davinci_profile_export 2>&1 | tee evidence/export-schema-tests.log
test "$(cargo +1.100.0 test --locked --profile ci -p vize_l0 --test source_operations_profile --features micro-profiling -- --list --format terse)" = 'source_operations_profile: test'
test -z "$(cargo +1.100.0 test --locked --profile ci -p vize_l0 --test source_operations_profile --features micro-profiling -- --list --ignored --format terse)"
