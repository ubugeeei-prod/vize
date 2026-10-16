import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import { nativeSetupCaptureRequired } from "../../tools/support/compat/github/native-setup-capture.mjs";
import { planNativeRuntime } from "../../tools/support/compat/github/plan-native-runtime.ts";
import { planPrChecks, prCheckOutputs } from "../../tools/support/compat/github/plan-pr-checks.ts";

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
    "crates/vize_atelier_sfc/tests/fixtures/native_original_for_value_sfc_vue_3_5_35.json",
    "tests/tooling/native-original-for-value-sfc-dom-reference.test.ts",
    "davinci/vize_l3/src/decision/dom/build/for_head/value.rs",
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
  assert.equal(nativeSetupCaptureRequired(["davinci/vize_l4/src/targets/dom_unrelated.rs"]), false);
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
  assert.match(action, /VIZE_NATIVE_ORIGINAL_FOR_VALUE_SFC_DOM_REQUIRE_CAPTURE: "1"/);
  assert.match(
    action,
    /original_for_value::six_whole_original_for_value_components_and_raw_maps_are_captured -- --exact/,
  );
  assert.match(
    action,
    /vp node --test tests\/tooling\/native-original-for-value-sfc-dom-reference\.test\.ts/,
  );
  assert.match(action, /test -s "\$VIZE_NATIVE_ORIGINAL_FOR_VALUE_SFC_DOM_CAPTURE"/);
  assert.match(action, /test -s "\$VIZE_NATIVE_ORIGINAL_FOR_VALUE_SFC_DOM_RUNTIME_CAPTURE"/);
  assert.match(action, /native-original-for-value-sfc-dom-runtime\.json/);
  assert.doesNotMatch(action, /continue-on-error|hashFiles|existsSync|--test-name-pattern/);
});

test("individual For resolver, DOM collector and writer changes keep mandatory affected runtime capture", () => {
  for (const path of [
    "davinci/vize_l2/src/resolution.rs",
    "davinci/vize_l2/src/resolution/walk/for_head.rs",
    "davinci/vize_l2/src/resolution/for_head/borrowed.rs",
    "davinci/vize_l3/src/decision/dom.rs",
    "davinci/vize_l3/src/decision/dom/build.rs",
    "davinci/vize_l3/src/decision/dom/build/binding.rs",
    "davinci/vize_l3/src/decision/dom/dependencies.rs",
    "davinci/vize_l3/src/decision/dom/build/for_head/runtime.rs",
    "davinci/vize_l4/src/targets/dom.rs",
    "davinci/vize_l4/src/targets/dom/for_head.rs",
    "davinci/vize_l4/src/targets/dom/expression.rs",
    "davinci/vize_l4/src/targets/dom/write.rs",
    "davinci/vize_l4/src/targets/dom/vue/for_head.rs",
    "davinci/vize_l3/src/decision/dom/build/for_head/value.rs",
    "tests/tooling/native-original-for-value-sfc-dom-reference.test.ts",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_for_value_sfc_vue_3_5_35.json",
    "davinci/vize_l4/src/expr.rs",
    "davinci/vize_l4/src/expr/vue.rs",
  ]) {
    assert(nativeSetupCaptureRequired([path]), path);
    const plan = planNativeRuntime([path]);
    assert(plan.run, path);
    assert(plan.matrix.lane.includes("setup"), path);
  }
  for (const path of [
    "davinci/vize_l2/src/resolution_unrelated.rs",
    "davinci/vize_l3/src/decision/dom_unrelated.rs",
    "davinci/vize_l4/src/targets/dom_unrelated.rs",
    "davinci/vize_l4/src/targets/ssr.rs",
    "davinci/vize_l4/src/expr_unrelated.rs",
    "tests/tooling/native-original-for-value-sfc-dom-reference_unrelated.test.ts",
    "crates/vize_atelier_sfc/tests/fixtures/native_original_for_value_sfc_unrelated.json",
    "docs/davinci/decisions/2026-10-04-original-for-alias-body-dom-proposal.md",
    "docs/davinci/decisions/2026-10-04-original-for-primitive-dom-proposal.md",
  ]) {
    assert.equal(nativeSetupCaptureRequired([path]), false, path);
  }
});

test("the PR planner requires the setup lane and its chained selected-SFC capture without full-ci", () => {
  // The baseline crate cannot select native captures: this oracle change must
  // reach the setup lane through the actual affected-runtime planner.
  const cargoMetadata = {
    workspace_root: "/repo",
    workspace_members: ["vize_relief"],
    packages: [
      {
        id: "vize_relief",
        name: "vize_relief",
        manifest_path: "/repo/crates/vize_relief/Cargo.toml",
        dependencies: [],
      },
    ],
  };
  for (const eventName of ["pull_request", "merge_group"]) {
    const plan = planPrChecks({
      paths: ["tests/tooling/native-original-for-value-sfc-dom-reference.test.ts"],
      eventName,
      labels: [],
      workspace: [],
      cargoMetadata,
    });
    const outputs = prCheckOutputs(plan);
    assert(plan.native.run, eventName);
    assert.deepEqual(JSON.parse(outputs["native-runtime-matrix"]), {
      lane: ["setup", "selected-template"],
    });
    if (eventName === "pull_request") assert.equal(outputs.heavy, "false");
  }
  const workflow = parse(
    fs.readFileSync(
      new URL("../../.github/workflows/pr-source-checks.yml", import.meta.url),
      "utf8",
    ),
  );
  const planner = workflow.jobs["pr-source-plan"];
  assert.equal(
    planner.outputs["native-runtime-matrix"],
    "${{ steps.plan.outputs.native-runtime-matrix }}",
  );
  const planStep = planner.steps.find((step: { id?: string }) => step.id === "plan");
  assert.equal(
    planStep.run,
    'node tools/support/compat/github/plan-pr-checks.ts "$BASE_SHA" "$GITHUB_SHA" "$GITHUB_EVENT_NAME"',
  );
  const runtime = workflow.jobs["pr-native-runtime"];
  assert.equal(runtime.needs, "pr-source-plan");
  assert.equal(runtime.if, undefined);
  assert.equal(runtime["continue-on-error"], undefined);
  assert.equal(
    runtime.strategy.matrix,
    "${{ fromJSON(needs.pr-source-plan.outputs.native-runtime-matrix) }}",
  );
  const hooks = runtime.steps.filter(
    (step: { uses?: string }) => step.uses === "./.github/actions/test-native-js-setup",
  );
  assert.equal(hooks.length, 1);
  assert.equal(hooks[0].if, "${{ matrix.lane == 'setup' }}");
  assert.equal(hooks[0]["continue-on-error"], undefined);
  assert(workflow.jobs["source-report"].needs.includes("pr-native-runtime"));
  const enclosingAction = parse(
    fs.readFileSync(
      new URL("../../.github/actions/test-native-js-setup/action.yml", import.meta.url),
      "utf8",
    ),
  );
  const captures = enclosingAction.runs.steps.filter(
    (step: { uses?: string }) => step.uses === "./.github/actions/test-native-selected-sfc-dom",
  );
  assert.equal(captures.length, 1);
  assert.equal(captures[0].if, undefined);
  assert.equal(captures[0]["continue-on-error"], undefined);
});
