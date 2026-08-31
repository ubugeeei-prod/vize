import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  requiredReleaseWorkflowEvidence,
  requiredReleaseWorkflows,
  selectRequiredWorkflowRuns,
} from "../../legacy-tools/github/release-preflight-evidence.mjs";
import { readRepoFile } from "./support/github-workflows.ts";
import { releaseSha, successfulReleaseRun } from "./support/release-preflight.ts";

interface WorkflowStep {
  name?: string;
  id?: string;
  if?: string;
  uses?: string;
  env?: Record<string, string>;
  run?: string;
  with?: Record<string, unknown>;
}

interface WorkflowJob {
  if?: string;
  needs?: string;
  concurrency?: { group?: string; queue?: string; "cancel-in-progress"?: boolean };
  permissions?: Record<string, string>;
  outputs?: Record<string, string>;
  steps?: WorkflowStep[];
}

interface Workflow {
  name?: string;
  on?: Record<string, unknown>;
  concurrency?: { group?: string; "cancel-in-progress"?: boolean };
  permissions?: Record<string, string>;
  jobs?: Record<string, WorkflowJob>;
}

function readWorkflow(file: string): Workflow {
  return parse(readRepoFile(".github", "workflows", file)) as Workflow;
}

function workflowJob(workflow: Workflow, name: string): WorkflowJob {
  const job = workflow.jobs?.[name];
  assert.ok(job, `missing workflow job ${name}`);
  return job;
}

function namedStep(job: WorkflowJob, name: string): WorkflowStep {
  const step = job.steps?.find((candidate) => candidate.name === name);
  assert.ok(step, `missing workflow step ${name}`);
  return step;
}

test("docs build evidence is immutable per SHA and has no Pages authority", () => {
  const workflow = readWorkflow("build-docs.yml");
  const events = workflow.on as {
    push?: { branches?: string[] };
    workflow_dispatch?: unknown;
  };

  assert.equal(workflow.name, "Docs build");
  assert.deepEqual(Object.keys(events).sort(), ["push", "workflow_dispatch"]);
  assert.deepEqual(events.push?.branches, ["main"]);
  assert.equal(workflow.concurrency?.group, "docs-build-${{ github.sha }}");
  assert.equal(workflow.concurrency?.["cancel-in-progress"], true);
  assert.deepEqual(Object.keys(workflow.jobs ?? {}).sort(), ["build-docs", "build-playground"]);
  assert.equal(workflow.permissions?.contents, "read");

  const source = readRepoFile(".github", "workflows", "build-docs.yml");
  assert.doesNotMatch(source, /pages:\s*write|id-token:\s*write|actions\/deploy-pages/);
  for (const artifact of ["docs", "playground", "musea-examples"]) {
    const upload = Object.values(workflow.jobs ?? {})
      .flatMap((job) => job.steps ?? [])
      .find((step) => step.with?.name === artifact);
    assert.ok(upload, `missing ${artifact} artifact upload`);
    assert.match(upload.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
    assert.equal(upload.with?.["if-no-files-found"], "error");
  }
});

test("docs deployment serializes before revalidating current main", () => {
  const workflow = readWorkflow("deploy-docs.yml");
  const events = workflow.on as {
    workflow_run?: { workflows?: string[]; types?: string[]; branches?: string[] };
  };
  const deploy = workflowJob(workflow, "deploy");
  const checkoutMain = namedStep(deploy, "Checkout current main");
  const compare = namedStep(deploy, "Compare docs build with current main");

  assert.equal(workflow.name, "Deploy docs");
  assert.deepEqual(Object.keys(events), ["workflow_run"]);
  assert.deepEqual(events.workflow_run?.workflows, ["Docs build"]);
  assert.deepEqual(events.workflow_run?.types, ["completed"]);
  assert.deepEqual(events.workflow_run?.branches, ["main"]);
  assert.deepEqual(Object.keys(workflow.jobs ?? {}), ["deploy"]);
  assert.equal(workflow.concurrency, undefined);
  assert.equal(deploy.if, "${{ github.event.workflow_run.conclusion == 'success' }}");
  assert.equal(deploy.concurrency?.group, "pages-main");
  assert.equal(
    deploy.concurrency?.queue,
    "max",
    "a stale late arrival must not replace a newer pending deployment",
  );
  assert.equal(
    deploy.concurrency?.["cancel-in-progress"],
    false,
    "an older completed build must never cancel a newer deployment",
  );
  assert.equal(checkoutMain.with?.ref, "main");
  assert.equal(
    deploy.steps?.[0],
    checkoutMain,
    "main must be fetched after concurrency is acquired",
  );
  assert.equal(deploy.steps?.[1], compare, "the freshness check must open the deploy section");
  assert.equal(compare.env?.DOCS_BUILD_SHA, "${{ github.event.workflow_run.head_sha }}");
  assert.match(compare.run ?? "", /\^\[0-9a-f\]\{40\}\$/);
  assert.match(compare.run ?? "", /CURRENT_MAIN_SHA="\$\(git rev-parse HEAD\)"/);
  assert.match(compare.run ?? "", /\[\[ "\$DOCS_BUILD_SHA" == "\$CURRENT_MAIN_SHA" \]\]/);
  assert.match(compare.run ?? "", /eligible=false/);
});

test("only the exact current-main build can download or deploy Pages", () => {
  const workflow = readWorkflow("deploy-docs.yml");
  const deploy = workflowJob(workflow, "deploy");
  const download = namedStep(deploy, "Download docs build artifacts");
  const checkout = namedStep(deploy, "Checkout docs build");
  namedStep(deploy, "Deploy to GitHub Pages");
  const eligibility = "${{ steps.main.outputs.eligible == 'true' }}";

  assert.equal(deploy.permissions?.actions, "read");
  assert.equal(deploy.permissions?.pages, "write");
  assert.equal(deploy.permissions?.["id-token"], "write");
  assert.equal(checkout?.with?.ref, "${{ github.event.workflow_run.head_sha }}");
  for (const step of deploy.steps?.slice(2) ?? []) {
    assert.equal(step.if, eligibility, `${step.name ?? step.uses} bypasses the freshness gate`);
  }
  assert.equal(download.with?.["run-id"], "${{ github.event.workflow_run.id }}");
  assert.equal(download.with?.["github-token"], "${{ github.token }}");
  assert.equal(download.with?.path, "artifacts");
});

test("release preflight requires docs build evidence, never mutable deployment", () => {
  assert.ok(requiredReleaseWorkflows.includes("Docs build"));
  assert.ok(!requiredReleaseWorkflows.includes("Deploy docs"));
  assert.deepEqual(requiredReleaseWorkflowEvidence.get("Docs build"), {
    path: ".github/workflows/build-docs.yml",
    events: ["push", "workflow_dispatch"],
    branches: { push: ["main"] },
  });

  const runs = requiredReleaseWorkflows.map((name, index) => successfulReleaseRun(name, index + 1));
  const docsBuild = runs.find((run) => run.name === "Docs build");
  assert.ok(docsBuild);
  docsBuild.name = "Deploy docs";
  docsBuild.path = ".github/workflows/deploy-docs.yml";
  docsBuild.event = "workflow_run";
  assert.throws(() => selectRequiredWorkflowRuns(runs, releaseSha), /Docs build: missing/);
});
