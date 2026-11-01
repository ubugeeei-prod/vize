import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { nativeVaporCaptureRequired } from "../../tools/support/compat/github/native-vapor-capture.mjs";

test("number producer, inputs and all three consumers select genuine hosted capture", () => {
  for (const path of [
    "davinci/vize_l3/tests/native_model_number/mod.rs",
    ...["dom", "ssr", "vapor"].map((target) => `davinci/vize_l4/src/targets/${target}/model.rs`),
    "crates/vize_atelier_sfc/src/native_selected_setup/tests.rs",
    "crates/vize_atelier_sfc/src/native_selected_setup/tests/original_input_model_number.rs",
    ...["custody", "output", "refusal", "finish"].map(
      (name) =>
        `crates/vize_atelier_sfc/src/native_selected_setup/tests/original_input_model_number/${name}.rs`,
    ),
    ...["inputs", "refusal_inputs", "all_targets"].map(
      (name) =>
        `crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_number_${name}.json`,
    ),
    "tests/tooling/native-input-model-number-reference.test.ts",
    "tests/tooling/native-input-model-number-capture-workflow.test.ts",
    "tests/tooling/native-input-model-number-order.test.ts",
    "tests/tooling/support/native-input-model-number-order.ts",
    ...["boundary", "dom-runtime", "vapor-runtime"].map(
      (name) => `tests/tooling/support/native-input-model-number-${name}.ts`,
    ),
  ])
    assert.equal(nativeVaporCaptureRequired([path]), true, path);
  for (const path of [
    "davinci/vize_l3/tests/native_model_numbers.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_number_inputs.json.old",
    "tests/tooling/support/native-input-model-number-unknown.ts",
    "docs/davinci/model-number.md",
  ])
    assert.equal(nativeVaporCaptureRequired([path]), false, path);
});

test("number capture retains every phase before verdict and keeps old six-source replay", () => {
  const action = fs.readFileSync(
    new URL("../../.github/actions/test-native-vapor/action.yml", import.meta.url),
    "utf8",
  );
  const start = action.indexOf("    - name: Observe original number Models");
  const end = action.indexOf("    - name: Observe original trim and lazy Models", start);
  const uploadStart = action.indexOf("    - name: Upload original Vapor", end);
  assert(start > 0 && end > start && uploadStart > end);
  const step = action.slice(start, end);
  assert(step.includes("if: ${{ !cancelled() }}"));
  assert(!step.includes("continue-on-error"));
  assert(step.includes('VIZE_NATIVE_MODEL_NUMBER_REQUIRE_CAPTURE: "1"'));
  const statuses = step.indexOf(".statuses.json");
  const verdict = step.indexOf('test "$status" -eq 0');
  let previous = -1;
  for (const phase of [
    "l1",
    "l2",
    "l3",
    "rust",
    "dom_development",
    "dom_production",
    "ssr_development",
    "ssr_production",
    "vapor_development",
    "vapor_production",
  ]) {
    const attempt = step.indexOf(`|| model_number_${phase}_status=$?`);
    assert(attempt > previous && attempt < statuses && statuses < verdict, phase);
    previous = attempt;
  }
  const oldStep = action.slice(
    end,
    action.indexOf("    - name: Capture original trim Models", end),
  );
  assert(oldStep.includes('VIZE_NATIVE_MODEL_MODIFIERS_REQUIRE_CAPTURE: "1"'));
  assert(
    oldStep.includes(
      "six_original_models_retain_all_target_mode_attempts_before_verdict -- --exact --nocapture",
    ),
  );
  const upload = action.slice(uploadStart);
  assert(upload.includes("if: ${{ always() }}"));
  for (const suffix of [
    "capture.json",
    "capture.json.statuses.json",
    "primary.*.json",
    "runtime.*.json",
  ]) {
    assert(
      upload
        .split("\n")
        .includes(`          \${{ runner.temp }}/native-original-input-model-number-${suffix}`),
    );
  }
});
