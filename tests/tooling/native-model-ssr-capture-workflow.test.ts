import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import { nativeSsrCaptureRequired } from "../../tools/support/compat/github/native-ssr-capture.mjs";
import { nativeSetupCaptureRequired } from "../../tools/support/compat/github/native-setup-capture.mjs";

test("shared genuine Model identity always qualifies both actual target consumers", () => {
  const shared = "davinci/vize_l3/src/decision/model.rs";
  assert(nativeSsrCaptureRequired([shared]));
  assert(nativeSetupCaptureRequired([shared]));
  for (const path of [
    "davinci/vize_l3/src/decision/ssr/model.rs",
    "davinci/vize_l3/src/decision/ssr/build/model.rs",
    "davinci/vize_l3/tests/native_model_ssr.rs",
    "davinci/vize_l4/src/targets/ssr/model.rs",
    "crates/vize_atelier_sfc/src/native_ssr_setup/tests/original_input_model.rs",
    "crates/vize_atelier_sfc/src/native_ssr_setup/tests/original_input_model/custody.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_inputs_vue_3_5_35.json",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_ssr_contexts_vue_3_5_35.json",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_ssr_sfc_vue_3_5_35.json",
    "tests/tooling/native-original-input-model-sfc-ssr-reference.test.ts",
    "tests/tooling/support/native-original-input-model-sfc-ssr-boundary.ts",
    "tests/tooling/support/native-original-input-model-sfc-ssr-primary.ts",
    "tests/tooling/support/native-original-input-model-sfc-ssr-runtime.ts",
  ])
    assert(nativeSsrCaptureRequired([path]), path);
  for (const path of [
    "davinci/vize_l3/tests/native_model_ssr_unrelated.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_ssr_contexts_vue_3_5_35.json.old",
    "tests/tooling/support/native-original-input-model-sfc-ssr-unrelated.ts",
    "docs/davinci/native-input-model-ssr.md",
  ])
    assert.equal(nativeSsrCaptureRequired([path]), false, path);
});

test("Model SSR retains every real attempt before its whole current verdict", () => {
  const action = parse(fs.readFileSync(".github/actions/test-native-ssr/action.yml", "utf8"));
  const step = action.runs.steps.find(
    (entry: { id?: string }) => entry.id === "native-input-model-ssr",
  );
  assert(step);
  assert.equal(step.if, "${{ !cancelled() }}");
  assert.equal(step["continue-on-error"], undefined);
  assert.equal(step.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_SSR_REQUIRE_CAPTURE, "1");
  for (const command of [
    "cargo test --locked --profile ci -p vize_l3 --test native_model_ssr -- --nocapture || model_l3_status=$?",
    "cargo test --locked --profile ci -p vize_atelier_sfc native_ssr_setup::tests::original_input_model::seven_whole_original_input_model_ssr_components_and_raw_maps_are_captured -- --exact --nocapture || model_rust_status=$?",
    "NODE_ENV=development vp node --test tests/tooling/native-original-input-model-sfc-ssr-reference.test.ts || model_development_status=$?",
    "NODE_ENV=production vp node --test tests/tooling/native-original-input-model-sfc-ssr-reference.test.ts || model_production_status=$?",
  ])
    assert(step.run.split("\n").includes(command), command);
  const verdict = step.run.indexOf('test "$model_l3_status" -eq 0');
  assert(verdict > step.run.indexOf("NODE_ENV=production"));
  assert(verdict > step.run.indexOf(".statuses.json"));
  const upload = action.runs.steps.find((entry: { uses?: string }) =>
    entry.uses?.startsWith("actions/upload-artifact@"),
  );
  assert(upload);
  assert.equal(upload.if, "${{ always() && steps.native-ssr.outcome != 'skipped' }}");
  for (const entry of [
    "capture.json",
    "capture.json.statuses.json",
    "primary.*.json",
    "runtime.*.json",
  ])
    assert(
      upload.with.path
        .split("\n")
        .includes(`\${{ runner.temp }}/native-original-input-model-sfc-ssr-${entry}`),
    );
});
