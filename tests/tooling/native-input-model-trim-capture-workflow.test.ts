import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { nativeVaporCaptureRequired } from "../../tools/support/compat/github/native-vapor-capture.mjs";

test("all original trim source and consumer inputs select the actual runtime lane", () => {
  for (const path of [
    "davinci/vize_l3/tests/native_model_trim.rs",
    "crates/vize_atelier_sfc/tests/native_input_model_trim/main.rs",
    "crates/vize_atelier_sfc/tests/native_input_model_trim/custody.rs",
    "crates/vize_atelier_sfc/tests/native_input_model_trim/output.rs",
    "crates/vize_atelier_sfc/tests/native_input_model_trim/refusal.rs",
    "crates/vize_atelier_sfc/tests/native_input_model_trim/finish.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_trim_inputs.json",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_trim_all_targets.json",
    "tests/tooling/native-input-model-trim-reference.test.ts",
    "tests/tooling/native-input-model-trim-capture-workflow.test.ts",
    "tests/tooling/support/native-input-model-trim-boundary.ts",
    "tests/tooling/support/native-input-model-trim-dom-runtime.ts",
    "tests/tooling/support/native-input-model-trim-vapor-runtime.ts",
    "tests/tooling/support/native-input-model-trim-vapor-module.ts",
  ])
    assert(nativeVaporCaptureRequired([path]), path);
  for (const path of [
    "davinci/vize_l3/tests/native_model_trim_old.rs",
    "crates/vize_atelier_sfc/tests/native_input_model_trim/unknown.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_trim_inputs.json.old",
    "tests/tooling/support/native-input-model-trim-unknown.ts",
    "docs/davinci/native-model-trim.md",
  ])
    assert.equal(nativeVaporCaptureRequired([path]), false, path);
});

test("all ten attempts and fourteen raw packets precede the actual failing verdict", () => {
  const action = fs.readFileSync(
    new URL("../../.github/actions/test-native-vapor/action.yml", import.meta.url),
    "utf8",
  );
  const start = action.indexOf("    - name: Capture original trim Models");
  const end = action.indexOf("    - name: Upload original Vapor", start);
  assert(start > 0 && end > start);
  const step = action.slice(start, end);
  assert(step.includes("if: ${{ !cancelled() }}"));
  assert(!step.includes("continue-on-error"));
  assert(step.includes('VIZE_NATIVE_MODEL_TRIM_REQUIRE_CAPTURE: "1"'));
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
  let previous = -1;
  for (const phase of phases) {
    const attempt = step.indexOf(`|| model_trim_${phase}_status=$?`);
    const verdict = step.indexOf(`test "$model_trim_${phase}_status" -eq 0`);
    assert(attempt > previous && statuses > attempt && verdict > statuses, phase);
    previous = attempt;
  }
  const upload = action.slice(end);
  assert(upload.includes("if: ${{ always() }}"));
  assert(upload.includes("if-no-files-found: error"));
  for (const name of [
    "capture.json",
    "capture.json.statuses.json",
    "primary.*.json",
    "runtime.*.json",
  ])
    assert(
      upload
        .split("\n")
        .includes(`          \${{ runner.temp }}/native-original-input-model-trim-${name}`),
    );
});
