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
  "tests/tooling/support/lint-report-parity.ts",
  "tests/_fixtures/differential/linter/reference-coverage-8849/report-references.json",
  "tests/tooling/lint-reference-coverage-workflow.test.ts",
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
