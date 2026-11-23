#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt)
features=(--features vize_canon/original-style-binding-tests)
cargo clippy "${flags[@]}" -p vize_l1 -p vize_l1_to_l2 -p vize_l4 -p vize_canon -p vize_atelier_sfc "${features[@]}" --lib --no-deps -- -D warnings
cargo test "${flags[@]}" -p vize_canon "${features[@]}" --lib css_var_usage::
export VIZE_STYLE_BINDING_INPUTS="$RUNNER_TEMP/original-style-inputs"
git ls-files -z -- '*.vue' '*.vue.txt' > "$VIZE_STYLE_BINDING_INPUTS"
cargo test "${flags[@]}" -p vize_canon "${features[@]}" --test original_style_bindings -- --include-ignored --nocapture --test-threads=1
cargo run "${flags[@]}" -p vize_canon "${features[@]}" --example original_style_binding_profile
