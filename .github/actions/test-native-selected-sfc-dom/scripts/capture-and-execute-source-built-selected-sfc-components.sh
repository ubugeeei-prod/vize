#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-selected-sfc-dom/scripts/capture-and-execute-source-built-selected-sfc-components.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked --profile ci -p vize_atelier_sfc native_selected::tests::five_whole_original_click_components_and_maps_are_captured -- --exact
vp node --test tests/tooling/native-selected-sfc-dom-reference.test.ts
test -s "$VIZE_NATIVE_SELECTED_SFC_DOM_CAPTURE"
test -s "$VIZE_NATIVE_SELECTED_SFC_DOM_RUNTIME_CAPTURE"
