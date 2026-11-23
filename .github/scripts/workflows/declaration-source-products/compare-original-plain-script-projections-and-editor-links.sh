#!/usr/bin/env bash
set -euo pipefail
export VIZE_BRIDGE_INPUTS="$RUNNER_TEMP/original-plain-bridge-inputs"
git ls-files -z -- '*.vue' '*.vue.txt' > "$VIZE_BRIDGE_INPUTS"
flags=(--locked --profile ci-opt -p vize_canon --lib --features plain-script-bridges)
cargo clippy "${flags[@]}" --no-deps -- -D warnings
cargo test --locked --profile ci-opt --lib -p vize_l2 --features script-declaration-queries query::script::declaration::
cargo test "${flags[@]}" plain_exports::observed:: -- --include-ignored --nocapture --test-threads=1
