#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-selected-sfc-dom/scripts/capture-and-execute-whole-original-mutable-for-components.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
cargo test --locked --profile ci -p vize_atelier_sfc native_selected_setup::tests::original_for::ten_whole_original_for_components_and_maps_are_captured -- --exact
vp node --test tests/tooling/native-original-for-sfc-dom-reference.test.ts
test -s "$VIZE_NATIVE_ORIGINAL_FOR_SFC_DOM_CAPTURE"
test -s "$VIZE_NATIVE_ORIGINAL_FOR_SFC_DOM_RUNTIME_CAPTURE"
