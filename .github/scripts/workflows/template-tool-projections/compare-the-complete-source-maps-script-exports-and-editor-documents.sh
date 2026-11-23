#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt)
editor=(-p vize_maestro --features experimental-template-operand-products,micro-profiling --lib)
cargo clippy "${flags[@]}" -p vize_l1_to_l2 --features template-operand-queries --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" -p vize_canon --features template-operand-projection --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" "${editor[@]}" --no-deps -- -D warnings
cargo test "${flags[@]}" -p vize_canon --features template-operand-projection --lib projection::expr::
cargo test "${flags[@]}" "${editor[@]}" virtual_code::generator::
export VIZE_TEMPLATE_PRODUCT_INPUTS="$RUNNER_TEMP/template-product-sources"
git ls-files -z -- '*.vue' '*.vue.txt' > "$VIZE_TEMPLATE_PRODUCT_INPUTS"
cargo test "${flags[@]}" "${editor[@]}" original_tool_template_projections_match -- --ignored --nocapture --test-threads=1
