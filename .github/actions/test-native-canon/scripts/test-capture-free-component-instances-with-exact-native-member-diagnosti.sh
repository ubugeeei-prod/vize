#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-canon/scripts/test-capture-free-component-instances-with-exact-native-member-diagnosti.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
unset VIZE_TEST_DISABLE_TSGO
cargo test --locked --profile ci -p vize_canon --lib batch::type_checker::tests::recent_issues::component_render_h_props:: -- --nocapture
