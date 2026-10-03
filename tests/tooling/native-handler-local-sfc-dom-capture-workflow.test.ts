import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { nativeSetupCaptureRequired } from "../../tools/support/compat/github/native-setup-capture.mjs";

const read = (path: string) => fs.readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");

test("original local-handler source qualifies the existing source-bound tooling worker", () => {
  for (const path of [
    "davinci/vize_l3/src/decision/dom/build/handler.rs",
    "davinci/vize_l3/src/decision/dom/build/handler/access.rs",
    "crates/vize_atelier_sfc/src/native_selected/tests/local.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_handler_local_sfc_click_vue_3_5_35.json",
    "tests/tooling/native-handler-local-sfc-dom-reference.test.ts",
    "tests/tooling/native-handler-local-sfc-dom-capture-workflow.test.ts",
    "tests/tooling/support/native-handler-local-sfc-dom-runtime.ts",
    ".github/actions/test-native-handler-local-sfc-dom/action.yml",
  ])
    assert.equal(nativeSetupCaptureRequired([path]), true, path);
  for (const path of [
    "docs/davinci/decisions/2026-10-04-native-handler-block-local-reads.md",
    "tests/tooling/native-handler-local-unrelated.test.ts",
    "davinci/vize_l3/src/decision/dom/build/handler_unrelated.rs",
  ])
    assert.equal(nativeSetupCaptureRequired([path]), false, path);
});

test("actual capture runs after plugin preparation and preserves previous complete captures", () => {
  const setup = read(".github/actions/test-native-js-setup/action.yml");
  assert(
    setup.indexOf("Prepare the pinned plugin isolation runtime") <
      setup.indexOf("Capture original block-local handler components and runtime"),
  );
  assert(
    setup.indexOf("test-native-selected-sfc-dom") <
      setup.indexOf("test-native-handler-local-sfc-dom"),
  );
  assert.match(setup, /uses: \.\/\.github\/actions\/test-native-handler-local-sfc-dom/);
  const action = read(".github/actions/test-native-handler-local-sfc-dom/action.yml");
  assert.match(
    action,
    /cargo test --locked --profile ci -p vize_atelier_sfc native_selected::tests::local::ten_original_block_local_components_and_maps_are_captured -- --exact/,
  );
  assert.match(action, /VIZE_NATIVE_HANDLER_LOCAL_SFC_DOM_REQUIRE_CAPTURE: "1"/);
  assert.match(
    action,
    /vp node --test tests\/tooling\/native-handler-local-sfc-dom-reference\.test\.ts/,
  );
  assert.match(action, /if: \$\{\{ always\(\) \}\}/);
  assert.match(
    action,
    /native-handler-local-sfc-dom-\$\{\{ github\.sha \}\}-\$\{\{ github\.run_attempt \}\}/,
  );
  assert.match(action, /if-no-files-found: error/);
  assert.doesNotMatch(action, /continue-on-error|test-name-pattern/);
});

test("acceptance checks full real maps and current-source runtime without skips", () => {
  const acceptance = read("tests/tooling/native-handler-local-sfc-dom-reference.test.ts");
  assert.match(acceptance, /fixture\.nativeMap/);
  assert.match(acceptance, /current-source-rust-capture/);
  assert.match(acceptance, /VIZE_NATIVE_HANDLER_LOCAL_SFC_DOM_REQUIRE_CAPTURE/);
  assert.doesNotMatch(acceptance, /test\.skip|skip:|test-name-pattern/);
});
