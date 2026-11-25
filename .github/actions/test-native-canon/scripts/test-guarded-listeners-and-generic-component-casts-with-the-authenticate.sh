#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-guarded-listeners-and-generic-component-casts-with-the-authenticate.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
set -euo pipefail
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize --test integration -- content_mapper_tsgo_cli::nested_function_guard_11853::standard_tsgo_nested_function_guards_keep_listener_reads_and_real_errors --ignored --exact --nocapture
cargo test --locked --profile ci -p vize --test integration -- content_mapper_tsgo_cli::generic_component_cast_11966::standard_tsgo_accepts_generic_slots_and_emits_component_cast --ignored --exact --nocapture
cargo test --locked --profile ci -p vize --test integration -- content_mapper_tsgo_cli::vfor_const_listener_narrowing_11973::standard_tsgo_keeps_v_for_const_narrowing_in_listener_callbacks content_mapper_tsgo_cli::replayed_pattern_duplicates::standard_tsgo_reports_each_replayed_pattern_diagnostic_once --ignored --exact --nocapture
