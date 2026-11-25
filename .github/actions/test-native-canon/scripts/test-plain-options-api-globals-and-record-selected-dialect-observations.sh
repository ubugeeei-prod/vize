#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-plain-options-api-globals-and-record-selected-dialect-observations.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib batch::type_checker::tests::recent_issues::plain_options_global_augmentation::plain_options_module_augmentations_keep_modern_this_typed -- --exact --nocapture
