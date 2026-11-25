#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-complete-model-event-overload-contracts-with-the-native-checker.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib batch::type_checker::tests::generic_component_listener_payload::overload_depth::emit_overload_tail_accumulator_preserves_shapes_and_all_events -- --exact --nocapture
