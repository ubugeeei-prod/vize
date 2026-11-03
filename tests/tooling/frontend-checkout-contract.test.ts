import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

import {
  proseReferences,
  readsProse,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import { restoreReviewedCheckout } from "./support/frontend-checkout-contract.ts";

type Step = { uses?: string; with?: Record<string, unknown>; [key: string]: unknown };
type Job = { steps: [Step, ...Step[]] };
type Workflow = { jobs: Record<string, Job> };
const current = parse(
  fs.readFileSync(new URL("../../.github/workflows/check.yml", import.meta.url), "utf8"),
) as Workflow;
const archived = parse(
  fs.readFileSync(
    new URL("./fixtures/frontend-runner-pool/check.before.yml.txt", import.meta.url),
    "utf8",
  ),
) as Workflow;

function selected(workflow: Workflow, site: string) {
  const job = workflow.jobs[site];
  assert.ok(job);
  return job;
}
function inputs(job: Job) {
  const value = job.steps[0].with;
  assert.ok(value);
  return value;
}
function paths(job: Job) {
  const value = inputs(job)["sparse-checkout"];
  assert.equal(typeof value, "string");
  return value as string;
}

test("the exact reviewed checkouts invert without changing another runtime step", () => {
  for (const site of ["iteration-plan", "test-report"] as const) {
    const job = structuredClone(selected(current, site));
    restoreReviewedCheckout(job, selected(archived, site), site);
    assert.deepEqual(job.steps[0], selected(archived, site).steps[0]);
    assert.deepEqual(job.steps.slice(1), selected(current, site).steps.slice(1));
  }
});

test("checkout inversion refuses missing, duplicated and unreviewed inputs", () => {
  for (const site of ["iteration-plan", "test-report"] as const) {
    for (const mutate of [
      (job: Job) => {
        delete job.steps[0].with;
      },
      (job: Job) => {
        inputs(job)["fetch-depth"] = 0;
      },
      (job: Job) => {
        inputs(job)["sparse-checkout-cone-mode"] = true;
      },
      (job: Job) => {
        inputs(job).filter = "blob:none";
      },
      (job: Job) => {
        inputs(job)["sparse-checkout"] = paths(job) + "/unowned.rs\n";
      },
      (job: Job) => {
        inputs(job)["sparse-checkout"] = paths(job) + "/package.json\n";
      },
      (job: Job) => {
        inputs(job)["sparse-checkout"] = paths(job).replace("/package.json\n", "");
      },
      (job: Job) => {
        job.steps[0].uses = "actions/checkout@unreviewed";
      },
      (job: Job) => {
        job.steps.unshift(structuredClone(job.steps[0]));
      },
    ]) {
      const job = structuredClone(selected(current, site));
      mutate(job);
      assert.throws(() => restoreReviewedCheckout(job, selected(archived, site), site));
    }
  }
});

test("both workflow inputs explicitly select their whole runner contract guard", () => {
  const source = fs.readFileSync(
    new URL("./frontend-runner-pool.test.ts", import.meta.url),
    "utf8",
  );
  const references = proseReferences(source);
  for (const path of [".github/workflows/check.yml", ".github/workflows/pr-source-checks.yml"])
    assert.equal(readsProse(path, [references]), true, path);
});
