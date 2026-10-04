import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { nativeSetupCaptureRequired } from "../../tools/support/compat/github/native-setup-capture.mjs";

test("actual selected setup consumers require source-built whole-module runtime acceptance", () => {
  for (const path of [
    "crates/vize_atelier_sfc/src/native_selected_setup.rs",
    "crates/vize_atelier_sfc/src/native_selected_setup/tests.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_selected_setup_sfc_vue_3_5_35.json",
    "tests/tooling/native-selected-setup-sfc-dom-reference.test.ts",
    "tests/tooling/native-selected-setup-sfc-dom-capture-workflow.test.ts",
    "tests/tooling/support/native-selected-setup-sfc-dom-runtime.ts",
    "davinci/vize_l3/src/decision/native/setup.rs",
    "davinci/vize_l3/src/decision/dom/build.rs",
    "davinci/vize_l3/src/decision/dom/build/binding.rs",
    "davinci/vize_l3/src/decision/dom/dependencies.rs",
    "davinci/vize_l3/src/decision/dom/build/for_head/runtime.rs",
    "davinci/vize_l4/src/targets/dom.rs",
    "davinci/vize_l4/src/targets/dom/for_head.rs",
    "davinci/vize_l4/src/targets/dom/vue/for_head.rs",
    "davinci/vize_l4/src/targets/dom/expression.rs",
    "davinci/vize_l4/src/targets/dom/write.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_for_sfc_vue_3_5_35.json",
    "tests/tooling/native-original-for-sfc-dom-reference.test.ts",
  ])
    assert(nativeSetupCaptureRequired([path]), path);
  assert.equal(
    nativeSetupCaptureRequired([
      "docs/davinci/decisions/2026-10-04-selected-setup-dom-consumer.md",
    ]),
    false,
  );
  assert.equal(
    nativeSetupCaptureRequired(["crates/vize_atelier_sfc/src/native_selected_setup_unrelated.rs"]),
    false,
  );
  assert.equal(
    nativeSetupCaptureRequired(["davinci/vize_l4/src/targets/dom/for_head_other.rs"]),
    false,
  );
  assert.equal(
    nativeSetupCaptureRequired([
      "docs/davinci/decisions/2026-10-04-original-for-primitive-dom-proposal.md",
    ]),
    false,
  );
  const action = fs.readFileSync(
    new URL("../../.github/actions/test-native-selected-sfc-dom/action.yml", import.meta.url),
    "utf8",
  );
  assert.match(action, /VIZE_NATIVE_SELECTED_SETUP_SFC_DOM_REQUIRE_CAPTURE: "1"/);
  assert.match(
    action,
    /cargo test --locked --profile ci -p vize_atelier_sfc native_selected_setup::tests::seven_whole_original_setup_components_and_maps_are_captured -- --exact/,
  );
  assert.match(
    action,
    /vp node --test tests\/tooling\/native-selected-setup-sfc-dom-reference\.test\.ts/,
  );
  assert.match(action, /test -s "\$VIZE_NATIVE_SELECTED_SETUP_SFC_DOM_CAPTURE"/);
  assert.match(action, /test -s "\$VIZE_NATIVE_SELECTED_SETUP_SFC_DOM_RUNTIME_CAPTURE"/);
  assert.match(
    action,
    /native-selected-sfc-dom-\$\{\{ github\.sha \}\}-\$\{\{ github\.run_attempt \}\}/,
  );
  assert.match(action, /VIZE_NATIVE_ORIGINAL_FOR_SFC_DOM_REQUIRE_CAPTURE: "1"/);
  assert.match(
    action,
    /original_for::ten_whole_original_for_components_and_maps_are_captured -- --exact/,
  );
  assert.match(action, /native-original-for-sfc-dom-runtime\.json/);
  assert.doesNotMatch(action, /continue-on-error|hashFiles|existsSync|--test-name-pattern/);
});
