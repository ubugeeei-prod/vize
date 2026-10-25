import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
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
    "tests/tooling/native-runtime-cli-selection.test.mjs",
    "crates/vize_maestro/src/ide/diagnostics.rs",
    "crates/vize_maestro/src/ide/diagnostics/typecheck_unavailable_tests.rs",
    "tests/tooling/native-runtime-napi-selection.test.mjs",
    "tests/tooling/support/native-runtime-selection/lsp.mjs",
    ".github/actions/test-native-runtime-selection/action.yml",
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
  const read = (file: string) => readFileSync(new URL(`../../${file}`, import.meta.url), "utf8");
  const source = read(".github/workflows/pr-source-checks.yml");
  const js = read(".github/workflows/pr-js-checks.yml");
  const action = read(".github/actions/test-native-runtime-selection/action.yml");
  assert(
    source.includes(
      "user-native-runtime-selection: ${{ steps.plan.outputs.user-native-runtime-selection }}",
    ),
  );
  assert(source.includes("with: { surface: cli-lsp }"));
  assert(js.includes("inputs.run-runtime-selection && matrix.group == 'builders'"));
  assert(js.includes("with: { surface: napi }"));
  assert(action.includes('VIZE_REQUIRE_NATIVE_RUNTIME_SELECTION: "1"'));
  assert(action.includes("node --test tests/tooling/native-runtime-cli-selection.test.mjs"));
  assert(action.includes("node tests/tooling/support/native-runtime-selection/lsp.mjs"));
  assert(action.includes("node --test tests/tooling/native-runtime-napi-selection.test.mjs"));
  assert(action.includes("if: ${{ always() }}"));
  assert(!/cargo build|build:native|build:debug|continue-on-error/.test(action));
});
