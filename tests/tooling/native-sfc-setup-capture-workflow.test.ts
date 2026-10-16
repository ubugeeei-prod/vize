import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { nativeSetupCaptureRequired } from "../../tools/support/compat/github/native-setup-capture.mjs";
import { planNativeRuntime } from "../../tools/support/compat/github/plan-native-runtime.ts";
import { assertNativeRuntimeHook } from "./support/native-runtime-workflow.ts";

test("actual native setup inputs qualify affected source capture while prose stays fast", () => {
  for (const path of [
    "davinci/vize_l2/src/file.rs",
    "davinci/vize_l2/src/file/build.rs",
    "davinci/vize_l2/src/lang/js/file/walk/annotations.rs",
    "davinci/vize_l4/src/module/setup.rs",
    "davinci/vize_l4/src/targets/dom/vue.rs",
    "crates/vize_atelier_sfc/src/native.rs",
    "crates/vize_atelier_sfc/src/native/setup/tests/annotations.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_sfc_ts_annotation_setup_vue_3_5_35.json",
    "crates/vize_atelier_sfc/tests/fixtures/native_sfc_ordinary_empty_vue_3_5_35.json",
    "tests/tooling/native-sfc-ordinary-empty-reference.test.ts",
    "tests/tooling/support/native-sfc-ordinary-reference.ts",
    "tests/tooling/native-sfc-primitive-setup-reference.test.ts",
    "tests/tooling/support/native-sfc-setup-reference.ts",
    ".github/actions/test-native-js-setup/action.yml",
    "vendor/oxc_parser/src/lexer/string.rs",
  ]) {
    assert(nativeSetupCaptureRequired([path]), path);
    assert(planNativeRuntime([path]).matrix.lane.includes("setup"), path);
  }
  for (const path of [
    "docs/davinci/plan/completion-2026-10-03.md",
    "docs/davinci/decisions/2026-09-27-level-restructure.md",
    "README.md",
    "crates/vize_canon/src/native.rs",
    "davinci/vize_l2/tests/file_vue_setup.rs",
    "npm/ui/src/index.ts",
  ])
    assert.equal(nativeSetupCaptureRequired([path]), false, path);
  assert.equal(nativeSetupCaptureRequired([]), false);
});

test("a capture-only source change selects setup independently of ordinary tooling suites", () => {
  const plan = planNativeRuntime(["davinci/vize_l2/src/lang/js/file/setup.rs"]);
  assert(plan.run);
  assert(plan.matrix.lane.includes("setup"));
  assert.deepEqual(planNativeRuntime(["docs/davinci/plan/completion-2026-10-03.md"]), {
    run: false,
    matrix: { lane: ["none"] },
  });
});

test("the required setup lane keeps complete capture and exact source artifacts", () => {
  assertNativeRuntimeHook("setup", "test-native-js-setup");
  const action = fs.readFileSync(
    new URL("../../.github/actions/test-native-js-setup/action.yml", import.meta.url),
    "utf8",
  );
  assert.match(action, /cargo test --locked --profile ci -p vize_atelier_sfc/);
  assert.match(action, /three_source_owned_annotation_modules_and_maps_are_captured -- --exact/);
  assert.match(action, /three_whole_ordinary_empty_modules_and_maps_are_captured -- --exact/);
  assert.match(action, /VIZE_NATIVE_SFC_ORDINARY_EMPTY_CAPTURE: \$\{\{ runner\.temp \}\}/);
  assert.match(action, /native-sfc-ordinary-empty-reference\.test\.ts/);
  assert.match(action, /native-js-setup-\$\{\{ github\.sha \}\}-\$\{\{ github\.run_attempt \}\}/);
  assert.match(action, /if-no-files-found: error/);
  assert.doesNotMatch(action, /continue-on-error|hashFiles|existsSync/);
});
