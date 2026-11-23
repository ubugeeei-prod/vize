#!/usr/bin/env bash
set -euo pipefail
export NODE_PATH="$RUNNER_TEMP/template-oracle/node_modules"
export VIZE_TEMPLATE_PASS_INPUTS="$RUNNER_TEMP/original-template-passes"
export VIZE_ORIGINAL_TEMPLATE_ORACLE="$RUNNER_TEMP/template-oracle.json"
npm install --prefix "$RUNNER_TEMP/template-oracle" --no-audit --no-fund --ignore-scripts @vue/compiler-sfc@3.5.35
node davinci/vize_l1_to_l2/tests/original_template_queries/oracle.ts "$RUNNER_TEMP/original-template-passes" "$VIZE_ORIGINAL_TEMPLATE_ORACLE"
flags=(--locked --profile ci-opt -p vize_l1_to_l2 --features original-template-queries)
cargo clippy "${flags[@]}" --lib --no-deps -- -D warnings
cargo test "${flags[@]}" --test original_template_queries -- --include-ignored --nocapture --test-threads=1
cargo test --locked --profile ci-opt -p vize_l1 --features independent-template-block --doc CheckedTemplateBlock
cargo test --locked --profile ci-opt -p vize_maestro --features vize_l1_to_l2/original-template-queries --test original_template_requests
resident=(--locked --profile ci-opt -p vize_incremental --features resident-template-passes)
cargo clippy "${resident[@]}" --lib --no-deps -- -D warnings
cargo test "${resident[@]}" --lib template::tests:: -- --include-ignored --nocapture --test-threads=1
cargo test "${resident[@]}" --test resident_template_cost -- --include-ignored --nocapture --test-threads=1
cargo clippy --locked --profile ci-opt -p vize_maestro --features experimental-resident-template-passes --lib --no-deps -- -D warnings
cargo test --locked --profile ci-opt -p vize_maestro --features experimental-resident-template-passes,micro-profiling --lib source_project::template:: -- --include-ignored --nocapture --test-threads=1
