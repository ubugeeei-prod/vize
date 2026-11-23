#!/usr/bin/env bash
cargo test -p vize --test integration -- content_mapper_tsgo_lsp:: --nocapture content_mapper_tsgo_lsp_hover::
cargo test -p vize --test integration -- content_mapper_lsp_shutdown:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_declaration_lsp:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_lsp_event_forms:: --nocapture
cargo test -p vize_canon --test lsp_import_resolution -- --nocapture
