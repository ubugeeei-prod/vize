#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/content-mapper-conformance/run-content-mapper-lsp-and-virtual-overlay-editor-checks.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test -p vize --test integration -- content_mapper_tsgo_lsp:: --nocapture content_mapper_tsgo_lsp_hover::
cargo test -p vize --test integration -- content_mapper_lsp_shutdown:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_declaration_lsp:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_lsp_event_forms:: --nocapture
cargo test -p vize_canon --test lsp_import_resolution -- --nocapture
