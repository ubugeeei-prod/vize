#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-repeated-loop-source-reads-and-public-instance-alias-shadowing.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib batch::type_checker::tests::recent_issues::setup_custom_properties::loop_sources_keep_authored_reads_and_parent_scope -- --exact --nocapture
