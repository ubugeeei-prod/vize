#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-typed-legacy-mixin-methods-without-erasing-native-findings.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib batch::type_checker::tests::recent_issues::options_api_inherited_members:: -- --nocapture
cargo test --locked --profile ci -p vize_canon --features legacy --lib batch::type_checker::tests::recent_issues::legacy_js_mixin_receiver:: -- --nocapture
cargo test --locked --profile ci -p vize_canon --lib virtual_ts::generator::options_api_bridge::inherited::tests::native_mixin_method_laws_preserve_opaque_and_constructor_types -- --exact --nocapture
