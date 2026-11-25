#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-scoped-css/scripts/capture-scoped-modules-from-the-current-rust-source-and-mount-real-vue.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
git rev-parse HEAD > "$RUNNER_TEMP/native-scoped-css-source.txt"
cargo test --locked --profile ci -p vize_atelier_sfc native::scoped_tests::four_source_bound_scoped_modules_css_and_default_identity_match_complete_contracts -- --exact
cargo test --locked --profile ci -p vize_atelier_sfc native::class_list_tests::three_original_class_list_sfc_modules_css_maps_and_scope_id_match_whole_contracts -- --exact
cargo test --locked --profile ci -p vize_atelier_sfc native::scoped_empty_tests::three_original_empty_sfc_modules_css_maps_and_scope_id_match_whole_contracts -- --exact
cargo test --locked --profile ci -p vize_atelier_sfc --test native_scoped_ssr three_whole_original_scoped_ssr_modules_css_maps_and_file_custody_match -- --exact
vp node --test tests/tooling/native-sfc-scoped-css-reference.test.ts
vp node --test tests/tooling/native-sfc-scoped-class-list-reference.test.ts
vp node --test tests/tooling/native-sfc-scoped-empty-reference.test.ts
vp node --test tests/tooling/native-sfc-scoped-ssr-reference.test.ts
cargo test --locked --profile ci -p vize_atelier_sfc --test css_scope_body
vp node --test tests/tooling/scope-body-browser.test.ts
