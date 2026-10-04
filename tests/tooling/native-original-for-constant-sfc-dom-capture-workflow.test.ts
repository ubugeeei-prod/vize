import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { nativeSetupCaptureRequired } from "../../tools/support/compat/github/native-setup-capture.mjs";

test("every constant source, emitter-only or full capture input requires a real fresh native setup packet", () => {
  for (const path of [
    "davinci/vize_l3/src/decision/dom/build/for_head/runtime.rs",
    "davinci/vize_l4/src/targets/dom/for_head.rs",
    "davinci/vize_l4/src/targets/dom/vue/for_head.rs",
    "crates/vize_atelier_sfc/src/native_selected_setup/tests/original_for_constant.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_for_constant_sfc_vue_3_5_35.json",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_for_constant_inherited_sfc_vue_3_5_35.json",
    "tests/tooling/native-original-for-constant-sfc-dom-reference.test.ts",
    "tests/tooling/native-original-for-constant-sfc-dom-capture-workflow.test.ts",
    "tests/tooling/support/native-original-for-constant-sfc-dom-runtime.ts",
    "tests/tooling/support/native-original-for-constant-sfc-dom-processes.ts",
  ])
    assert(nativeSetupCaptureRequired([path]), path);
  for (const path of [
    "docs/davinci/decisions/2026-10-04-original-for-constant-dom-proposal.md",
    "tests/tooling/support/native-original-for-constant-sfc-dom-runtime.md",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_for_constant_sfc_vue_3_5_35.json.notes",
    "davinci/vize_l4/src/targets/ssr/for_head.rs",
  ])
    assert.equal(nativeSetupCaptureRequired([path]), false, path);
});

test("actual constant action source builds all eleven whole modules before mandatory fresh dev and prod processes", () => {
  const action = fs.readFileSync(
    new URL("../../.github/actions/test-native-selected-sfc-dom/action.yml", import.meta.url),
    "utf8",
  );
  assert(action.includes('VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_REQUIRE_CAPTURE: "1"'));
  const rust = action.indexOf(
    "original_for_constant::eleven_whole_original_constant_for_components_and_raw_maps_are_captured -- --exact",
  );
  const runner = action.indexOf(
    "vp node tests/tooling/support/native-original-for-constant-sfc-dom-processes.ts",
  );
  assert(rust >= 0 && runner > rust);
  assert(!action.includes("continue-on-error"));
  for (const name of [
    "native-original-for-constant-sfc-dom-capture.json",
    "native-original-for-constant-sfc-dom-development-runtime.json",
    "native-original-for-constant-sfc-dom-production-runtime.json",
  ])
    assert.equal(action.split(name).length - 1, 2, name);
  const processes = fs.readFileSync(
    new URL("./support/native-original-for-constant-sfc-dom-processes.ts", import.meta.url),
    "utf8",
  );
  assert(processes.includes('["development", "production"]'));
  assert(
    processes.indexOf("attempts.push(receipt)") <
      processes.indexOf("for (const attempt of attempts)"),
  );
  for (const field of [
    "exitStatus",
    "signal",
    "processError",
    "stdoutBase64",
    "stderrBase64",
    "sourceCaptureHash",
  ])
    assert(processes.includes(field));
  assert(!processes.includes("retry" + "("));
  assert(action.includes("|| rust_status=$?"));
  assert(action.includes("|| node_status=$?"));
  assert(action.indexOf("rustStatus: Number") < action.indexOf('test "$rust_status" -eq 0'));
  for (const path of ["unqualified-attempts.json", "-*-unqualified-runtime.json"])
    assert(action.includes(path));
  const reference = fs.readFileSync(
    new URL("./native-original-for-constant-sfc-dom-reference.test.ts", import.meta.url),
    "utf8",
  );
  assert(!reference.includes("row?.code ?? fixture.expectedCode"));
  assert(
    reference.indexOf("const original = await executeConstantComponent") <
      reference.indexOf("if (!captured)"),
  );
  assert(reference.includes("attempts.push(attempt)"));
  assert(reference.indexOf("if (rawPath)") < reference.indexOf("assert.equal(executions.length"));
  assert(action.includes("if: ${{ always() }}"));
});
