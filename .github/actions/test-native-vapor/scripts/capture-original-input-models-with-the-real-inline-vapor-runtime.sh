#!/usr/bin/env bash
model_l3_status=0
cargo test --locked --profile ci -p vize_l3 --test native_model_vapor -- --nocapture || model_l3_status=$?
model_rust_status=0
cargo test --locked --profile ci -p vize_atelier_sfc --lib native_vapor_setup::tests:: -- --nocapture || model_rust_status=$?
model_development_status=0
NODE_ENV=development vp node --test tests/tooling/native-original-input-model-vapor-reference.test.ts || model_development_status=$?
model_production_status=0
NODE_ENV=production vp node --test tests/tooling/native-original-input-model-vapor-reference.test.ts || model_production_status=$?
printf '{"l3":%s,"rust":%s,"development":%s,"production":%s}\n' "$model_l3_status" "$model_rust_status" "$model_development_status" "$model_production_status" > "${VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_CAPTURE}.statuses.json"
test "$model_l3_status" -eq 0
test "$model_rust_status" -eq 0
test "$model_development_status" -eq 0
test "$model_production_status" -eq 0
test -s "$VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_CAPTURE"
test -s "${VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_PRIMARY_CAPTURE}.development.json"
test -s "${VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_PRIMARY_CAPTURE}.production.json"
test -s "${VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_RUNTIME_CAPTURE}.development.json"
test -s "${VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_RUNTIME_CAPTURE}.production.json"
