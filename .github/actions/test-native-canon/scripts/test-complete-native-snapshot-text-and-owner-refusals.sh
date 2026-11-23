#!/usr/bin/env bash
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib lsp_client::editor_lsp::snapshot_source::tests::native::actual_native_snapshot_retains_complete_stdlib_and_overlay_text_with_owner_refusals -- --ignored --exact --nocapture
