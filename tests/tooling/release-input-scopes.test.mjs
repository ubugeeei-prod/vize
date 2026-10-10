import assert from "node:assert/strict";
import { test } from "node:test";
import { toolingTestScopes } from "../../tools/config/vite-plus/tooling-test-scopes.ts";
import {
  localImportInputs,
  planToolingTests,
  toolingTestFiles,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const releaseFiles = toolingTestFiles().filter((file) => file.startsWith("tests/tooling/release/"));
const scoped = toolingTestScopes.find((entry) => entry.input === "toolingReleaseContracts")?.tests;
assert.ok(scoped);
const unscoped = releaseFiles.filter((file) => !scoped.includes(file));
const selected = (path) =>
  planToolingTests([path]).tests.filter((file) => file.startsWith("tests/tooling/release/"));

void test("audited release contracts have complete imports and leave unresolved cases broad", () => {
  assert.equal(releaseFiles.length, 37);
  assert.equal(scoped.length, 31);
  assert.deepEqual(unscoped, [
    "tests/tooling/release/release-guest-locks.test.ts",
    "tests/tooling/release/release-integration-catalog.test.ts",
    "tests/tooling/release/release-preflight-full-js.test.ts",
    "tests/tooling/release/release-public-native-invocation.test.ts",
    "tests/tooling/release/release-smoke-init-fresh.test.ts",
    "tests/tooling/release/release-smoke-init-typecheck.test.ts",
  ]);
  for (const file of scoped) {
    assert.ok(releaseFiles.includes(file), `${file} must exist`);
    assert.equal(localImportInputs(file).complete, true, `${file} import closure`);
  }
  assert.deepEqual(selected("davinci/vize_l1/src/parser.rs"), unscoped);
});

void test("installed VRT observer inputs select both complete audited contracts", () => {
  const contracts = [
    "tests/tooling/release/release-public-vrt-observer.test.ts",
    "tests/tooling/release/release-public-vrt-reports.test.ts",
  ];
  for (const contract of contracts) {
    assert.ok(scoped.includes(contract));
    assert.equal(localImportInputs(contract).complete, true, contract);
  }
  for (const input of [
    ...["api", "artifacts", "audit", "cli", "fixtures", "host", "hosted", "observer"].map(
      (name) => `tools/support/release/public_acceptance/vrt_${name}.ts`,
    ),
    "tools/support/release/public_acceptance/accept.ts",
    ".github/workflows/release-public-acceptance.yml",
    "tests/tooling/fixtures/musea/snapshot-collision/left/Button.art.vue",
    "tests/tooling/fixtures/musea/snapshot-collision/right/Button.art.vue",
    "tests/_fixtures/differential/musea/gallery-vrt-options.json",
    "tests/_fixtures/differential/musea/hosted-audits.json",
    "tests/_fixtures/differential/musea/vrt-report-ownership.json",
  ]) {
    const tests = selected(input);
    for (const contract of contracts) assert.ok(tests.includes(contract), `${input}: ${contract}`);
  }
});

void test("Matrix attempt selectors select the new audited evidence contract", () => {
  const contract = "tests/tooling/release/release-preflight-matrix-selection.test.ts";
  assert.ok(scoped.includes(contract));
  for (const input of [
    "tools/support/release/preflight_matrix_selection.rs",
    "tools/support/compat/github/release-preflight-matrix-selection.mjs",
    "tests/tooling/support/release-matrix-attempt-fixture.ts",
  ]) {
    assert.ok(selected(input).includes(contract), input);
  }
});

void test("hosted operator source and authority inputs select its audited contract", () => {
  const contract = "tests/tooling/release/release-operator.test.ts";
  assert.ok(scoped.includes(contract));
  for (const input of [
    ".github/workflows/release-operator.yml",
    "tools/commands/release/pr.rs",
    "tools/support/release/pr_start.rs",
    "tools/support/release/pr_pin_watch.rs",
    "tools/support/release/pr_pin_retire.rs",
    "tools/support/release/pr_pin_retire_archive.rs",
    "tools/support/release/pr_pin_retire_ledger.rs",
    "tools/support/release/pr_pin_retire_absence.rs",
    "tools/support/release/pr_pin_retire_release.rs",
    "tools/support/release/pr_budget.rs",
    "tools/support/release/pr_pin_retire_guard.rs",
    "tools/support/release/pr_pin_retire_attempt_fixtures.rs",
    "tools/moon/cmd/release/main.mbt",
    "tests/tooling/support/fake-command.ts",
  ]) {
    assert.ok(selected(input).includes(contract), input);
  }
});

void test("public absence source and copied fixtures select the existing audited contract", () => {
  const contract = "tests/tooling/release/release-public-acceptance.test.ts";
  for (const input of [
    "tools/support/release/retirement_absence.ts",
    "tools/support/release/retirement_evidence.ts",
    "tests/tooling/support/release-retirement-absence-fixtures.ts",
    "tests/tooling/support/release-retirement-marketplace-fixtures.ts",
  ]) {
    assert.ok(selected(input).includes(contract), input);
  }
});

void test("release scripts, manifests, workflow, docs and copied fixture restore every contract", () => {
  for (const path of [
    "tools/commands/release/pr.rs",
    "tools/support/compat/npm/smoke-release-runtime.mjs",
    ".github/workflows/release.yml",
    "npm/cli/package.json",
    "editors/vscode/package.json",
    "Cargo.toml",
    "docs/content/guide/init.md",
    "crates/vize/tests/fixtures/content_mapper_project/src/App.vue",
  ]) {
    assert.deepEqual(selected(path), releaseFiles, path);
  }
});

void test("direct, global and unknown changes remain broad; merge runs every release test", () => {
  for (const path of [
    "tests/tooling/release/release-pr.test.ts",
    "pnpm-lock.yaml",
    "new-root/unknown.ts",
  ]) {
    const plan = planToolingTests([path]);
    if (path.endsWith("release-pr.test.ts")) {
      assert.ok(plan.tests.includes(path));
    } else {
      assert.deepEqual(
        plan.tests.filter((file) => releaseFiles.includes(file)),
        releaseFiles,
      );
    }
  }
  const merge = planToolingTests(["davinci/vize_l1/src/parser.rs"], { tier: "merge" });
  assert.deepEqual(
    merge.tests.filter((file) => releaseFiles.includes(file)),
    releaseFiles,
  );
});
