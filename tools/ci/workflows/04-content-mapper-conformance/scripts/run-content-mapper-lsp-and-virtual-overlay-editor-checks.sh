#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/04-content-mapper-conformance/scripts/run-content-mapper-lsp-and-virtual-overlay-editor-checks.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test -p vize --test integration -- content_mapper_tsgo_lsp:: --nocapture content_mapper_tsgo_lsp_hover::
cargo test -p vize --test integration -- content_mapper_lsp_shutdown:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_declaration_lsp:: --nocapture
cargo test -p vize --test integration -- content_mapper_tsgo_lsp_event_forms:: --nocapture
cargo test -p vize_canon --test lsp_import_resolution -- --nocapture
# These package-condition and stale/cancel laws use the installed production
# native provider, independently of the upstream Content Mapper host above.
env -u VIZE_TEST_DISABLE_TSGO \
  VIZE_TEST_REQUIRE_TSGO=1 \
  VIZE_TEST_TSGO_PATH="$GITHUB_WORKSPACE/node_modules/@typescript/typescript-linux-x64/lib/tsc" \
  cargo test -p vize_maestro --lib -- \
    ide::definition::component_import::native_target::tests:: \
    server::native_requests::tests::component_targets:: --nocapture
