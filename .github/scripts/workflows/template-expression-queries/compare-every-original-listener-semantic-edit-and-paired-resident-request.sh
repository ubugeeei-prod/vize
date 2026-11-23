#!/usr/bin/env bash
set -euo pipefail
export VIZE_EXPRESSION_QUERY_MANIFEST="$RUNNER_TEMP/original-listener-inputs"
export VIZE_REQUIRE_REPOSITORY_LISTENERS=1
git ls-files -z -- '*.vue' '*.vue.txt' > "$VIZE_EXPRESSION_QUERY_MANIFEST"
flags=(--locked --profile ci-opt)
native=(-p vize_l1_to_l2 --features component-listener-queries,template-operand-queries,vize_l2/position-index)
editor=(-p vize_maestro --features experimental-component-listeners,experimental-template-operand-products,micro-profiling --lib)
cargo clippy "${flags[@]}" -p vize_l2 --features component-listener-queries,template-operand-queries --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" "${native[@]}" --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" "${editor[@]}" --no-deps -- -D warnings
cargo test "${flags[@]}" "${native[@]}" --test expression_queries component_listeners:: -- --include-ignored --nocapture --test-threads=1
cargo test "${flags[@]}" "${editor[@]}" event_rename:: -- --include-ignored --nocapture --test-threads=1
cargo test "${flags[@]}" "${editor[@]}" named_dom_event_expressions_keep_their_checker_links_and_editor_exports
cargo test "${flags[@]}" "${editor[@]}" original_event_documents -- --ignored --nocapture --test-threads=1
