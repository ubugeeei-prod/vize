#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt)
editor=(-p vize_maestro --features experimental-style-lint-products,micro-profiling --lib)
cargo clippy "${flags[@]}" -p vize_patina --features observed-style-rules --lib --no-deps -- -D warnings
cargo test "${flags[@]}" -p vize_patina --features observed-style-rules --lib rules::css
cargo clippy "${flags[@]}" -p vize_canon --features observed-style-modules --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" "${editor[@]}" --no-deps -- -D warnings
cargo test "${flags[@]}" -p vize_l1 --features observed-styles --lib css::lang::
cargo test "${flags[@]}" -p vize_canon --features observed-style-modules --lib css_module
cargo test "${flags[@]}" "${editor[@]}" css_module
export VIZE_STYLE_MODULE_INPUTS="$RUNNER_TEMP/style-module-sources"
cargo test "${flags[@]}" "${editor[@]}" original_style_module_products_match -- --ignored --nocapture --test-threads=1
cargo test "${flags[@]}" "${editor[@]}" original_style_lint_products_match -- --ignored --nocapture --test-threads=1
