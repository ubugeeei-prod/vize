#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt)
editor=(-p vize_maestro --features experimental-script-queries,micro-profiling --lib)
cargo clippy "${flags[@]}" -p vize_l1 --features authored-script-queries --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" "${editor[@]}" --no-deps -- -D warnings
cargo test "${flags[@]}" "${editor[@]}" server::document_structure::symbols::tests
cargo test "${flags[@]}" "${editor[@]}" ide::file_rename::
export VIZE_SCRIPT_QUERY_INPUTS="$RUNNER_TEMP/editor-scripts"
git ls-files -z -- '*.vue' '*.vue.txt' > "$VIZE_SCRIPT_QUERY_INPUTS"
cargo test "${flags[@]}" "${editor[@]}" source_project::script_queries:: -- --include-ignored --nocapture --test-threads=1
