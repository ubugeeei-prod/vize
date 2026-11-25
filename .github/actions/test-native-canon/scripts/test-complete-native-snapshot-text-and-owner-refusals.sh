#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-complete-native-snapshot-text-and-owner-refusals.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib lsp_client::editor_lsp::snapshot_source::tests::native::actual_native_snapshot_retains_complete_stdlib_and_overlay_text_with_owner_refusals -- --ignored --exact --nocapture
