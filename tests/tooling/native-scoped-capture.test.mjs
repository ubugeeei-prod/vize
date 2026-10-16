import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { nativeScopedCaptureRequired } from "../../tools/support/compat/github/native-scoped-capture.mjs";
import { nativeSsrCaptureRequired } from "../../tools/support/compat/github/native-ssr-capture.mjs";
import { planNativeRuntime } from "../../tools/support/compat/github/plan-native-runtime.ts";
import { assertNativeRuntimeHook } from "./support/native-runtime-workflow.ts";

const read = (path) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");

void test("every new original scoped dependency selects both source proofs even without ordinary tooling files", () => {
  for (const path of [
    "davinci/vize_l1/src/css.rs",
    "davinci/vize_l1/src/css/empty.rs",
    "davinci/vize_l1/src/css/parser.rs",
    "davinci/vize_l1_to_l2/src/native_file/selected_scoped.rs",
    "davinci/vize_l4/src/module/scope.rs",
    "crates/vize_atelier_sfc/src/native.rs",
    "crates/vize_atelier_sfc/src/native/styles/scoped.rs",
    "crates/vize_atelier_sfc/src/native/scope.rs",
    "crates/vize_atelier_sfc/src/native_scoped_ssr.rs",
    "crates/vize_atelier_sfc/tests/native_scoped_ssr.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native-scoped-ssr-vue-3.5.35.json",
    "tests/tooling/native-sfc-scoped-ssr-reference.test.ts",
    "tests/tooling/support/native-scoped-ssr-maps.ts",
    "tests/tooling/support/native-scoped-ssr-browser.ts",
    ".github/actions/test-native-scoped-css/action.yml",
    "tools/support/compat/github/native-scoped-capture.mjs",
  ]) {
    assert.equal(nativeScopedCaptureRequired([path]), true, path);
    const plan = planNativeRuntime([path]);
    assert(plan.run, path);
    assert(plan.matrix.lane.includes("ssr"), path);
    assert(plan.matrix.lane.includes("scoped-css"), path);
  }
});

void test("SSR input classification stays precise and affected changes select both runtime proofs", () => {
  for (const path of [
    "davinci/vize_l1_to_l2/src/native_file/selected.rs",
    "davinci/vize_l2/src/lang/js/file/native/scoped.rs",
    "davinci/vize_l3/src/decision/ssr/scoped.rs",
    "davinci/vize_l4/src/targets/ssr.rs",
    "crates/vize_atelier_sfc/src/native_ssr.rs",
  ]) {
    assert.equal(nativeSsrCaptureRequired([path]), true, path);
    const plan = planNativeRuntime([path]);
    assert(plan.run, path);
    assert(plan.matrix.lane.includes("ssr"), path);
    assert(plan.matrix.lane.includes("scoped-css"), path);
  }
  for (const path of [
    "README.md",
    "docs/davinci/decisions/2026-10-04-native-selected-scoped-ssr.md",
    "docs/davinci/plan/storage-inventory.tsv",
    "docs/davinci/plan/croquis-consumption/vize_atelier_sfc.md",
    "crates/vize_atelier_sfc/src/native_vapor.rs",
    "davinci/vize_l4/src/targets/dom.rs",
  ]) {
    assert.equal(nativeScopedCaptureRequired([path]), false, path);
    assert.equal(nativeSsrCaptureRequired([path]), false, path);
  }
  assert.equal(nativeSsrCaptureRequired([]), false);
  assert.equal(planNativeRuntime(["README.md"]).run, false);
});

void test("SSR and scoped CSS lanes require their complete runtime capture and source report", () => {
  assertNativeRuntimeHook("ssr", "test-native-ssr");
  assertNativeRuntimeHook("scoped-css", "test-native-scoped-css");
  const action = read(".github/actions/test-native-scoped-css/action.yml");
  assert(action.includes('VIZE_NATIVE_SCOPED_SSR_REQUIRE_NATIVE: "1"'));
  assert(
    action.includes(
      "--test native_scoped_ssr three_whole_original_scoped_ssr_modules_css_maps_and_file_custody_match -- --exact",
    ),
  );
  assert(action.includes("if: always()") && action.includes("if-no-files-found: error"));
});
