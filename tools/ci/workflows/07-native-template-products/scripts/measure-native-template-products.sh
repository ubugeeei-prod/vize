#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/07-native-template-products/scripts/measure-native-template-products.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
report="$RUNNER_TEMP/native-template-products"
mkdir -p "$report"
git rev-parse HEAD > "$report/revision.txt"
rustc --version --verbose > "$report/rustc.txt"
uname -a > "$report/host.txt"
export VIZE_TEMPLATE_PRODUCT_INPUTS="$report/tracked-vue-inputs"
git ls-files -z -- '*.vue' > "$VIZE_TEMPLATE_PRODUCT_INPUTS"
flags=(--locked --profile ci-opt -p vize_maestro --features experimental-template-operand-products,experimental-lexical-arrow-queries,micro-profiling --lib)
cargo test "${flags[@]}" virtual_code::generator::template_products::tests:: -- --nocapture --test-threads=1 2>&1 | tee "$report/parity.log"
cargo test "${flags[@]}" source_project::template_products::tests:: -- --nocapture --test-threads=1 2>&1 | tee "$report/open-buffer-products.log"
cargo test "${flags[@]}" original_tool_template_projections_match -- --ignored --nocapture --test-threads=1 2>&1 | tee "$report/original-sources.log"
export VIZE_TEMPLATE_ADMISSION_REPORT="$report/native-admission.json"
cargo test "${flags[@]}" original_template_admission_families -- --ignored --nocapture --test-threads=1 2>&1 | tee "$report/native-admission.log"
cargo test "${flags[@]}" paired_template_tool_products -- --ignored --nocapture --test-threads=1 2>&1 | tee "$report/paired-resources.log"
cargo test "${flags[@]}" paired_extended_template_tool_products -- --ignored --nocapture --test-threads=1 2>&1 | tee "$report/extended-paired-resources.log"
