#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt -p vize_patina --features authored-script-rules)
cargo clippy "${flags[@]}" --lib --no-deps -- -D warnings
cargo test "${flags[@]}" --doc rules::script::authored
export VIZE_SCRIPT_QUERY_INPUTS="$RUNNER_TEMP/editor-scripts"
cargo test "${flags[@]}" --test authored_script_rules -- --include-ignored --nocapture --test-threads=1
