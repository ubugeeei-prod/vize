import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { Script } from "node:vm";
import { nativeAttributeValues7502CaptureRequired } from "../../tools/support/compat/github/native-attribute-values-7502-capture.mjs";
import { nativeScopedCaptureRequired } from "../../tools/support/compat/github/native-scoped-capture.mjs";
import { nativeSsrCaptureRequired } from "../../tools/support/compat/github/native-ssr-capture.mjs";
import {
  nativeSetupCaptureRequired,
  toolingChecksRequired,
} from "../../tools/support/compat/github/native-setup-capture.mjs";
import { nativeVaporCaptureRequired } from "../../tools/support/compat/github/native-vapor-capture.mjs";
import { toolingShardMatrix } from "../../tools/support/compat/github/tooling-test-shards.ts";

const read = (path) => readFileSync(new URL(`../../${path}`, import.meta.url), "utf8");
const planner = read("tools/support/compat/github/plan-tooling-tests.mjs");
// Evaluate the expressions actually written to GITHUB_OUTPUT rather than a
// second hand-maintained copy of the planner's source-selection logic.
function output(name, paths) {
  const expression = planner.match(new RegExp(`${name}=\\$\\{([^}]+)\\}`))?.[1];
  assert(expression, name);
  return new Script(expression).runInNewContext({
    plan: { tier: "pr", tests: [] },
    paths,
    capturePaths: paths,
    toolingChecksRequired,
    nativeSsrCaptureRequired,
    nativeVaporCaptureRequired,
    nativeScopedCaptureRequired,
    nativeSetupCaptureRequired,
    nativeAttributeValues7502CaptureRequired,
  });
}

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
    assert.equal(output("native-ssr-capture", [path]), true, path);
    assert.equal(output("tooling", [path]), true, path);
  }
  assert.deepEqual(toolingShardMatrix({ tier: "pr", tests: [] }), {
    include: [{ index: 1, total: 1 }],
  });
});

void test("old SSR classification is preserved and unrelated docs/inventories never select a new proof", () => {
  for (const path of [
    "davinci/vize_l1_to_l2/src/native_file/selected.rs",
    "davinci/vize_l2/src/lang/js/file/native/scoped.rs",
    "davinci/vize_l3/src/decision/ssr/scoped.rs",
    "davinci/vize_l4/src/targets/ssr.rs",
    "crates/vize_atelier_sfc/src/native_ssr.rs",
  ]) {
    assert.equal(nativeSsrCaptureRequired([path]), true, path);
    assert.equal(output("native-ssr-capture", [path]), true, path);
    assert.equal(output("tooling", [path]), true, path);
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
    assert.equal(output("native-ssr-capture", [path]), false, path);
  }
  assert.equal(output("native-ssr-capture", []), false);
  assert.equal(output("tooling", ["README.md"]), false);
});

void test("both existing action guards retain the complete event/tooling/shard truth table", () => {
  const workflow = read(".github/workflows/pr-source-checks.yml");
  assert(workflow.split("\n").length - 1 <= 350);
  for (const action of ["test-native-ssr", "test-native-scoped-css"]) {
    const condition = workflow.match(
      new RegExp(
        `if: \\$\\{\\{ ([^\\n]+) \\}\\}\\n        uses: \\./\\.github/actions/${action}\\n`,
      ),
    )?.[1];
    assert(condition, action);
    const expression = condition
      .replaceAll("needs.pr-source-plan.outputs.tooling", "tooling")
      .replaceAll("needs.pr-source-plan.outputs.native-ssr-capture", "capture")
      .replaceAll("matrix.index", "index")
      .replaceAll("github.event_name", "event");
    const run = new Script(expression);
    for (const event of ["pull_request", "merge_group", "push", "workflow_dispatch"])
      for (const tooling of ["true", "false"])
        for (const capture of ["true", "false"])
          for (const index of [1, 2, 3, 4])
            assert.equal(
              run.runInNewContext({ tooling, capture, index, event }),
              tooling === "true" &&
                index === 1 &&
                (event === "merge_group" || (event === "pull_request" && capture === "true")),
              `${action}/${event}/${tooling}/${capture}/${index}`,
            );
  }
  const action = read(".github/actions/test-native-scoped-css/action.yml");
  assert(action.includes('VIZE_NATIVE_SCOPED_SSR_REQUIRE_NATIVE: "1"'));
  assert(
    action.includes(
      "--test native_scoped_ssr three_whole_original_scoped_ssr_modules_css_maps_and_file_custody_match -- --exact",
    ),
  );
  assert(action.includes("if: always()") && action.includes("if-no-files-found: error"));
});
