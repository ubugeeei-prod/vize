import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  nativeSetupCaptureRequired,
  toolingChecksRequired,
} from "../../tools/support/compat/github/native-setup-capture.mjs";
import { toolingShardMatrix } from "../../tools/support/compat/github/tooling-test-shards.ts";

const read = (path: string) => fs.readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");

test("only actual selected SFC product and acceptance inputs add the source capture requirement", () => {
  for (const path of [
    "crates/vize_atelier_sfc/src/native_selected.rs",
    "crates/vize_atelier_sfc/src/native_selected/tests.rs",
    "crates/vize_atelier_sfc/src/native_selected/tests/maps.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_selected_sfc_click_vue_3_5_35.json",
    "tests/tooling/native-selected-sfc-dom-reference.test.ts",
    "tests/tooling/native-selected-sfc-dom-capture-workflow.test.ts",
    "tests/tooling/support/native-selected-sfc-dom-runtime.ts",
    ".github/actions/test-native-selected-sfc-dom/action.yml",
    "davinci/vize_l4/src/targets.rs",
    "davinci/vize_l4/src/targets/static_class.rs",
    "davinci/vize_l4/tests/native_dom_static_class.rs",
    "davinci/vize_l4/tests/fixtures/native-dom-static-class-vue-3.5.35.json",
    "tests/tooling/native-dom-static-class-reference.test.ts",
  ])
    assert(nativeSetupCaptureRequired([path]), path);
  for (const path of [
    "docs/davinci/decisions/2026-10-04-native-selected-sfc-dom.md",
    "crates/vize_atelier_sfc/src/native_selected_unrelated.rs",
    "crates/vize_atelier_sfc/tests/fixtures/other_selected_sfc.json",
    "tests/tooling/native-selected-unrelated.test.ts",
    "tests/tooling/support/native-selected-unrelated.ts",
    ".github/actions/other-native-selected/action.yml",
  ])
    assert.equal(nativeSetupCaptureRequired([path]), false, path);
  const plan = { tier: "pr" as const, tests: [] };
  assert(toolingChecksRequired(plan, ["crates/vize_atelier_sfc/src/native_selected.rs"]));
  assert.deepEqual(toolingShardMatrix(plan), { include: [{ index: 1, total: 1 }] });
});

test("the bounded first-worker hook captures whole selected SFCs before selected tooling runs", () => {
  const workflow = read(".github/workflows/pr-source-checks.yml");
  assert(workflow.trimEnd().split("\n").length <= 350);
  const build = workflow.indexOf("- name: Build and install vize CLI");
  const prep = workflow.indexOf("- name: Prepare the pinned plugin isolation runtime");
  const hook = workflow.indexOf("- name: Test source-bound native JS setup modules and runtime");
  const tooling = workflow.indexOf("- name: Test selected PR tooling scripts");
  assert(build >= 0 && build < prep && prep < hook && hook < tooling);
  const step = workflow.slice(hook).split("- name:")[1];
  assert.match(step, /matrix\.index == 1/);
  assert.match(step, /github\.event_name == 'merge_group' \|\|/);
  assert.match(step, /native-setup-capture == 'true'/);
  assert.match(step, /uses: \.\/\.github\/actions\/test-native-js-setup/);
  assert.doesNotMatch(step, /continue-on-error|hashFiles|existsSync/);
  const setup = read(".github/actions/test-native-js-setup/action.yml");
  const upload = setup.indexOf("- name: Upload native JS setup module and runtime captures");
  const selected = setup.indexOf(
    "- name: Capture whole original selected SFC components and runtime",
  );
  assert(upload >= 0 && upload < selected);
  const nested = setup.slice(selected);
  assert.match(nested, /uses: \.\/\.github\/actions\/test-native-selected-sfc-dom/);
  assert.doesNotMatch(nested, /\bif:|continue-on-error/);
});

test("selected SFC hosted acceptance requires actual whole Rust capture and retains exact-SHA artifacts", () => {
  const action = read(".github/actions/test-native-selected-sfc-dom/action.yml");
  assert.match(action, /VIZE_NATIVE_SELECTED_SFC_DOM_REQUIRE_CAPTURE: "1"/);
  assert.match(action, /VIZE_NATIVE_SELECTED_SFC_DOM_CAPTURE: \$\{\{ runner\.temp \}\}/);
  assert.match(action, /VIZE_NATIVE_SELECTED_SFC_DOM_RUNTIME_CAPTURE: \$\{\{ runner\.temp \}\}/);
  assert.match(
    action,
    /cargo test --locked --profile ci -p vize_atelier_sfc native_selected::tests::five_whole_original_click_components_and_maps_are_captured -- --exact/,
  );
  assert.match(
    action,
    /vp node --test tests\/tooling\/native-selected-sfc-dom-reference\.test\.ts/,
  );
  assert(action.indexOf("cargo test") < action.indexOf("vp node --test"));
  assert.match(action, /test -s "\$VIZE_NATIVE_SELECTED_SFC_DOM_CAPTURE"/);
  assert.match(action, /test -s "\$VIZE_NATIVE_SELECTED_SFC_DOM_RUNTIME_CAPTURE"/);
  assert.match(
    action,
    /native-selected-sfc-dom-\$\{\{ github\.sha \}\}-\$\{\{ github\.run_attempt \}\}/,
  );
  assert.match(action, /if: \$\{\{ always\(\) \}\}/);
  assert.match(action, /if-no-files-found: error/);
  assert.doesNotMatch(action, /continue-on-error|hashFiles|existsSync|--test-name-pattern/);
  const acceptance = read("tests/tooling/native-selected-sfc-dom-reference.test.ts");
  assert.doesNotMatch(acceptance, /\bskip\s*:/);
  assert.match(acceptance, /assert\(captured, "hosted acceptance requires actual Rust capture"\)/);
  assert.match(acceptance, /assert\.deepEqual\(\s*captured\.fixtures/);
});
