#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-typed-external-props-in-the-selected-legacy-dialect.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib batch::type_checker::tests::recent_issues::legacy_external_component_props:: -- --nocapture
cargo test --locked --profile ci -p vize_canon --lib virtual_ts::legacy_vue2_vuetify_tests:: -- --nocapture --test-threads=1
cargo test --locked --profile ci -p vize_canon --lib virtual_ts::scope::inline_callback_classifier:: -- --nocapture
