import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";
import { userNativeRuntimeSelectionRequired } from "../../tools/support/compat/github/user-native-runtime-selection.ts";
import { planPrChecks, prCheckOutputs } from "../../tools/support/compat/github/plan-pr-checks.ts";

const names = ["vize_carton", "vize_canon", "vize_patina", "vize_relief"];
const cargoMetadata = {
  workspace_root: "/repo",
  workspace_members: names,
  packages: names.map((name) => ({
    id: name,
    name,
    manifest_path: `/repo/crates/${name}/Cargo.toml`,
    dependencies: [],
  })),
};

test("runtime changes and observer-only changes select the existing CLI and source-addon jobs", () => {
  for (const path of [
    "crates/vize_carton/src/corsa_resolver/packages.rs",
    "crates/vize_canon/src/lsp_client/bootstrap.rs",
    "crates/vize_patina/src/linter/corsa_session/paths/store.rs",
    "npm/cli/package.json",
    "editors/vscode/src/corsa-runtime.ts",
    "tests/tooling/native-runtime-cli-selection.test.ts",
    "crates/vize_maestro/src/ide/diagnostics.rs",
    "crates/vize_maestro/src/ide/diagnostics/typecheck_unavailable_tests.rs",
    "tests/tooling/native-runtime-napi-selection.test.ts",
    "tests/tooling/support/native-runtime-selection/lsp.ts",
    ".github/actions/test-native-runtime-selection/action.yml",
    // Observer dependencies: build receipts, the formatter producer, the planner and the frozen runtime.
    "tests/differential/build-receipt.ts",
    "npm/native/scripts/formatter-history-build.mts",
    "tools/support/compat/github/plan-pr-checks.ts",
    ".github/actions/setup-frozen-native-test-runtime/action.yml",
  ]) {
    for (const eventName of ["pull_request", "merge_group"]) {
      assert(userNativeRuntimeSelectionRequired([path]), path);
      const plan = planPrChecks({
        paths: [path],
        eventName,
        labels: [],
        workspace: [],
        cargoMetadata,
      });
      assert.equal(prCheckOutputs(plan)["user-native-runtime-selection"], "true", path);
      assert(plan.native.matrix.lane.includes("lsp"), path);
      assert(plan.js.matrix.group.includes("builders"), path);
      assert.equal(plan.js.native, true);
    }
  }
  assert(userNativeRuntimeSelectionRequired([]), "an unknown predecessor remains conservative");
});

test("unrelated markup and documentation do not add the compiler selection matrix", () => {
  for (const path of [
    "docs/content/guide/type-checking.md",
    "npm/ui/src/Example.vue",
    "crates/vize_glyph/src/lib.rs",
  ]) {
    assert.equal(userNativeRuntimeSelectionRequired([path]), false, path);
  }
});

test("required workflow calls reuse ordinary source producers and retain failed raw observations", () => {
  type Step = {
    name?: string;
    if?: string;
    uses?: string;
    run?: string;
    with?: Record<string, string>;
    env?: Record<string, string>;
  };
  const read = (file: string) =>
    parse(readFileSync(new URL(`../../${file}`, import.meta.url), "utf8"));
  const source = read(".github/workflows/pr-source-checks.yml");
  const js = read(".github/workflows/pr-js-checks.yml");
  const action = read(".github/actions/test-native-runtime-selection/action.yml");
  const selection = (workflow: { jobs: Record<string, { steps?: Step[] }> }) => {
    const steps = Object.values(workflow.jobs)
      .flatMap((job) => job.steps ?? [])
      .filter((step) => step.uses === "./.github/actions/test-native-runtime-selection");
    assert.equal(steps.length, 1);
    return steps[0]!;
  };
  assert.equal(
    source.jobs["pr-source-plan"].outputs["user-native-runtime-selection"],
    "${{ steps.plan.outputs.user-native-runtime-selection }}",
  );
  const cli = selection(source);
  assert.deepEqual(cli.with, { surface: "cli-lsp" });
  assert.match(
    cli.if!,
    /matrix\.lane == 'lsp' && needs\.pr-source-plan\.outputs\.user-native-runtime-selection == 'true'/,
  );
  const napi = selection(js);
  assert.deepEqual(napi.with, { surface: "napi" });
  assert.match(napi.if!, /inputs\.run-runtime-selection && matrix\.group == 'builders'/);
  const steps: Step[] = action.runs.steps;
  const observe = steps.find((step) => step.env?.VIZE_REQUIRE_NATIVE_RUNTIME_SELECTION);
  assert.equal(observe?.env?.VIZE_REQUIRE_NATIVE_RUNTIME_SELECTION, "1");
  assert.equal(observe?.env?.VIZE_RUNTIME_AUTHORITY_EXPECTATION, "after");
  for (const command of [
    "node --test tests/tooling/native-runtime-cli-selection.test.ts",
    "node tests/tooling/support/native-runtime-selection/authority.ts",
    "node tests/tooling/support/native-runtime-selection/lsp.ts",
    "node --test tests/tooling/native-runtime-napi-selection.test.ts",
  ])
    assert(observe?.run?.includes(command), command);
  assert.equal(
    steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"))?.if,
    "${{ always() }}",
  );
  assert(!steps.some((step) => /cargo build|build:native|build:debug/.test(step.run ?? "")));
  assert(!steps.some((step) => "continue-on-error" in step));
});
