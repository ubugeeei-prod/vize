#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/actions/test-native-selected-sfc-dom/scripts/capture-new-original-for-key-components-and-execute-both-primary-runtime.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
key_rust_status=0
cargo test --locked --profile ci -p vize_atelier_sfc native_selected_setup::tests::original_for_key::nine_whole_original_key_sources_capture_real_bindings_and_maps -- --exact --nocapture || key_rust_status=$?
key_development_status=0
NODE_ENV=development vp node --test tests/tooling/native-original-for-key-sfc-dom-reference.test.ts || key_development_status=$?
key_production_status=0
NODE_ENV=production vp node --test tests/tooling/native-original-for-key-sfc-dom-reference.test.ts || key_production_status=$?
printf '{"rust":%s,"development":%s,"production":%s}\n' "$key_rust_status" "$key_development_status" "$key_production_status" > "${VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_CAPTURE}.statuses.json"
test "$key_rust_status" -eq 0
test "$key_development_status" -eq 0
test "$key_production_status" -eq 0
test -s "$VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_CAPTURE"
test -s "${VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_PRIMARY_CAPTURE}.development.json"
test -s "${VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_PRIMARY_CAPTURE}.production.json"
test -s "${VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_RUNTIME_CAPTURE}.development.json"
test -s "${VIZE_NATIVE_ORIGINAL_FOR_KEY_SFC_DOM_RUNTIME_CAPTURE}.production.json"
