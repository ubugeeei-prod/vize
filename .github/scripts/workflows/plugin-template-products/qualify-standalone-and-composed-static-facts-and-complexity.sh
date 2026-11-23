#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt -p vize_l1_to_l2)
export VIZE_COMPOSED_TEMPLATE_FILES="$RUNNER_TEMP/authored-static-templates"
git ls-files -z -- '*.vue' > "$VIZE_COMPOSED_TEMPLATE_FILES"
cargo test "${flags[@]}" --test cfg_complexity_oracle composed:: -- --include-ignored --nocapture --test-threads=1
cargo test "${flags[@]}" --test composed_template_alloc
