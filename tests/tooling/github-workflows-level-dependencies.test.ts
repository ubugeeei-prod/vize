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
  steps?: Array<{ if?: string; run?: string; "continue-on-error"?: boolean }>;
};
type Workflow = { jobs: Record<string, Job> };
const workflow = parse(readRepoFile(".github", "workflows", "check.yml")) as Workflow;
const dependencyWorkflow = parse(
  readRepoFile(".github", "workflows", "level-dependencies.yml"),
) as Workflow;

test("docs-only dependency policy changes still run a mandatory metadata gate", () => {
  assert.deepEqual(planSourceChecks(["docs/davinci/plan/level-dependency-allowlist.json"]), {
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
    /node tools\/support\/compat\/davinci\/level-dependencies\.mjs --base/,
  );
  assert.ok(workflow.jobs?.["test-report"]?.needs?.includes("level-dependency-direction"));
});
