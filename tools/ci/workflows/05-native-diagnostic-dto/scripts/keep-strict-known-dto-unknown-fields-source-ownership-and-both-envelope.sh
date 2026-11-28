#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/05-native-diagnostic-dto/scripts/keep-strict-known-dto-unknown-fields-source-ownership-and-both-envelope.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked --profile ci -p vize_canon --lib lsp_client::initial_diagnostics::
cargo test --locked --profile ci -p vize_canon --lib lsp_client::editor_lsp::snapshot_source::
cargo clippy --locked --profile ci -p vize_canon --lib -- -D warnings
