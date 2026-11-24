#!/usr/bin/env bash
model_checkbox_l1_status=0
cargo test --locked --profile ci -p vize_l1 --lib markup::htmlish::view::operand::model::tests:: -- --nocapture || model_checkbox_l1_status=$?
model_checkbox_l2_status=0
cargo test --locked --profile ci -p vize_l2 --lib file::region::native::model::tests::checkbox:: -- --nocapture || model_checkbox_l2_status=$?
model_checkbox_l3_status=0
cargo test --locked --profile ci -p vize_l3 --test native_model_lazy native_model_checkbox:: -- --nocapture || model_checkbox_l3_status=$?
model_checkbox_rust_status=0
cargo test --locked --profile ci -p vize_atelier_sfc --lib native_selected_setup::tests::original_input_model_checkbox::three_original_checkbox_models_retain_all_target_mode_attempts_before_verdict -- --exact --nocapture || model_checkbox_rust_status=$?
model_checkbox_dom_development_status=0
VIZE_NATIVE_MODEL_CHECKBOX_TARGET=dom NODE_ENV=development vp node --test tests/tooling/native-input-model-checkbox-reference.test.ts || model_checkbox_dom_development_status=$?
model_checkbox_dom_production_status=0
VIZE_NATIVE_MODEL_CHECKBOX_TARGET=dom NODE_ENV=production vp node --test tests/tooling/native-input-model-checkbox-reference.test.ts || model_checkbox_dom_production_status=$?
model_checkbox_ssr_development_status=0
VIZE_NATIVE_MODEL_CHECKBOX_TARGET=ssr NODE_ENV=development vp node --test tests/tooling/native-input-model-checkbox-reference.test.ts || model_checkbox_ssr_development_status=$?
model_checkbox_ssr_production_status=0
VIZE_NATIVE_MODEL_CHECKBOX_TARGET=ssr NODE_ENV=production vp node --test tests/tooling/native-input-model-checkbox-reference.test.ts || model_checkbox_ssr_production_status=$?
model_checkbox_vapor_development_status=0
VIZE_NATIVE_MODEL_CHECKBOX_TARGET=vapor NODE_ENV=development vp node --test tests/tooling/native-input-model-checkbox-reference.test.ts || model_checkbox_vapor_development_status=$?
model_checkbox_vapor_production_status=0
VIZE_NATIVE_MODEL_CHECKBOX_TARGET=vapor NODE_ENV=production vp node --test tests/tooling/native-input-model-checkbox-reference.test.ts || model_checkbox_vapor_production_status=$?
printf '{"l1":%s,"l2":%s,"l3":%s,"rust":%s,"domDevelopment":%s,"domProduction":%s,"ssrDevelopment":%s,"ssrProduction":%s,"vaporDevelopment":%s,"vaporProduction":%s}\n' "$model_checkbox_l1_status" "$model_checkbox_l2_status" "$model_checkbox_l3_status" "$model_checkbox_rust_status" "$model_checkbox_dom_development_status" "$model_checkbox_dom_production_status" "$model_checkbox_ssr_development_status" "$model_checkbox_ssr_production_status" "$model_checkbox_vapor_development_status" "$model_checkbox_vapor_production_status" > "${VIZE_NATIVE_MODEL_CHECKBOX_CAPTURE}.statuses.json"
for status in "$model_checkbox_l1_status" "$model_checkbox_l2_status" "$model_checkbox_l3_status" "$model_checkbox_rust_status" "$model_checkbox_dom_development_status" "$model_checkbox_dom_production_status" "$model_checkbox_ssr_development_status" "$model_checkbox_ssr_production_status" "$model_checkbox_vapor_development_status" "$model_checkbox_vapor_production_status"; do
  test "$status" -eq 0
done
test -s "$VIZE_NATIVE_MODEL_CHECKBOX_CAPTURE"
for target in dom ssr vapor; do
  for mode in development production; do
    test -s "${VIZE_NATIVE_MODEL_CHECKBOX_PRIMARY_CAPTURE}.${target}.${mode}.json"
    test -s "${VIZE_NATIVE_MODEL_CHECKBOX_RUNTIME_CAPTURE}.${target}.${mode}.json"
  done
done
