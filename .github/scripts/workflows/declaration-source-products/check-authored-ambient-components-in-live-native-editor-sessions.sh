#!/usr/bin/env bash
set -euo pipefail
"$VIZE_TEST_TSGO_PATH" --version
flags=(--locked --profile ci-opt -p vize_maestro --features experimental-ambient-project --lib)
cargo clippy --locked --profile ci-opt -p vize_canon --features ambient-project-declarations --lib --no-deps -- -D warnings
cargo clippy "${flags[@]}" --no-deps -- -D warnings
cargo test --locked --profile ci-opt -p vize_canon --features ambient-project-declarations --lib virtual_project::ambient_declarations::
cargo test --locked --profile ci-opt -p vize_canon --features ambient-project-declarations --lib tracked_documents::tests::
cargo test "${flags[@]}" corsa::ambient_project:: -- --include-ignored --nocapture --test-threads=1
