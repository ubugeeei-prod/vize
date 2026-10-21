import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { planSourceChecks } from "../../tools/support/compat/github/plan-source-checks.ts";
import { readRepoFile } from "./support/github-workflows.ts";

type Job = {
  if?: string;
  needs?: string[] | string;
  uses?: string;
  "continue-on-error"?: boolean;
  "timeout-minutes"?: number;
  steps?: Array<{
    if?: string;
    run?: string;
    uses?: string;
    with?: Record<string, unknown>;
    "continue-on-error"?: boolean;
  }>;
};
type Workflow = { jobs: Record<string, Job> };
const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as Workflow;
const dependencyWorkflow = parse(
  readRepoFile(".github", "workflows", "level-dependencies.yml"),
) as Workflow;

test("docs-only dependency policy changes still run a mandatory metadata gate", () => {
  for (const policy of [
    "docs/davinci/plan/level-dependency-allowlist.json",
    "docs/davinci/plan/product-dependency-allowlists/vize_glyph.json",
    "docs/davinci/plan/product-dependency-allowlists/vize_patina.json",
  ])
    assert.deepEqual(planSourceChecks([policy]), {
      rust: true,
      js: true,
      tooling: true,
      playground: true,
    });
  const gate = workflow.jobs?.["level-dependency-direction"];
  assert.ok(gate);
  assert.equal(gate.if, undefined);
  assert.equal(gate.needs, undefined);
  assert.equal(gate.uses, "./.github/workflows/level-dependencies.yml");
  const implementation = dependencyWorkflow.jobs["level-dependencies"];
  assert.equal(implementation.if, undefined);
  assert.equal(implementation.needs, undefined);
  assert.equal(implementation["continue-on-error"], undefined);
  assert.ok(implementation.steps?.length);
  assert.ok(
    implementation.steps?.every(
      (step) => step.if === undefined && step["continue-on-error"] === undefined,
    ),
  );
  assert.match(
    implementation.steps?.map((step) => step.run ?? "").join("\n") ?? "",
    /node tools\/support\/compat\/davinci\/level-dependencies\.ts --base/,
  );
  assert.ok(workflow.jobs?.["test-report"]?.needs?.includes("level-dependency-direction"));
});

test("dependency checks start without a package-manager bootstrap inside their five-minute budget", () => {
  const implementation = dependencyWorkflow.jobs["level-dependencies"];
  assert.equal(implementation["timeout-minutes"], 5);
  const steps = implementation.steps ?? [];
  const setupIndex = steps.findIndex((step) => step.uses?.startsWith("actions/setup-node@"));
  assert.ok(setupIndex >= 0, "set up Node directly before the mandatory checks");
  const setup = steps[setupIndex];
  assert.match(setup.uses ?? "", /^actions\/setup-node@[a-f0-9]{40}$/u);
  assert.equal(setup.with?.["node-version-file"], "package.json");
  assert.equal(setup.with?.["package-manager-cache"], false);
  assert.equal(setup.with?.cache, undefined);
  assert.ok(steps.every((step) => !step.uses?.includes("setup-vp@")));
  for (const command of [
    "node --test tests/tooling/davinci-level-dependencies.test.mjs tests/tooling/davinci-directory-dependencies.test.mjs",
    'node tools/support/compat/davinci/level-dependencies.ts --base "$DEPENDENCY_BASE"',
  ]) {
    assert.ok(steps.findIndex((step) => step.run === command) > setupIndex);
  }
});
