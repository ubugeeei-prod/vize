#!/usr/bin/env bash
set -euo pipefail
export VIZE_TOOLCHAIN_HISTORY="$RUNNER_TEMP/shared-documents"
flags=(--locked --profile ci-opt -p vize_maestro --features experimental-lexical-arrow-queries,experimental-recovered-literal-prefix,experimental-verified-style-format,experimental-style-lint-products,micro-profiling --lib)
cargo clippy "${flags[@]}" --no-deps -- -D warnings
cargo test "${flags[@]}" toolchain_session_from_one_open_buffer -- --nocapture
export VIZE_COMPOSED_STYLE_INPUTS="$RUNNER_TEMP/vue-paths"
git ls-files -z -- '*.vue' '*.vue.txt' > "$VIZE_COMPOSED_STYLE_INPUTS"
cargo test "${flags[@]}" shared_style_queries_on_repository_documents -- --ignored --nocapture
cargo test "${flags[@]}" source_project::style_format::composition:: -- --include-ignored --nocapture --test-threads=1
test -s "$VIZE_TOOLCHAIN_HISTORY"
cargo test --locked --profile ci-opt -p vize_atelier_sfc --features bundler-block-products --test bundler_toolchain_session -- --ignored --nocapture
cargo test --locked --profile ci-opt --manifest-path crates/vize_incremental/providers/graph/Cargo.toml --test incremental original_template_graphs_follow_the_same_toolchain_session -- --ignored --nocapture
