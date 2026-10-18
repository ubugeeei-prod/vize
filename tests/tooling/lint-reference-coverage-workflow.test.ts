import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { matchesGlob } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

interface Step {
  name?: string;
  uses?: string;
  run?: string;
  if?: string;
  "continue-on-error"?: boolean;
  with?: Record<string, unknown>;
}
interface Workflow {
  on: { pull_request: { paths: string[] } };
  permissions: { contents: string };
  jobs: { contracts: { "timeout-minutes": number; steps: Step[] } };
}
const workflow = parse(
  readFileSync(
    new URL("../../.github/workflows/lint-reference-coverage-contracts.yml", import.meta.url),
    "utf8",
  ),
) as Workflow;
const sources = [
  "tools/commands/fixtures/lint-divergence-report.ts",
  "tools/support/fixtures/lint_reference_coverage.ts",
  "tools/support/fixtures/lint_divergence/selection.ts",
  "tests/tooling/_helpers/lint-report-rust-parity.ts",
  "tests/tooling/_helpers/lint-report-schema.ts",
  "tests/tooling/support/lint-report-parity.ts",
  "tests/_fixtures/differential/linter/reference-coverage-8849/report-references.json",
  "tests/tooling/lint-reference-coverage-workflow.test.ts",
  "tests/tooling/docs-all-lint-rules.test.ts",
  "docs/scripts/generate-patina-rules-page.ts",
  "tests/tooling/support/docs-category-sources.ts",
  "tests/tooling/support/docs-rule-overview-sources.ts",
  "docs/scripts/navigation-render-routes.ts",
  "docs/scripts/navigation-render-concurrency.ts",
  "docs/scripts/materialize-content.ts",
  "docs/scripts/rule-render-assertions.ts",
  "docs/scripts/rules/render.ts",
  "docs/content/rules/all.md",
  "docs/content/ja/rules/all.md",
  "docs/content/generated/rules/fr/html.md",
  "crates/vize_patina/src/rule.rs",
  "crates/vize_patina/src/rule/happy_path.rs",
  "crates/vize_patina/src/rules/opinionated/html.rs",
  "crates/vize_patina/src/preset.rs",
  "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap",
  "crates/vize_patina/src/linter/script_rules/registry.rs",
  "crates/vize_patina/src/linter/script_rules/registry/rules.rs",
  "tests/_fixtures/lint-preset-default-warnings/source-retention-successors.json",
];
function requireSourceTriggers(paths: string[]) {
  for (const source of sources)
    assert.ok(
      paths.some((pattern) => matchesGlob(source, pattern)),
      `unqualified source-only change: ${source}`,
    );
}

test("helper-only and authority-only changes cannot bypass actual report parity", () => {
  const paths = workflow.on.pull_request.paths;
  requireSourceTriggers(paths);
  assert.throws(
    () => requireSourceTriggers(paths.filter((pattern) => !pattern.includes("_helpers"))),
    /unqualified source-only change: tests\/tooling\/_helpers/,
  );
  assert.throws(
    () =>
      requireSourceTriggers(
        paths.filter((pattern) => !pattern.includes("reference-coverage-8849")),
      ),
    /unqualified source-only change: tests\/_fixtures/,
  );
});

test("exact authored source must earn strict parity before retained evidence can pass", () => {
  assert.deepEqual(workflow.permissions, { contents: "read" });
  const job = workflow.jobs.contracts;
  assert.ok(job["timeout-minutes"] <= 15);
  const checkout = job.steps.find((step) => step.uses?.startsWith("actions/checkout@"));
  assert.ok(checkout?.with);
  assert.equal(checkout.with.ref, "${{ github.event.pull_request.head.sha }}");
  assert.equal(checkout.with["persist-credentials"], false);
  const parity = job.steps.find((step) =>
    step.run?.includes("--test tests/tooling/support/lint-report-parity.ts"),
  );
  assert.ok(parity);
  assert.equal(parity["continue-on-error"], undefined);
  assert.equal(parity.if, undefined);
  assert.match(parity.run ?? "", /^env -u GITHUB_SHA node --test /);
  const reference = job.steps.find((step) =>
    step.run?.startsWith("git fetch --no-tags --depth=1 origin"),
  );
  assert.ok(reference);
  const refs = reference.run?.match(/[a-f0-9]{40}/g);
  assert.deepEqual(refs, [
    "49545646fde08ade1000c4a33b38c9dd1ac4fb99",
    "7421901d05ac2e252636dd8ee426e8783255378d",
  ]);
  const upload = job.steps.find((step) => step.uses?.startsWith("actions/upload-artifact@"));
  assert.ok(upload);
  assert.equal(upload.if, "${{ always() }}");
  assert.equal(upload.with?.["if-no-files-found"], "error");
});

test("native preset documentation earns its whole law without dropping earlier contracts", () => {
  const name = "tests/tooling/docs-all-lint-rules.test.ts";
  const contracts = workflow.jobs.contracts.steps.find((step) => step.run?.includes(name));
  assert.ok(contracts?.run);
  assert.equal(contracts.if, undefined);
  assert.equal(contracts["continue-on-error"], undefined);
  const expected = [
    "tests/tooling/lint-reference-classifier.test.ts",
    "tests/tooling/lint-reference-coverage-workflow.test.ts",
    "tests/tooling/lint-reference-coverage.test.mjs",
    "tests/tooling/lint-baseline-script-language.test.mjs",
    "tests/tooling/lint-preset-default-warnings-source.test.mjs",
    "tests/tooling/patina-legacy-instance-api-reference.test.mjs",
    "tests/tooling/lint-divergence-processor.test.mjs",
    "tests/tooling/lint-divergence-raw-errors.test.mjs",
    "tests/tooling/lint-divergence-surface-classes.test.ts",
    "tests/tooling/lint-divergence-pug-surface.test.ts",
    "tests/tooling/lint-divergence-zero-vue-runner.test.ts",
    "tests/tooling/lint-divergence-report-runner.test.ts",
    "tests/tooling/lint-divergence-report.test.ts",
    "tests/tooling/real-project-matrix-workflow.test.ts",
    name,
  ];
  function requireWholeCommand(command: string) {
    assert.match(command, /^env -u GITHUB_SHA node --test /);
    assert.deepEqual(command.match(/tests\/tooling\/[\w./-]+\.test\.(?:ts|mjs)/g), expected);
  }
  requireWholeCommand(contracts.run);
  for (const changed of [
    contracts.run.replace(expected[0], ""),
    contracts.run.replace(expected[0], "tests/tooling/unrelated.test.ts"),
    contracts.run.replace("env -u GITHUB_SHA node --test", "echo"),
  ])
    assert.throws(() => requireWholeCommand(changed));
  for (const omitted of [
    name,
    "docs/scripts/generate-patina-rules-page.ts",
    "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap",
  ])
    assert.throws(
      () =>
        requireSourceTriggers(
          workflow.on.pull_request.paths.filter((pattern) => !matchesGlob(omitted, pattern)),
        ),
      /unqualified source-only change/,
    );
});
