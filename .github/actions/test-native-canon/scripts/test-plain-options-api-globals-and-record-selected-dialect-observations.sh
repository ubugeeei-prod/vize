#!/usr/bin/env bash
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib batch::type_checker::tests::recent_issues::plain_options_global_augmentation::plain_options_module_augmentations_keep_modern_this_typed -- --exact --nocapture
