import assert from "node:assert/strict";
import { GitHub, type Deployment } from "./docs-deployment-api.ts";
import {
  custodyEnvironment,
  custodyTask,
  legacyPublisher,
  qualifyDeployment,
  qualifyLegacy,
  type Floor,
} from "./docs-deployment-custody.ts";
import {
  validateRun,
  type OutstandingWriters,
  type PublisherJob,
  type Receipt,
  type WorkflowRun,
} from "./docs-deployment-policy.ts";
import {
  metadataPages,
  verifyPendingWriter,
  verifySolePagesWriter,
} from "./docs-deployment-writers.ts";

type HistoryRun = WorkflowRun & { created_at: string };
type Effect = { publisher: WorkflowRun; job: PublisherJob };
const pagesStep = (job: PublisherJob) =>
  job.steps.find((step) => step.name === "Deploy to GitHub Pages");

async function receiptAnchor(api: GitHub, repositoryId: number, currentRunId: number) {
  const candidates: Effect[] = [];
  for await (const records of metadataPages<Deployment>(
    api,
    "/deployments?environment=" + custodyEnvironment + "&task=" + custodyTask,
  )) {
    for (const record of records) {
      const receipt = record.payload as Receipt;
      assert(
        receipt?.schema === custodyTask && receipt.publisher,
        "Whole candidate receipt identity",
      );
      if (receipt.publisher.runId === currentRunId) continue;
      const [publisher, job] = await Promise.all([
        api.json<WorkflowRun>(
          "/actions/runs/" + receipt.publisher.runId + "/attempts/" + receipt.publisher.attempt,
        ),
        api.json<PublisherJob>("/actions/jobs/" + receipt.publisher.jobId),
      ]);
      validateRun(publisher, repositoryId, ".github/workflows/deploy-docs.yml");
      assert.equal(publisher.event, "workflow_run");
      assert.equal(job.run_id, publisher.id);
      assert.equal(job.run_attempt, publisher.run_attempt);
      assert.equal(job.head_sha, publisher.head_sha);
      assert.equal(job.steps.filter((step) => step.name === "Deploy to GitHub Pages").length, 1);
      if (pagesStep(job)!.conclusion === "success") {
        const floor = await qualifyDeployment(api, repositoryId, record, publisher, job);
        return { floor, candidates };
      }
      candidates.push({ publisher, job });
    }
  }
  return { floor: null, candidates };
}

async function observeHistory(
  api: GitHub,
  repositoryId: number,
  currentRunId: number,
  outstanding: OutstandingWriters,
  guardedWorkflowHash: string | null,
) {
  const runs = await api.list<HistoryRun>(
    "/actions/workflows/deploy-docs.yml/runs?branch=main&created=" +
      encodeURIComponent(">=" + outstanding.since),
    "workflow_runs",
  );
  const attempts = new Map<string, { runId: number; attempt: number }>();
  for (const run of runs)
    for (let attempt = 1; attempt <= run.run_attempt; attempt++)
      attempts.set(run.id + ":" + attempt, { runId: run.id, attempt });
  for (const item of outstanding.publisherAttempts)
    attempts.set(item.runId + ":" + item.attempt, item);
  const effects: Effect[] = [];
  const pending: OutstandingWriters["publisherAttempts"] = [];
  const knownRuns = new Set<number>();
  for (const item of attempts.values()) {
    if (item.runId === currentRunId) continue;
    const publisher = await api.json<WorkflowRun>(
      "/actions/runs/" + item.runId + "/attempts/" + item.attempt,
    );
    validateRun(publisher, repositoryId, ".github/workflows/deploy-docs.yml");
    assert.equal(publisher.id, item.runId);
    assert.equal(publisher.run_attempt, item.attempt);
    assert.equal(publisher.event, "workflow_run");
    knownRuns.add(publisher.id);
    const jobs = await api.list<PublisherJob>(
      "/actions/runs/" + item.runId + "/attempts/" + item.attempt + "/jobs",
      "jobs",
    );
    if (publisher.status !== "completed") {
      await verifyPendingWriter(api, publisher, repositoryId, guardedWorkflowHash);
      pending.push(item);
    }
    for (const job of jobs) {
      assert.equal(job.run_id, publisher.id);
      assert.equal(job.run_attempt, publisher.run_attempt);
      assert.equal(job.head_sha, publisher.head_sha);
      assert(job.steps.filter((step) => step.name === "Deploy to GitHub Pages").length <= 1);
      const step = pagesStep(job);
      if (
        !step ||
        step.conclusion === "skipped" ||
        (step.conclusion === null && step.started_at === null)
      )
        continue;
      effects.push({ publisher, job });
    }
  }
  return { effects, pending, knownRuns };
}

export async function publicationState(
  api: GitHub,
  repositoryId: number,
  currentRunId: number,
  guardedWorkflowHash: string | null = null,
  configured = true,
) {
  if (!configured) {
    const since = "1970-01-01T00:00:00Z";
    const journal = { since, publisherAttempts: [], environmentIds: [] };
    const { effects, pending, knownRuns } = await observeHistory(
      api,
      repositoryId,
      currentRunId,
      journal,
      guardedWorkflowHash,
    );
    assert(
      effects.every(({ job }) => {
        const step = pagesStep(job)!;
        return (
          step.conclusion !== null &&
          step.completed_at &&
          Number.isFinite(Date.parse(step.completed_at))
        );
      }),
      "An uncertain actual Pages effect refuses publication even without a configured site",
    );
    const environmentIds = await verifySolePagesWriter(
      api,
      since,
      knownRuns,
      currentRunId,
      [],
      null,
      null,
      repositoryId,
      guardedWorkflowHash,
    );
    return { floor: null, outstanding: { since, publisherAttempts: pending, environmentIds } };
  }
  const { floor: anchor, candidates } = await receiptAnchor(api, repositoryId, currentRunId);
  let journal: OutstandingWriters;
  if (anchor) journal = anchor.receipt.outstanding;
  else {
    const original = await api.json<HistoryRun>("/actions/runs/" + legacyPublisher);
    validateRun(original, repositoryId, ".github/workflows/deploy-docs.yml");
    assert.equal(original.run_attempt, 1);
    journal = {
      since: original.created_at,
      publisherAttempts: [{ runId: original.id, attempt: 1 }],
      environmentIds: [],
    };
  }
  const { effects, pending, knownRuns } = await observeHistory(
    api,
    repositoryId,
    currentRunId,
    journal,
    guardedWorkflowHash,
  );
  for (const effect of candidates) {
    if (!effects.some((item) => item.job.id === effect.job.id)) effects.push(effect);
    knownRuns.add(effect.publisher.id);
    if (
      effect.publisher.status !== "completed" &&
      !pending.some(
        (item) =>
          item.runId === effect.publisher.id && item.attempt === effect.publisher.run_attempt,
      )
    ) {
      await verifyPendingWriter(api, effect.publisher, repositoryId, guardedWorkflowHash);
      pending.push({ runId: effect.publisher.id, attempt: effect.publisher.run_attempt });
    }
  }
  let floor: Floor | null = anchor;
  const successes = effects.filter(({ job }) => pagesStep(job)?.conclusion === "success");
  successes.sort(
    (a, b) =>
      Date.parse(pagesStep(b.job)!.completed_at!) - Date.parse(pagesStep(a.job)!.completed_at!),
  );
  if (!floor && successes.length)
    floor = await qualifyLegacy(api, repositoryId, successes[0].publisher, successes[0].job);
  for (const effect of effects) {
    const step = pagesStep(effect.job)!;
    if (floor && effect.job.id === floor.receipt.publisher.jobId) continue;
    assert(
      step.completed_at &&
        Number.isFinite(Date.parse(step.completed_at)) &&
        floor &&
        Date.parse(step.completed_at) < Date.parse(floor.completedAt),
      "A current or uncertain later Pages effect lacks authenticated publication custody",
    );
  }
  if (floor) knownRuns.add(floor.receipt.publisher.runId);
  const environmentIds = await verifySolePagesWriter(
    api,
    journal.since,
    knownRuns,
    currentRunId,
    journal.environmentIds,
    floor?.completedAt ?? null,
    floor?.receipt.publisher.jobId ?? null,
    repositoryId,
    guardedWorkflowHash,
  );
  return {
    floor,
    outstanding: {
      since: journal.since,
      publisherAttempts: pending.sort((a, b) => a.runId - b.runId || a.attempt - b.attempt),
      environmentIds,
    },
  };
}
