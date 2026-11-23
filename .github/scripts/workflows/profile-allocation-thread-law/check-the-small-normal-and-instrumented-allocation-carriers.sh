#!/usr/bin/env bash
set -euo pipefail
mkdir -p evidence
git rev-parse HEAD > evidence/source.sha
rustc +1.100.0 -vV > evidence/rustc.txt
lscpu > evidence/cpu.txt
node "$VIZE_CI_FILES/../shared/runner-resources.ts" report 'untimed correctness and source-counter law; no throughput comparison' RUNNER_NAME RUNNER_OS RUNNER_ARCH RUNNER_ENVIRONMENT CARGO_BUILD_JOBS RAYON_NUM_THREADS GOMAXPROCS > evidence/resources.json
cargo +1.100.0 test --locked --profile ci -p vize_l0 --test profile_allocation_thread_law --message-format json -- --nocapture 2>&1 | tee evidence/normal-law.log
cargo +1.100.0 test --locked --profile ci -p vize_l0 --test profile_allocation_thread_law --features micro-profiling --message-format json -- --nocapture 2>&1 | tee evidence/instrumented-law.log
cargo +1.100.0 test --locked --profile ci -p vize_l0 --features micro-profiling --doc ProfileGuard 2>&1 | tee evidence/send-law.log
node "$VIZE_CI_FILES/verify-send-law.ts"
cargo +1.100.0 test --locked --profile ci -p vize_l0 --test source_operations_profile --features micro-profiling 2>&1 | tee evidence/observer-law.log
cargo +1.100.0 test --locked --profile ci -p vize_carton --test davinci_profile_export --features micro-profiling per_span_allocation_counts_and_schema_validation 2>&1 | tee evidence/nested-export-law.log
