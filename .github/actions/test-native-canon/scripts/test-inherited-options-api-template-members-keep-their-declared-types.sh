#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-inherited-options-api-template-members-keep-their-declared-types.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --features cache-custody --lib batch::type_checker::tests::recent_issues::options_api_inherited_template_members:: -- --nocapture
cargo test --locked --profile ci -p vize_canon --lib virtual_ts::scope::globals::instance::tests::options_api_inherited_members_are_declared_from_the_authored_instance -- --exact --nocapture
cargo test --locked --profile ci -p vize_canon --lib virtual_ts::tests::inherited_options::test_options_api_unresolved_extends_declares_template_refs_from_the_instance -- --exact --nocapture
cargo test --locked --profile ci -p vize_canon --lib virtual_ts::tests::inherited_calls:: -- --nocapture
