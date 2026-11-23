#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt)
checker=(-p vize_canon --features authored-declarations --lib)
editor=(-p vize_maestro --features experimental-declaration-products,micro-profiling --lib)
export VIZE_DECLARATION_INPUTS="$RUNNER_TEMP/declaration-inputs"
cargo clippy "${flags[@]}" -p vize_l1 --features authored-declarations --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" "${checker[@]}" --no-deps -- -D warnings
cargo clippy "${flags[@]}" "${editor[@]}" --no-deps -- -D warnings
cargo test "${flags[@]}" "${checker[@]}" batch::vue_template_ref_module::
cargo test "${flags[@]}" "${checker[@]}" batch::import_rewriter_dts_tests::
cargo test "${flags[@]}" "${editor[@]}" server::state::global_tag_names::
cargo test "${flags[@]}" "${editor[@]}" original_declaration_products_match -- --ignored --nocapture --test-threads=1
