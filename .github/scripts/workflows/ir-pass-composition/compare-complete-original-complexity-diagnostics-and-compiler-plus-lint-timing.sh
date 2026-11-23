#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt -p vize_patina --features original-template-lint)
cargo clippy "${flags[@]}" --lib --no-deps -- -D warnings
cargo test "${flags[@]}" --test original_template_lint -- --include-ignored --nocapture --test-threads=1
cargo test "${flags[@]}" --lib max_template_complexity::
