#!/usr/bin/env bash
set -euo pipefail
export NODE_PATH="$RUNNER_TEMP/scss-oracle/node_modules"
export VIZE_SCSS_MANIFEST="$RUNNER_TEMP/scss-inputs"
export VIZE_REQUIRE_REPOSITORY_SCSS=1
git ls-files -z -- '*.vue' '*.vue.txt' '*.scss' > "$VIZE_SCSS_MANIFEST"
flags=(--locked --profile ci-opt)
host=(-p vize_atelier_sfc --features native-scoped-scss)
editor=(-p vize_maestro --features experimental-scss-definitions)
cargo clippy "${flags[@]}" -p vize_l1_to_l2 --features component-text-condense --lib --no-deps -- -D warnings
cargo test "${flags[@]}" -p vize_l1_to_l2 --features component-text-condense --lib
cargo clippy "${flags[@]}" -p vize_l2 --features scss-rules --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" -p vize_l4 --features scoped-scss,indexed-source-maps --lib --no-deps -- -D warnings
cargo test "${flags[@]}" -p vize_l4 --features indexed-source-maps --lib write::source_map::indexed
cargo test "${flags[@]}" -p vize_l4 --features indexed-source-maps,serde_json/preserve_order --lib write::source_map::indexed
cargo clippy "${flags[@]}" "${host[@]}" --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" "${editor[@]}" --lib --no-deps -- -D warnings
cargo test "${flags[@]}" "${host[@]}" --test scss_products -- --include-ignored --nocapture --test-threads=1
cargo test "${flags[@]}" "${host[@]}" --test native_sfc_scss -- --include-ignored --nocapture --test-threads=1
cargo test "${flags[@]}" "${editor[@]}" --lib source_project::scss:: -- --include-ignored --nocapture --test-threads=1

export VIZE_FORMAT_SCSS_INPUTS="$VIZE_SCSS_MANIFEST"
cargo test "${flags[@]}" -p vize_glyph --features observed-style --test observed_style -- --include-ignored --nocapture --test-threads=1
cargo test "${flags[@]}" -p vize_maestro --features experimental-verified-style-format --lib source_project::style_format:: -- --include-ignored --nocapture --test-threads=1
cargo clippy "${flags[@]}" -p vize_glyph --features observed-style --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" -p vize_atelier_sfc --features verified-scss-edits --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" -p vize_maestro --features experimental-verified-style-format --lib --no-deps -- -D warnings
cargo run "${flags[@]}" -p vize_atelier_sfc --features verified-scss-edits --example verified_scss_profile
cargo test "${flags[@]}" -p vize_maestro --features experimental-verified-style-format,experimental-style-lint-products,micro-profiling --lib source_project::style_format::composition:: -- --include-ignored --nocapture --test-threads=1
