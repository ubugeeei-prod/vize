#!/usr/bin/env bash
set -euo pipefail
mkdir -p evidence
git rev-parse HEAD > evidence/source.sha
rustc +1.100.0 -vV > evidence/rustc.txt
lscpu > evidence/cpu.txt
node "$VIZE_CI_FILES/../shared/runner-resources.ts" report 'untimed correctness and source-counter law; no throughput comparison' RUNNER_NAME RUNNER_OS RUNNER_ARCH RUNNER_ENVIRONMENT CARGO_BUILD_JOBS RAYON_NUM_THREADS GOMAXPROCS > evidence/resources.json
cargo +1.100.0 test --locked --profile ci -p vize_maestro --test source_snapshot_profile -- --nocapture 2>&1 | tee evidence/normal.log
cargo +1.100.0 test --locked --profile ci -p vize_maestro --test source_snapshot_profile --features micro-profiling -- --nocapture 2>&1 | tee evidence/instrumented.log
cargo +1.100.0 test --locked --profile ci -p vize_maestro --lib server::state::workspace_project_files::symbols::allocation_tests::native_workspace_symbol_cache_allocation_law --features micro-profiling -- --exact --nocapture 2>&1 | tee evidence/native-workspace-symbol.log
cargo +1.100.0 test --locked --profile ci -p vize_l0 --test source_operations_profile 2>&1 | tee evidence/disabled-operands.log
node "$VIZE_CI_FILES/../shared/log-packets.ts" evidence/instrumented.log document_store_micro= 3 evidence/counts.json
node "$VIZE_CI_FILES/../shared/log-packets.ts" evidence/instrumented.log document_text_snapshot_allocation= 1 evidence/text-snapshot-allocation.json
node "$VIZE_CI_FILES/../shared/log-packets.ts" evidence/instrumented.log document_uri_snapshot_allocation= 1 evidence/uri-snapshot-allocation.json
node "$VIZE_CI_FILES/../shared/log-packets.ts" evidence/instrumented.log document_vue_batch_allocation= 1 evidence/vue-batch-allocation.json
node "$VIZE_CI_FILES/../shared/log-packets.ts" evidence/instrumented.log workspace_symbol_cache_allocation= 1 evidence/workspace-symbol-cache-allocation.json
node "$VIZE_CI_FILES/../shared/log-packets.ts" evidence/native-workspace-symbol.log native_workspace_symbol_cache_allocation= 1 evidence/native-workspace-symbol-cache-allocation.json
node "$VIZE_CI_FILES/../shared/log-packets.ts" evidence/native-workspace-symbol.log workspace_symbol_hit_selection_allocation= 1 evidence/workspace-symbol-hit-selection-allocation.json
node "$VIZE_CI_FILES/../shared/log-packets.ts" evidence/native-workspace-symbol.log native_disk_workspace_symbol_cache_allocation= 1 evidence/native-disk-workspace-symbol-cache-allocation.json
