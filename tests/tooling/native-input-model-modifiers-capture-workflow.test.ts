import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { nativeVaporCaptureRequired } from "../../tools/support/compat/github/native-vapor-capture.mjs";

test("modifier inputs, native producers, and all real consumer leaves select hosted capture", () => {
  for (const path of [
    "davinci/vize_l3/tests/native_model_lazy.rs",
    "crates/vize_atelier_sfc/src/native_selected_setup/tests/original_input_model_modifiers.rs",
    ...["custody", "output", "outside", "refusal", "finish"].map(
      (name) =>
        `crates/vize_atelier_sfc/src/native_selected_setup/tests/original_input_model_modifiers/${name}.rs`,
    ),
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_modifier_inputs.json",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_modifiers_all_targets.json",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_current_outside.json",
    "crates/vize_atelier_sfc/src/native_selected_setup/tests/current_model_outside.rs",
    "tests/tooling/native-input-model-modifiers-reference.test.ts",
    "tests/tooling/native-input-model-modifiers-capture-workflow.test.ts",
    "tests/tooling/support/native-input-model-lazy-dom-runtime.ts",
    "tests/tooling/support/native-input-model-lazy-vapor-runtime.ts",
    "tests/tooling/support/native-input-model-modifiers-outside.ts",
    "tests/tooling/support/native-input-model-current-outside.ts",
    "tests/tooling/native-original-input-model-sfc-dom-reference.test.ts",
    "tests/tooling/native-original-input-model-sfc-ssr-reference.test.ts",
  ])
    assert.equal(nativeVaporCaptureRequired([path]), true, path);
  for (const path of [
    "davinci/vize_l3/tests/native_model_lazy_old.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_modifier_inputs.json.old",
    "tests/tooling/support/native-input-model-lazy-unknown.ts",
    "docs/davinci/model-lazy.md",
  ])
    assert.equal(nativeVaporCaptureRequired([path]), false, path);
});

test("the initial observation retains all ten statuses before verdict and uploads on failure", () => {
  const action = fs.readFileSync(
    new URL("../../.github/actions/test-native-vapor/action.yml", import.meta.url),
    "utf8",
  );
  const start = action.indexOf("    - name: Observe original trim and lazy Models");
  const end = action.indexOf("    - name: Capture original trim Models", start);
  const uploadStart = action.indexOf("    - name: Upload original Vapor", end);
  assert(start > 0 && end > start && uploadStart > end);
  const step = action.slice(start, end);
  assert(step.includes("if: ${{ !cancelled() }}"));
  assert(!step.includes("continue-on-error"));
  assert(step.includes('VIZE_NATIVE_MODEL_MODIFIERS_REQUIRE_CAPTURE: "1"'));
  const phases = [
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
  ];
  const statuses = step.indexOf(".statuses.json");
  const verdict = step.indexOf('test "$status" -eq 0');
  let previous = -1;
  for (const phase of phases) {
    const attempt = step.indexOf(`|| model_modifier_${phase}_status=$?`);
    assert(attempt > previous && attempt < statuses && statuses < verdict, phase);
    previous = attempt;
  }
  const upload = action.slice(uploadStart);
  assert(upload.includes("if: ${{ always() }}"));
  assert(upload.includes("if-no-files-found: error"));
  for (const suffix of [
    "capture.json",
    "capture.json.statuses.json",
    "primary.*.json",
    "runtime.*.json",
  ])
    assert(
      upload
        .split("\n")
        .includes(`          \${{ runner.temp }}/native-original-input-model-modifiers-${suffix}`),
    );
});
