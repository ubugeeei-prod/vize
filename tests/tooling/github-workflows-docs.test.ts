import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";

import {
  requiredReleaseWorkflowEvidence,
  requiredReleaseWorkflows,
  selectRequiredWorkflowRuns,
} from "../../tools/support/compat/github/release-preflight-evidence.mjs";
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
    push?: { branches?: string[]; paths?: string[] };
    schedule?: Array<{ cron: string }>;
    workflow_dispatch?: unknown;
  };

  assert.equal(workflow.name, "Docs build");
  assert.deepEqual(Object.keys(events).sort(), ["push", "schedule", "workflow_dispatch"]);
  assert.deepEqual(events.push?.branches, ["main"]);
  assert.deepEqual(events.push?.paths, [
    ".github/workflows/build-docs.yml",
    "docs/**",
    "playground/**",
    "examples/vite-musea/**",
  ]);
  assert.deepEqual(events.schedule, [{ cron: "41 5 * * *" }]);
  assert.equal(
    workflow.concurrency?.group,
    "docs-build-${{ github.event_name }}-${{ github.ref }}-${{ github.sha }}",
  );
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

test("docs deployment serializes before validating actual source and publication custody", () => {
  const workflow = readWorkflow("deploy-docs.yml");
  const events = workflow.on as {
    workflow_run?: { workflows?: string[]; types?: string[]; branches?: string[] };
  };
  const deploy = workflowJob(workflow, "deploy");
  const checkoutMain = namedStep(deploy, "Checkout current main");
  const compare = namedStep(deploy, "Validate completed build and actual Pages publication floor");

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
  assert.equal(
    deploy.steps?.[2],
    compare,
    "primary custody must be checked before any source checkout or Pages effects",
  );
  assert.equal(compare.env?.GITHUB_TOKEN, "${{ github.token }}");
  assert.match(
    compare.run ?? "",
    /cp tools\/support\/compat\/github\/docs-deployment-\*\.ts "\$RUNNER_TEMP\/"/,
  );
  assert.match(compare.run ?? "", /vp node "\$RUNNER_TEMP\/docs-deployment-entry\.ts" prepare/);
});

test("only a source-custody-qualified build can download or deploy, and success follows actual Pages", () => {
  const workflow = readWorkflow("deploy-docs.yml");
  const deploy = workflowJob(workflow, "deploy");
  const download = namedStep(deploy, "Download docs build artifacts");
  const checkout = namedStep(deploy, "Checkout docs build");
  const pages = namedStep(deploy, "Deploy to GitHub Pages");
  const record = namedStep(deploy, "Record Pages artifact custody");
  const confirm = namedStep(deploy, "Confirm actual Pages publication");
  const eligibility = "${{ steps.main.outputs.eligible == 'true' }}";

  assert.equal(deploy.permissions?.actions, "read");
  assert.equal(deploy.permissions?.pages, "write");
  assert.equal(deploy.permissions?.deployments, "write");
  assert.equal(deploy.permissions?.["id-token"], "write");
  assert.equal(checkout?.with?.ref, "${{ github.event.workflow_run.head_sha }}");
  for (const step of deploy.steps?.slice(3).filter((step) => step !== confirm) ?? []) {
    assert.equal(step.if, eligibility, `${step.name ?? step.uses} bypasses the freshness gate`);
  }
  assert.equal(download.with?.["run-id"], "${{ github.event.workflow_run.id }}");
  assert.equal(download.with?.["artifact-ids"], "${{ steps.main.outputs.artifact_ids }}");
  assert.equal(download.with?.["github-token"], "${{ github.token }}");
  assert.equal(download.with?.path, "artifacts");
  assert.match(
    pages.uses ?? "",
    /^actions\/deploy-pages@d6db90164ac5ed86f2b6aed7e0febac5b3c0c03e$/,
  );
  assert.equal(
    confirm.if,
    "${{ steps.main.outputs.eligible == 'true' && steps.deployment.conclusion == 'success' }}",
  );
  assert(deploy.steps!.indexOf(record) < deploy.steps!.indexOf(pages));
  assert(deploy.steps!.indexOf(confirm) > deploy.steps!.indexOf(pages));
});

test("release preflight requires docs build evidence, never mutable deployment", () => {
  assert.ok(requiredReleaseWorkflows.includes("Docs build"));
  assert.ok(!requiredReleaseWorkflows.includes("Deploy docs"));
  assert.deepEqual(requiredReleaseWorkflowEvidence.get("Docs build"), {
    path: ".github/workflows/build-docs.yml",
    events: ["workflow_dispatch"],
  });

  const runs = requiredReleaseWorkflows.map((name, index) => successfulReleaseRun(name, index + 1));
  const docsBuild = runs.find((run) => run.name === "Docs build");
  assert.ok(docsBuild);
  docsBuild.name = "Deploy docs";
  docsBuild.path = ".github/workflows/deploy-docs.yml";
  docsBuild.event = "workflow_run";
  assert.throws(() => selectRequiredWorkflowRuns(runs, releaseSha), /Docs build: missing/);
});
