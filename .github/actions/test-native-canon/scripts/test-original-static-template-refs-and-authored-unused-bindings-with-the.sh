#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-original-static-template-refs-and-authored-unused-bindings-with-the.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize --test integration -- content_mapper_tsgo_cli::dynamic_template_refs::native_cli_static_template_refs_keep_unused_and_shadow_diagnostics --exact --nocapture
