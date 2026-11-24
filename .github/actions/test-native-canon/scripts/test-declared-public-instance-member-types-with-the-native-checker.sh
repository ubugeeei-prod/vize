#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-declared-public-instance-member-types-with-the-native-checker.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib batch::type_checker::tests::recent_issues::setup_custom_properties::custom_property_member_types_and_native_type_queries_remain_exact -- --exact --nocapture
