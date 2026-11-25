#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-selected-sfc-dom/scripts/capture-genuine-adjacent-root-if-else-components-in-both-explicit-modes.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
root_if_else_rust_status=0
cargo test --locked --profile ci -p vize_atelier_sfc native_selected_setup::tests::original_root_if_else::twelve_whole_original_root_if_else_components_and_raw_maps_are_captured -- --exact --nocapture || root_if_else_rust_status=$?
root_if_else_development_status=0
NODE_ENV=development vp node --test tests/tooling/native-original-root-if-else-sfc-dom-reference.test.ts || root_if_else_development_status=$?
root_if_else_production_status=0
NODE_ENV=production vp node --test tests/tooling/native-original-root-if-else-sfc-dom-reference.test.ts || root_if_else_production_status=$?
printf '{"rust":%s,"development":%s,"production":%s}\n' "$root_if_else_rust_status" "$root_if_else_development_status" "$root_if_else_production_status" > "${VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_SFC_DOM_CAPTURE}.statuses.json"
test "$root_if_else_rust_status" -eq 0
test "$root_if_else_development_status" -eq 0
test "$root_if_else_production_status" -eq 0
test -s "$VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_SFC_DOM_CAPTURE"
test -s "${VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_SFC_DOM_PRIMARY_CAPTURE}.development.json"
test -s "${VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_SFC_DOM_PRIMARY_CAPTURE}.production.json"
test -s "${VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_SFC_DOM_RUNTIME_CAPTURE}.development.json"
test -s "${VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_SFC_DOM_RUNTIME_CAPTURE}.production.json"
