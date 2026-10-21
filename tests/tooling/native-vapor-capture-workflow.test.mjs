import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { nativeVaporCaptureRequired } from "../../tools/support/compat/github/native-vapor-capture.mjs";
import { planNativeRuntime } from "../../tools/support/compat/github/plan-native-runtime.ts";
import { assertNativeRuntimeHook } from "./support/native-runtime-workflow.ts";

await test("original native Vapor source qualifies a fresh affected-PR capture without prose admissions", () => {
  for (const path of [
    "davinci/vize_l1/src/container/vue/descriptor/policy.rs",
    "davinci/vize_l2/src/lang/js/file/native.rs",
    "davinci/vize_l1_to_l2/src/native_file/selected.rs",
    "davinci/vize_l3/src/decision/vapor/build.rs",
    "davinci/vize_l4/src/targets/vapor/component.rs",
    "davinci/vize_l4/src/module.rs",
    "davinci/vize_l4/src/expr.rs",
    "davinci/vize_l4/src/expr/resolved.rs",
    "davinci/vize_l4/src/expr/vue.rs",
    "davinci/vize_l4/src/write/source_map.rs",
    "davinci/vize_l4/tests/native_vapor.rs",
    "crates/vize_atelier_jsx/src/vapor.rs",
    "crates/vize_atelier_jsx/tests/v_slots_forwarding.rs",
    "crates/vize_atelier_vapor/src/generate/operations/component.rs",
    "crates/vize_atelier_sfc/src/lib.rs",
    "crates/vize_atelier_sfc/src/native_vapor.rs",
    "crates/vize_atelier_sfc/src/native_vapor_setup.rs",
    "crates/vize_atelier_sfc/tests/native_vapor_sfc.rs",
    "crates/vize_atelier_sfc/tests/native_vapor_setup_sfc.rs",
    "crates/vize_atelier_sfc/tests/vapor_setup_refusals/mod.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native_vapor_sfc_vue_3_6_rc9.json",
    "tests/tooling/native-vapor-sfc-reference.test.mjs",
    "tests/tooling/native-vapor-setup-sfc-reference.test.mjs",
    "tests/tooling/jsx-runtime-slots-vapor.test.mjs",
    "tests/tooling/support/davinci-runtime-trace.mjs",
    "tests/tooling/support/native-vapor-sfc-oracle.mjs",
    "tests/tooling/support/native-vapor-primary-lifecycle.mjs",
    "tests/tooling/support/native-vapor-process-capture.mjs",
    "tests/tooling/support/vue-vapor-release.mjs",
    ".github/actions/test-native-vapor/action.yml",
    ".github/workflows/check.yml",
    ".github/workflows/pr-source-checks.yml",
  ]) {
    assert.equal(nativeVaporCaptureRequired([path]), true, path);
    assert(planNativeRuntime([path]).matrix.lane.includes("vapor"), path);
  }
  for (const path of [
    "README.md",
    "docs/davinci/decisions/2026-09-27-level-restructure.md",
    "crates/vize_atelier_sfc/src/native_ssr.rs",
    "crates/vize_atelier_sfc/tests/vapor_setup_refusals_unrelated/mod.rs",
    "npm/ui/src/main.ts",
    "davinci/vize_l4/src/targets/ts.rs",
    "davinci/vize_l4/src/expr.rs.unrelated",
    "davinci/vize_l4/src/expr_unrelated/resolved.rs",
  ])
    assert.equal(nativeVaporCaptureRequired([path]), false, path);
  assert.equal(nativeVaporCaptureRequired([]), false);
});

await test("the required Vapor lane preserves mandatory whole-source payloads", () => {
  const read = (path) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
  assertNativeRuntimeHook("vapor", "test-native-vapor");
  const action = read(".github/actions/test-native-vapor/action.yml");
  for (const path of [
    "native-vapor-modules.json",
    "native-vapor-runtime.json",
    "native-vapor-sfc-modules.json",
    "native-vapor-sfc-runtime.json",
    "native-vapor-primary-lifecycle.json",
    "native-vapor-processes.jsonl",
    "native-vapor-setup-sfc-modules.json",
    "native-vapor-setup-sfc-runtime.json",
    "jsx-runtime-slots-vapor.json",
    "jsx-runtime-slots-vapor.json.runtime.json",
  ])
    assert.ok(action.includes(path), path);
  assert.match(action, /cargo test --locked --profile ci -p vize_l4 --test native_vapor/);
  assert.match(
    action,
    /cargo test --locked --profile ci -p vize_atelier_sfc --test native_vapor_sfc/,
  );
  assert.match(
    action,
    /cargo test --locked --profile ci -p vize_atelier_jsx --test v_slots_forwarding/,
  );
  assert.ok(action.includes("tests/tooling/jsx-runtime-slots-vapor.test.mjs"));
  assert.match(action, /VIZE_NATIVE_VAPOR_REQUIRE_CAPTURE: "1"/);
  assert.match(action, /if-no-files-found: error/);
  assert.match(action, /if: \$\{\{ always\(\) \}\}/);
});
