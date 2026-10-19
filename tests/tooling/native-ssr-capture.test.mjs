import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import { planNativeRuntime } from "../../tools/support/compat/github/plan-native-runtime.ts";
import { nativeSsrCaptureRequired } from "../../tools/support/compat/github/native-ssr-capture.mjs";
import { directChildHistoricalSuccessor } from "./support/native-ssr-direct-child-successor.ts";

void test("the original whole SSR archives permit only the named nested-refusal successor", () => {
  const original = JSON.parse(
    fs.readFileSync(
      new URL(
        "../../crates/vize_atelier_sfc/tests/fixtures/native-setup-ssr-output.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  for (const packet of [original.capture, original.runtime]) {
    const before = structuredClone(packet);
    const successor = directChildHistoricalSuccessor(packet);
    assert.equal(successor.refusals.length, 19);
    assert.deepEqual({ ...successor, refusals: packet.refusals }, before);
    assert.deepEqual(packet, before);
    const unknown = structuredClone(packet);
    unknown.refusals[15].diagnostics.push({ unknown: true });
    assert.throws(() => directChildHistoricalSuccessor(unknown));
    const reordered = structuredClone(packet);
    [reordered.refusals[14], reordered.refusals[15]] = [
      reordered.refusals[15],
      reordered.refusals[14],
    ];
    assert.throws(() => directChildHistoricalSuccessor(reordered));
  }
});

void test("authentic SSR source dependencies require an affected PR capture", () => {
  for (const path of [
    "crates/vize_atelier_sfc/src/native_ssr.rs",
    "crates/vize_atelier_sfc/src/native_ssr_setup.rs",
    "crates/vize_atelier_sfc/src/native_ssr/output.rs",
    "crates/vize_atelier_sfc/src/lib.rs",
    "crates/vize_atelier_sfc/tests/native_scriptless_ssr.rs",
    "crates/vize_atelier_sfc/tests/native_scriptless_setup_ssr.rs",
    "crates/vize_atelier_sfc/tests/native_setup_ssr/rows.rs",
    "crates/vize_atelier_sfc/tests/native_setup_ssr/refusals.rs",
    "crates/vize_atelier_sfc/tests/native_setup_ssr/direct_child.rs",
    "crates/vize_atelier_sfc/tests/native_setup_ssr/nested_custody.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_sfc_setup_ssr_sources.json",
    "crates/vize_atelier_sfc/tests/fixtures/native-setup-ssr-output.json",
    "crates/vize_atelier_sfc/tests/fixtures/native-setup-ssr-direct-child-sources.json",
    "crates/vize_atelier_sfc/tests/fixtures/native-setup-ssr-direct-child-output.json",
    "crates/vize_atelier_sfc/tests/fixtures/native-setup-ssr-vue-3.5.35.json",
    "crates/vize_atelier_sfc/tests/fixtures/native-sfc-ssr-vue-3.5.35.json",
    "crates/vize_atelier_sfc/tests/fixtures/native-scriptless-ssr-output.json",
    "crates/vize_atelier_sfc/tests/fixtures/native-scriptless-ssr-output-v2.json",
    "crates/vize_atelier_sfc/tests/native_scriptless_ssr/class_successor.rs",
    "davinci/vize_l0/src/markup/tags.rs",
    "davinci/vize_l1/src/container/vue/descriptor.rs",
    "davinci/vize_l1/src/markup/native.rs",
    "davinci/vize_l1/src/event/native.rs",
    "davinci/vize_l1_to_l2/src/native_file.rs",
    "davinci/vize_l1_to_l2/src/native_file/selected/walk.rs",
    "davinci/vize_l1_to_l2/src/native_file/selected_setup.rs",
    "davinci/vize_l1_to_l2/src/native_file/selected_setup/tests/accepted.rs",
    "davinci/vize_l2/src/file/region/native/body/header.rs",
    "davinci/vize_l2/src/lang/js/handler/native.rs",
    "davinci/vize_l3/src/decision/build.rs",
    "davinci/vize_l3/tests/native_setup_ssr.rs",
    "davinci/vize_l3/tests/native_ssr_receipt.rs",
    "davinci/vize_l3/src/decision/ssr/build.rs",
    "davinci/vize_l4/src/targets/ssr/write.rs",
    "davinci/vize_l4/src/targets/static_class.rs",
    "davinci/vize_l4/src/targets.rs",
    "davinci/vize_l4/src/write/source_map.rs",
    "davinci/vize_l4/src/module.rs",
    "davinci/vize_l4/src/module/imports.rs",
    "davinci/vize_l4/src/module/setup.rs",
    "davinci/vize_l4/src/module/setup/segments.rs",
    "davinci/vize_l4/src/module/setup/segments/tests.rs",
    "davinci/vize_l4/src/module/scope.rs",
    "davinci/vize_l4/src/expr.rs",
    "davinci/vize_l4/src/expr/resolved.rs",
    "davinci/vize_l4/src/expr/vue.rs",
    "tests/tooling/native-sfc-scriptless-ssr-reference.test.ts",
    "tests/tooling/native-sfc-setup-ssr-reference.test.ts",
    "tests/tooling/native-sfc-setup-ssr-direct-child-reference.test.ts",
    "crates/vize_atelier_jsx/src/ssr.rs",
    "crates/vize_atelier_jsx/src/lower/v_slots.rs",
    "crates/vize_atelier_ssr/src/l4/emit/component.rs",
    "crates/vize_atelier_jsx/tests/ssr.rs",
    "crates/vize_atelier_jsx/tests/falsy_and.rs",
    "tests/tooling/support/jsx-falsy-and-runtime.mjs",
    "tests/tooling/jsx-runtime-slots-ssr.test.ts",
    "tests/tooling/l4-native-ssr-reference.test.ts",
    "tests/tooling/native-ssr-static-class-reference.test.ts",
    "tests/tooling/l4-selected-ssr-reference.test.ts",
    "tests/tooling/support/native-sfc-ssr-reference.ts",
    "tests/tooling/support/native-sfc-setup-ssr-reference.ts",
    "tests/tooling/support/native-sfc-setup-ssr-loader.ts",
    "tests/tooling/support/native-ssr-direct-child-successor.ts",
    ".github/actions/test-native-ssr/action.yml",
    ".github/workflows/pr-source-checks.yml",
    ".github/workflows/check.yml",
    "tools/support/compat/github/native-ssr-capture.mjs",
    "pnpm-lock.yaml",
    "Cargo.lock",
  ])
    assert.equal(nativeSsrCaptureRequired([path]), true, path);
});

void test("prose, generated ledgers and independent target consumers do not grant SSR capture inputs", () => {
  for (const path of [
    "docs/davinci/decisions/2026-10-04-native-scriptless-ssr-sfc.md",
    "docs/davinci/plan/croquis-consumption/vize_atelier_sfc.md",
    "docs/davinci/plan/storage-inventory.tsv",
    "crates/vize_atelier_sfc/src/native_vapor.rs",
    "crates/vize_atelier_sfc/src/native_selected.rs",
    "crates/vize_atelier_sfc/src/native/setup.rs",
    "davinci/vize_l1/src/css.rs",
    "davinci/vize_l4/src/targets/vapor/write.rs",
    "davinci/vize_l4/src/targets/dom/output.rs",
    "README.md",
  ])
    assert.equal(nativeSsrCaptureRequired([path]), false, path);
  assert.equal(nativeSsrCaptureRequired([]), false);
  assert.equal(
    nativeSsrCaptureRequired(["README.md", "crates/vize_atelier_sfc/src/native_ssr.rs"]),
    true,
  );
});

void test("the native runtime matrix requires SSR for affected sources and broad qualification", () => {
  const workflow = parse(
    fs.readFileSync(
      new URL("../../.github/workflows/pr-source-checks.yml", import.meta.url),
      "utf8",
    ),
  );
  const action = fs.readFileSync(
    new URL("../../.github/actions/test-native-ssr/action.yml", import.meta.url),
    "utf8",
  );
  const worker = workflow.jobs["pr-native-runtime"];
  assert.equal(worker.needs, "pr-source-plan");
  assert.equal(worker.strategy["fail-fast"], false);
  assert.equal(
    worker.strategy.matrix,
    "${{ fromJSON(needs.pr-source-plan.outputs.native-runtime-matrix) }}",
  );
  const hooks = worker.steps.filter((step) => step.uses === "./.github/actions/test-native-ssr");
  assert.equal(hooks.length, 1);
  assert.equal(hooks[0].if, "${{ matrix.lane == 'ssr' }}");
  assert.notEqual(hooks[0]["continue-on-error"], true);
  assert.ok(workflow.jobs["source-report"].needs.includes("pr-native-runtime"));
  assert.ok(planNativeRuntime([]).matrix.lane.includes("ssr"));
  assert.ok(
    planNativeRuntime(["crates/vize_atelier_sfc/src/native_ssr.rs"]).matrix.lane.includes("ssr"),
  );
  assert.ok(planNativeRuntime(["davinci/vize_l1/src/css.rs"]).matrix.lane.includes("ssr"));
  assert.equal(planNativeRuntime(["docs/unrelated.md"]).matrix.lane.includes("ssr"), false);
  assert(action.includes('VIZE_L4_SSR_REQUIRE_NATIVE: "1"'));
  assert(action.includes("VIZE_JSX_SLOTS_SSR_CAPTURE:"));
  assert(action.includes("VIZE_JSX_VAPOR_SLOTS_SSR_CAPTURE:"));
  const native = parse(action).runs.steps.find((step) => step.id === "native-ssr");
  assert.notEqual(
    native.env.VIZE_JSX_VAPOR_SLOTS_SSR_CAPTURE,
    native.env.VIZE_JSX_SLOTS_SSR_CAPTURE,
    "each native entrypoint owns its capture path",
  );
  assert(action.includes("-p vize_atelier_jsx --test ssr -- --nocapture"));
  assert(action.includes("tests/tooling/jsx-runtime-slots-ssr.test.ts"));
  assert(action.includes("jsx-runtime-slots-ssr.json.runtime.json"));
  assert(action.includes("jsx-vapor-runtime-slots-ssr.json.runtime.json"));
  assert(action.includes("--test native_scriptless_ssr -- --nocapture"));
  assert(action.includes("--test native_scriptless_setup_ssr -- --nocapture"));
  assert(action.includes("vp node --test tests/tooling/native-sfc-setup-ssr-reference.test.ts"));
  assert(action.includes("native-setup-ssr-modules.json.positive.json"));
  assert(action.includes("native-setup-ssr-runtime.json"));
  assert(action.includes("native-sfc-setup-ssr-direct-child-reference.test.ts"));
  assert(action.includes("native-setup-ssr-modules.json.direct-child.json"));
  assert(action.includes("native-setup-ssr-runtime.json.direct-child.json"));
  assert(
    action.includes("native-sfc-ssr-modules.json") &&
      action.includes("native-sfc-ssr-runtime.json"),
  );
});

void test("NativeSSR retains complete value-sensitive JSX modules and runtime outcomes", () => {
  const action = parse(fs.readFileSync(".github/actions/test-native-ssr/action.yml", "utf8"));
  const native = action.runs.steps.find((step) => step.id === "native-ssr");
  assert.equal(
    native.env.VIZE_JSX_VALUE_CAPTURE,
    "${{ runner.temp }}/jsx-value-sensitive-modules.json",
  );
  assert.equal(
    native.env.VIZE_JSX_FALSY_RECEIPT,
    "${{ runner.temp }}/jsx-value-sensitive-runtime.json",
  );
  assert(
    native.run
      .split("\n")
      .includes(
        "cargo test --locked --profile ci -p vize_atelier_jsx --test falsy_and actual_values_updates_and_hydration_match_jsx_child_semantics -- --exact --nocapture",
      ),
  );
  const upload = action.runs.steps.find((step) =>
    step.uses?.startsWith("actions/upload-artifact@"),
  );
  for (const capture of [native.env.VIZE_JSX_VALUE_CAPTURE, native.env.VIZE_JSX_FALSY_RECEIPT])
    assert(upload.with.path.split("\n").includes(capture));
  assert.equal(upload.if, "${{ always() && steps.native-ssr.outcome != 'skipped' }}");
});
