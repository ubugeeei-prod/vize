#!/usr/bin/env bash
cargo test --locked --profile ci -p vize_canon --lib lsp_client::initial_diagnostics::
cargo test --locked --profile ci -p vize_canon --lib lsp_client::editor_lsp::snapshot_source::
cargo clippy --locked --profile ci -p vize_canon --lib -- -D warnings
