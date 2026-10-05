#!/usr/bin/env bash
set -euo pipefail
cargo test --locked --profile ci-opt -p vize_canon --lib nested_generated_config:: -- --nocapture
cargo test --locked --profile ci-opt -p vize_canon --test tier_l_incremental failure::tests:: -- --nocapture
export VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR="${VIZE_NESTED_BATCH_CAPTURE_DIR:-target/vize-tests/metrics}/snapshot-source"
mkdir -p "$VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR"
cargo test --locked --profile ci-opt -p vize_canon --doc SnapshotSourceText -- --nocapture | tee "$VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR/doctests.log"
rg -F 'test result: ok. 3 passed; 0 failed;' "$VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR/doctests.log"
cargo test --locked --profile ci-opt -p vize_canon --lib lsp_client::editor_lsp::snapshot_source:: -- --include-ignored --nocapture
if [[ -n "${VIZE_NESTED_BATCH_CAPTURE_DIR:-}" ]]; then
  export VIZE_NATIVE_BULK_CAPTURE_DIR="$VIZE_NESTED_BATCH_CAPTURE_DIR/bulk-native"
fi
cargo test --locked --profile ci-opt -p vize_canon --lib lsp_client::editor_lsp::bulk_diagnostics::tests:: -- --nocapture
cargo test --locked -p vize_l1_to_l2 --features legacy-differential --test davinci_lowering_corpus -- --nocapture
cargo test --locked -p vize_l1_to_l2 --features legacy-differential --test davinci_dom_corpus -- --nocapture
cargo test --locked -p vize_l1_to_l2 --features legacy-differential --test davinci_remarks_corpus -- --nocapture
cargo test --locked -p vize_canon --features legacy-differential --test davinci_projection_differential -- --nocapture
