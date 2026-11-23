#!/usr/bin/env bash
set -euo pipefail
flags=(--locked --profile ci-opt -p vize_l1_to_l2 --features observed-template-passes)
export VIZE_TEMPLATE_PASS_INPUTS="$RUNNER_TEMP/original-template-passes"
export VIZE_COMPOSED_TEMPLATE_FILES="$RUNNER_TEMP/original-cfg-templates"
git ls-files -z -- '*.vue' '*.vue.txt' > "$VIZE_TEMPLATE_PASS_INPUTS"
git ls-files -z -- '*.vue' > "$VIZE_COMPOSED_TEMPLATE_FILES"
cargo clippy "${flags[@]}" --lib --no-deps -- -D warnings
cargo test --locked --profile ci-opt -p vize_l2 --features composed-template-analysis --lib lattice::composed::
cargo test "${flags[@]}" --test observed_template_passes -- --include-ignored --nocapture --test-threads=1
cargo test "${flags[@]}" --test cfg_complexity_oracle composed:: -- --include-ignored --nocapture --test-threads=1
