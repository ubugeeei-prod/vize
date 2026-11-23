#!/usr/bin/env bash
set -euo pipefail
cargo test --locked --profile ci-opt -p vize_l2_to_l3 --features graph-queries
cargo test --locked --profile ci-opt -p vize_incremental --features resident-providers
cargo test --locked --manifest-path crates/vize_incremental/providers/graph/Cargo.toml --profile ci-opt
cargo clippy --locked --profile ci-opt -p vize_l3 -p vize_incremental --features vize_incremental/resident-providers,vize_l3/graph-queries --lib --no-deps -- -D warnings
export VIZE_IR_GRAPH_INPUTS="$RUNNER_TEMP/vue-inputs"
git ls-files -z -- '*.vue' '*.vue.txt' > "$VIZE_IR_GRAPH_INPUTS"
cargo test --locked --profile ci-opt -p vize_l2_to_l3 --features graph-queries --test native_ir_corpus -- --ignored --nocapture
cargo test --locked --manifest-path crates/vize_incremental/providers/graph/Cargo.toml --profile ci-opt --test source_revisions -- --ignored --nocapture
cargo run --locked --manifest-path crates/vize_incremental/providers/graph/Cargo.toml --profile ci-opt --example profile
