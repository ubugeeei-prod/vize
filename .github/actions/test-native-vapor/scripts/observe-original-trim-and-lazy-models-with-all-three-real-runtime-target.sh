#!/usr/bin/env bash
model_modifier_l1_status=0
cargo test --locked --profile ci -p vize_l1 --lib markup::htmlish::view::operand::model::tests:: -- --nocapture || model_modifier_l1_status=$?
model_modifier_l2_status=0
cargo test --locked --profile ci -p vize_l2 --lib file::region::native::model::tests:: -- --nocapture || model_modifier_l2_status=$?
model_modifier_l3_status=0
cargo test --locked --profile ci -p vize_l3 --test native_model_trim --test native_model_lazy -- --nocapture || model_modifier_l3_status=$?
model_modifier_rust_status=0
cargo test --locked --profile ci -p vize_atelier_sfc --lib native_selected_setup::tests::original_input_model_modifiers::six_original_models_retain_all_target_mode_attempts_before_verdict -- --exact --nocapture || model_modifier_rust_status=$?
model_modifier_dom_development_status=0
VIZE_NATIVE_MODEL_MODIFIERS_TARGET=dom NODE_ENV=development vp node --test tests/tooling/native-input-model-modifiers-reference.test.ts || model_modifier_dom_development_status=$?
model_modifier_dom_production_status=0
VIZE_NATIVE_MODEL_MODIFIERS_TARGET=dom NODE_ENV=production vp node --test tests/tooling/native-input-model-modifiers-reference.test.ts || model_modifier_dom_production_status=$?
model_modifier_ssr_development_status=0
VIZE_NATIVE_MODEL_MODIFIERS_TARGET=ssr NODE_ENV=development vp node --test tests/tooling/native-input-model-modifiers-reference.test.ts || model_modifier_ssr_development_status=$?
model_modifier_ssr_production_status=0
VIZE_NATIVE_MODEL_MODIFIERS_TARGET=ssr NODE_ENV=production vp node --test tests/tooling/native-input-model-modifiers-reference.test.ts || model_modifier_ssr_production_status=$?
model_modifier_vapor_development_status=0
VIZE_NATIVE_MODEL_MODIFIERS_TARGET=vapor NODE_ENV=development vp node --test tests/tooling/native-input-model-modifiers-reference.test.ts || model_modifier_vapor_development_status=$?
model_modifier_vapor_production_status=0
VIZE_NATIVE_MODEL_MODIFIERS_TARGET=vapor NODE_ENV=production vp node --test tests/tooling/native-input-model-modifiers-reference.test.ts || model_modifier_vapor_production_status=$?
printf '{"l1":%s,"l2":%s,"l3":%s,"rust":%s,"domDevelopment":%s,"domProduction":%s,"ssrDevelopment":%s,"ssrProduction":%s,"vaporDevelopment":%s,"vaporProduction":%s}\n' "$model_modifier_l1_status" "$model_modifier_l2_status" "$model_modifier_l3_status" "$model_modifier_rust_status" "$model_modifier_dom_development_status" "$model_modifier_dom_production_status" "$model_modifier_ssr_development_status" "$model_modifier_ssr_production_status" "$model_modifier_vapor_development_status" "$model_modifier_vapor_production_status" > "${VIZE_NATIVE_MODEL_MODIFIERS_CAPTURE}.statuses.json"
for status in "$model_modifier_l1_status" "$model_modifier_l2_status" "$model_modifier_l3_status" "$model_modifier_rust_status" "$model_modifier_dom_development_status" "$model_modifier_dom_production_status" "$model_modifier_ssr_development_status" "$model_modifier_ssr_production_status" "$model_modifier_vapor_development_status" "$model_modifier_vapor_production_status"; do
  test "$status" -eq 0
done
test -s "$VIZE_NATIVE_MODEL_MODIFIERS_CAPTURE"
for target in dom ssr vapor; do
  for mode in development production; do
    test -s "${VIZE_NATIVE_MODEL_MODIFIERS_PRIMARY_CAPTURE}.${target}.${mode}.json"
    test -s "${VIZE_NATIVE_MODEL_MODIFIERS_RUNTIME_CAPTURE}.${target}.${mode}.json"
  done
done
