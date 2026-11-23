#!/usr/bin/env bash
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize --test integration -- content_mapper_tsgo_cli::dynamic_template_refs::native_cli_static_template_refs_keep_unused_and_shadow_diagnostics --exact --nocapture
