import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { nativeSsrCaptureRequired } from "../../tools/support/compat/github/native-ssr-capture.mjs";

void test("authentic SSR source dependencies require an affected PR capture", () => {
  for (const path of [
    "crates/vize_atelier_sfc/src/native_ssr.rs",
    "crates/vize_atelier_sfc/src/native_ssr/output.rs",
    "crates/vize_atelier_sfc/src/lib.rs",
    "crates/vize_atelier_sfc/tests/native_scriptless_ssr.rs",
    "crates/vize_atelier_sfc/tests/fixtures/native-sfc-ssr-vue-3.5.35.json",
    "crates/vize_atelier_sfc/tests/fixtures/native-scriptless-ssr-output.json",
    "davinci/vize_l0/src/markup/tags.rs",
    "davinci/vize_l1/src/container/vue/descriptor.rs",
    "davinci/vize_l1/src/markup/native.rs",
    "davinci/vize_l1/src/event/native.rs",
    "davinci/vize_l1_to_l2/src/native_file.rs",
    "davinci/vize_l1_to_l2/src/native_file/selected/walk.rs",
    "davinci/vize_l2/src/file/region/native/body/header.rs",
    "davinci/vize_l2/src/lang/js/handler/native.rs",
    "davinci/vize_l3/src/decision/build.rs",
    "davinci/vize_l3/src/decision/ssr/build.rs",
    "davinci/vize_l4/src/targets/ssr/write.rs",
    "davinci/vize_l4/src/write/source_map.rs",
    "davinci/vize_l4/src/module.rs",
    "davinci/vize_l4/src/module/imports.rs",
    "davinci/vize_l4/src/module/setup.rs",
    "tests/tooling/native-sfc-scriptless-ssr-reference.test.ts",
    "tests/tooling/l4-native-ssr-reference.test.ts",
    "tests/tooling/l4-selected-ssr-reference.test.ts",
    "tests/tooling/support/native-sfc-ssr-reference.ts",
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

void test("the existing first-shard action remains mandatory for merge groups and qualified PR sources", () => {
  const workflow = fs.readFileSync(
    new URL("../../.github/workflows/pr-source-checks.yml", import.meta.url),
    "utf8",
  );
  const planner = fs.readFileSync(
    new URL("../../tools/support/compat/github/plan-tooling-tests.mjs", import.meta.url),
    "utf8",
  );
  const action = fs.readFileSync(
    new URL("../../.github/actions/test-native-ssr/action.yml", import.meta.url),
    "utf8",
  );
  assert(workflow.split("\n").length - 1 <= 350);
  assert(
    workflow.includes("native-ssr-capture: ${{ steps.tooling-plan.outputs.native-ssr-capture }}"),
  );
  assert(
    workflow.includes(
      "matrix.index == 1 && (github.event_name == 'merge_group' || (github.event_name == 'pull_request' && needs.pr-source-plan.outputs.native-ssr-capture == 'true'))",
    ),
  );
  assert.equal(workflow.match(/uses: \.\/\.github\/actions\/test-native-ssr/g)?.length, 1);
  assert(
    planner.includes(
      "toolingChecksRequired(plan, paths) || nativeSsrCaptureRequired(capturePaths)",
    ),
  );
  assert(planner.includes("native-ssr-capture=${nativeSsrCaptureRequired(capturePaths)}"));
  assert(
    planner.includes(
      'const capturePaths = /^0+$/.test(base) ? [".github/workflows/check.yml"] : paths;',
    ),
  );
  assert(action.includes('VIZE_L4_SSR_REQUIRE_NATIVE: "1"'));
  assert(action.includes("--test native_scriptless_ssr -- --nocapture"));
  assert(
    action.includes("native-sfc-ssr-modules.json") &&
      action.includes("native-sfc-ssr-runtime.json"),
  );
});
