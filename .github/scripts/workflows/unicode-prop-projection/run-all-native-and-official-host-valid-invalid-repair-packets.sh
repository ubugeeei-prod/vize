#!/usr/bin/env bash
cargo test --locked --profile ci -p vize_canon --lib virtual_ts::props::mappings::tests:: -- --nocapture
cargo test --locked --profile ci -p vize_canon --test fix_history_diagnostics unicode_prop_bindings_preserve_complete_authored_diagnostics -- --exact --nocapture
cargo test --locked --profile ci -p vize --test integration -- content_mapper_tsgo_cli::unicode_prop_bindings:: --nocapture --test-threads=1
