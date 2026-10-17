import assert from "node:assert/strict";
import { matchesGlob } from "node:path";
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

const configFreeInitContract = "tests/tooling/release/release-smoke-init-config-free.test.ts";
const deliveredInitContracts = [
  "tests/tooling/release/release-smoke-init-child-outcome.test.ts",
  "tests/tooling/release/release-smoke-init-js-manager-plans.test.ts",
  "tests/tooling/release/release-smoke-init-yarn-checkjs.test.ts",
];
const installedReportedContracts = ["tests/tooling/release/release-public-vapor.test.ts"];
const stablePatchContracts = ["tests/tooling/release/release-stable-operator.test.ts"];
const sharedPublisherContracts = ["tests/tooling/release/release-public-shared-publisher.test.ts"];
const installedSmokeMigrationContracts = [
  "tests/tooling/release/release-smoke-parity-peer-copy.test.ts",
  "tests/tooling/release/release-smoke-public-native.test.ts",
  "tests/tooling/release/release-smoke-typescript-parity.test.ts",
  "tests/tooling/release/release-smoke-typescript-runtime.test.ts",
];
// The immutable generated consumer imports its installed native provider dynamically.
// Its two callers must retain the planner's conservative broad fallback.
const dynamicPublisherContracts = [
  "tests/tooling/release/release-public-acceptance.test.ts",
  ...sharedPublisherContracts,
];

const installedHttpsContracts = [
  "tests/tooling/release/release-public-quick-tunnel.test.ts",
  "tests/tooling/release/release-public-trusted-vrt-color.test.ts",
  "tests/tooling/release/release-public-trusted-vrt.test.ts",
  "tests/tooling/release/release-public-vrt-announcement.test.ts",
  "tests/tooling/release/release-public-vrt-literal-inputs.test.ts",
  "tests/tooling/release/release-public-vrt-log-barrier.test.ts",
  "tests/tooling/release/release-public-vrt-tunnel-inputs.test.ts",
  "tests/tooling/release/release-public-vrt-upload-refusal.test.ts",
];

void test("audited release contracts have complete imports and leave unresolved cases broad", () => {
  const historicalUnscoped = [
    "tests/tooling/release/legacy-check-qualification.test.ts",
    "tests/tooling/release/release-integration-catalog.test.ts",
    "tests/tooling/release/release-notes.test.ts",
    "tests/tooling/release/release-preflight-full-js.test.ts",
    "tests/tooling/release/release-public-native-invocation.test.ts",
    "tests/tooling/release/release-smoke-init-fresh.test.ts",
    "tests/tooling/release/release-smoke-init-typecheck.test.ts",
  ];
  const expectedUnscoped = [
    ...historicalUnscoped,
    ...deliveredInitContracts,
    configFreeInitContract,
    ...installedHttpsContracts,
    ...installedReportedContracts,
    ...stablePatchContracts,
    ...sharedPublisherContracts,
    ...installedSmokeMigrationContracts,
  ].sort();
  const inventory = [...scoped, ...expectedUnscoped].sort();
  assert.equal(new Set(inventory).size, inventory.length, "contracts belong to exactly one scope");
  assert.deepEqual(unscoped, expectedUnscoped, "unreviewed contracts retain broad inputs");
  assert.deepEqual(
    releaseFiles,
    inventory,
    "every release contract has an explicit inventory entry",
  );
  for (const file of scoped) {
    assert.ok(releaseFiles.includes(file), `${file} must exist`);
    assert.equal(
      localImportInputs(file).complete,
      !dynamicPublisherContracts.includes(file),
      `${file} import closure`,
    );
  }
  assert.deepEqual(
    selected("davinci/vize_l1/src/parser.rs"),
    [...new Set([...unscoped, ...dynamicPublisherContracts])].sort(),
  );
});

void test("installed smoke migration contracts retain unreviewed broad inputs", () => {
  const plans = ["davinci/vize_l1/src/parser.rs", "crates/vize_maestro/src/server.rs"].map(
    (input) => ({ input, plan: planToolingTests([input]) }),
  );
  for (const contract of installedSmokeMigrationContracts) {
    assert.equal(
      toolingTestScopes.some((entry) =>
        entry.tests.some((pattern) => matchesGlob(contract, pattern)),
      ),
      false,
      `${contract} has no audited scope registration`,
    );
    const imports = localImportInputs(contract);
    for (const { input, plan } of plans) {
      assert.equal(imports.files.includes(input), false, `${contract}: independent input`);
      assert.equal(plan.reason, "declared task and import inputs", input);
      assert.ok(plan.tests.includes(contract), `${input}: ${contract} retains broad fallback`);
    }
  }
});

void test("shared publisher authority retains broad inputs across the dynamic consumer oracle", () => {
  for (const contract of dynamicPublisherContracts) {
    const imports = localImportInputs(contract);
    assert.equal(
      imports.complete,
      false,
      "the generated consumer oracle has a dynamic native import",
    );
    for (const input of [
      "tools/support/release/public_acceptance/plan.ts",
      "tools/support/release/consumer_recovery/oracles.ts",
      "tests/tooling/support/release-public-acceptance-fixtures.ts",
      "tests/tooling/support/github-workflows.ts",
    ]) {
      assert.ok(imports.files.includes(input), `${contract}: ${input}`);
    }
    for (const input of [
      ".github/actions/publish-release-package/action.yml",
      "tools/support/release/consumer_recovery/oracles.ts",
      "tests/tooling/support/release-public-acceptance-fixtures.ts",
    ]) {
      assert.ok(selected(input).includes(contract), input);
    }
  }
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

void test("Matrix enforcement inputs select the complete actual-runtime contract", () => {
  const contract = "tests/tooling/release/release-matrix-enforcement.test.ts";
  assert.ok(scoped.includes(contract));
  assert.equal(localImportInputs(contract).complete, true);
  for (const input of [
    ".github/workflows/real-project-matrix.yml",
    "tools/commands/fixtures/real-project-surface-verdict.rs",
    "tools/commands/ci/github/release-preflight.rs",
    "tools/support/release/preflight_surface_evidence.rs",
    "tools/support/compat/github/release-preflight-surface-evidence.mjs",
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
    "tools/support/release/release/main.ts",
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
    "tools/support/release/retirement_transport.ts",
    "tests/tooling/support/release-retirement-absence-fixtures.ts",
    "tests/tooling/support/release-retirement-transport-law.ts",
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
    "crates/vize/tests/fixtures/content_mapper_project/src/App.vue",
  ]) {
    assert.deepEqual(selected(path), releaseFiles, path);
  }
  // Prose selects the contracts that declare it; the merge queue runs the rest.
  const init = selected("docs/content/guide/init.md");
  for (const file of scoped) assert.ok(init.includes(file), file);
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
