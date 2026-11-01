import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { nativeVaporCaptureRequired } from "../../tools/support/compat/github/native-vapor-capture.mjs";

test("original Model and all Vapor consumers select the actual native lane", () => {
  for (const path of [
    "davinci/vize_l3/src/decision/model.rs",
    "davinci/vize_l3/src/decision/vapor/model.rs",
    "davinci/vize_l3/src/decision/vapor/build/model.rs",
    "davinci/vize_l3/tests/native_model_vapor.rs",
    "davinci/vize_l4/src/targets/vapor/model.rs",
    "crates/vize_atelier_sfc/src/native_vapor_setup/tests/original_input_model.rs",
    "crates/vize_atelier_sfc/src/native_vapor_setup/tests/original_input_model/custody.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_inputs_vue_3_5_35.json",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_ssr_contexts_vue_3_5_35.json",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_vapor_sfc_vue_3_6_rc9.json",
    "tests/tooling/native-original-input-model-vapor-reference.test.ts",
    "tests/tooling/support/native-original-input-model-vapor-boundary.ts",
    "tests/tooling/support/native-original-input-model-vapor-primary.ts",
    "tests/tooling/support/native-original-input-model-vapor-runtime.ts",
    "tests/tooling/support/native-original-input-model-vapor-observation.ts",
  ])
    assert(nativeVaporCaptureRequired([path]), path);
  for (const path of [
    "davinci/vize_l3/tests/native_model_vapor_unrelated.rs",
    "tests/tooling/support/native-original-input-model-vapor-unrelated.ts",
    "tests/tooling/native-original-input-model-vapor-reference.test.ts.old",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_inputs_vue_3_5_35.json.old",
    "docs/davinci/native-vapor-input-model.md",
  ])
    assert.equal(nativeVaporCaptureRequired([path]), false, path);
});

test("all four real attempts and six packets survive before a failing verdict", () => {
  const action = fs.readFileSync(
    new URL("../../.github/actions/test-native-vapor/action.yml", import.meta.url),
    "utf8",
  );
  const start = action.indexOf("    - name: Capture original input Models");
  const end = action.indexOf("    - name: Upload original Vapor", start);
  assert(start > 0 && end > start);
  const step = action.slice(start, end);
  assert(step.includes("if: ${{ !cancelled() }}"));
  assert(!step.includes("continue-on-error"));
  assert(step.includes('VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_REQUIRE_CAPTURE: "1"'));
  const commands = [
    "cargo test --locked --profile ci -p vize_l3 --test native_model_vapor -- --nocapture || model_l3_status=$?",
    "cargo test --locked --profile ci -p vize_atelier_sfc --lib native_vapor_setup::tests:: -- --nocapture || model_rust_status=$?",
    "NODE_ENV=development vp node --test tests/tooling/native-original-input-model-vapor-reference.test.ts || model_development_status=$?",
    "NODE_ENV=production vp node --test tests/tooling/native-original-input-model-vapor-reference.test.ts || model_production_status=$?",
  ];
  let previous = -1;
  for (const command of commands) {
    const position = step.indexOf(command);
    assert(position > previous, command);
    previous = position;
  }
  const statuses = step.indexOf(".statuses.json");
  const verdict = step.indexOf('test "$model_l3_status" -eq 0');
  assert(statuses > previous && verdict > statuses);
  for (const name of ["l3", "rust", "development", "production"])
    assert(step.includes(`test "$model_${name}_status" -eq 0`));
  const upload = action.slice(end);
  assert(upload.includes("if: ${{ always() }}"));
  assert(upload.includes("if-no-files-found: error"));
  for (const name of [
    "capture.json",
    "capture.json.statuses.json",
    "primary.development.json",
    "primary.production.json",
    "runtime.development.json",
    "runtime.production.json",
  ])
    assert(
      upload
        .split("\n")
        .includes(`          \${{ runner.temp }}/native-original-input-model-vapor-${name}`),
    );
});
