import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import {
  completedCanonicalWorker,
  validateCanonicalUpload,
} from "../../tools/support/compat/github/canonical-corpus-workers.mjs";
import { selectCanonicalWorkers } from "../../tools/support/compat/github/canonical-corpus-workers.mjs";
import {
  artifactBase64,
  checkBase64,
  jobBase64,
  provenance,
  reversedArtifactBase64,
  reversedCheckBase64,
  reversedJobBase64,
  reversedProvenance,
} from "./fixtures/canonical-worker-clocks/c1fa-dom.ts";

const decode = (value: string): unknown =>
  JSON.parse(Buffer.from(value, "base64").toString("utf8"));
const digest = (value: string) =>
  createHash("sha256").update(Buffer.from(value, "base64")).digest("hex");
type Step = {
  name: string;
  number: number;
  status: string;
  conclusion: string;
  started_at: string | null;
  completed_at: string | null;
};
type Check = {
  id: number;
  name: string;
  head_sha: string;
  url: string;
  details_url: string;
  status: string;
  conclusion: string;
  started_at: string;
  completed_at: string;
  check_suite: { id: number };
};
type Job = {
  id: number;
  run_id: number;
  run_attempt: number;
  head_sha: string;
  name: string;
  check_run_url: string;
  started_at: string;
  completed_at: string;
  steps: Step[];
  canonicalCheck?: Check;
};
type Artifact = {
  id: number;
  name: string;
  created_at: string;
  digest: string;
  workflow_run: { id: number; repository_id: number; head_repository_id: number; head_sha: string };
};
const retainedCheck = decode(checkBase64) as Check;
const job: Job = { ...(decode(jobBase64) as Job), canonicalCheck: retainedCheck };
const artifact = decode(artifactBase64) as Artifact;
const stepOf = (value: Job, name: string) => {
  const step = value.steps.find((entry) => entry.name === name);
  assert.ok(step, "Missing fixture step: " + name);
  return step;
};
const checkOf = (value: Job) => {
  assert.ok(value.canonicalCheck);
  return value.canonicalCheck;
};
const firstJob = (value: Job[]) => {
  const item = value.at(0);
  assert.ok(item);
  return item;
};
const firstArtifact = (value: Artifact[]) => {
  const item = value.at(0);
  assert.ok(item);
  return item;
};
const command = "Run L2 DOM differential corpus";
const context = {
  repository: "ubugeeei-prod/vize",
  repositoryId: artifact.workflow_run.repository_id,
  providerRepositoryId: artifact.workflow_run.head_repository_id,
  runId: job.run_id,
  attempt: job.run_attempt,
  sha: job.head_sha,
  providerSha: job.head_sha,
  event: "merge_group",
};
const run = {
  id: job.run_id,
  run_attempt: job.run_attempt,
  head_sha: job.head_sha,
  check_suite_id: retainedCheck.check_suite.id,
  event: context.event,
  path: ".github/workflows/check.yml",
  repository: { id: context.repositoryId, full_name: context.repository },
  head_repository: { id: context.providerRepositoryId },
};
const inspect = (value: Job = job, item: Artifact = artifact) => {
  const result = completedCanonicalWorker(value, command, context, run);
  validateCanonicalUpload(item, result);
  return result;
};

await test("retained exact c1fa job, Check and artifact expose both clock domains without correction", () => {
  assert.equal(digest(jobBase64), provenance.jobSha256);
  assert.equal(digest(checkBase64), provenance.checkSha256);
  assert.equal(digest(artifactBase64), provenance.selectedArtifactSha256);
  const result = inspect();
  assert.ok("runnerLifecycle" in result.timing);
  assert.equal(result.timing.authority, "linked-check-api-and-runner-lifecycle");
  assert.deepEqual(result.timing.apiJob, {
    startedAt: "2026-11-01T09:05:06Z",
    completedAt: "2026-11-01T09:09:30Z",
  });
  assert.equal(result.timing.runnerLifecycle.setup.startedAt, "2026-11-01T08:05:07Z");
  assert.equal(result.timing.runnerLifecycle.complete.completedAt, "2026-11-01T08:09:28Z");
  assert.equal(result.timing.requiredSteps.length, 6);
  assert.equal(result.upload.completed_at, "2026-11-01T08:09:26Z");
  assert.equal(artifact.created_at, "2026-11-01T09:09:25Z");
  assert.equal(stepOf(job, "Set up job").started_at, "2026-11-01T08:05:07Z");
});

await test("missing, duplicate, incomplete or reversed lifecycle anchors refuse clock fallback", () => {
  for (const change of [
    (value) => value.steps.shift(),
    (value) => value.steps.push({ ...stepOf(value, "Set up job"), number: 100 }),
    (value) => {
      stepOf(value, "Set up job").conclusion = "skipped";
    },
    (value) => {
      stepOf(value, "Complete job").name = "foreign completion";
    },
    (value) => {
      stepOf(value, "Complete job").started_at = "2026-11-01T08:05:01Z";
    },
    (value) => {
      stepOf(value, "Set up job").number = 99;
    },
  ] satisfies ((value: Job) => void)[]) {
    const forged = structuredClone(job);
    change(forged);
    assert.throws(() => inspect(forged));
  }
});

await test("one foreign, failed, skipped, reversed or overlapping required step cannot borrow anchors", () => {
  for (const change of [
    (step) => {
      step.started_at = "2026-11-01T07:00:00Z";
    },
    (step) => {
      step.completed_at = "2026-11-01T08:10:00Z";
    },
    (step) => {
      step.completed_at = "2026-11-01T08:08:17Z";
    },
    (step) => {
      step.started_at = "2026-11-01T08:08:17Z";
    },
    (step) => {
      step.conclusion = "failure";
    },
    (step) => {
      step.status = "queued";
    },
    (step) => {
      step.conclusion = "skipped";
    },
    (step) => {
      step.number = 14;
    },
  ] satisfies ((value: Step) => void)[]) {
    const forged = structuredClone(job);
    change(stepOf(forged, command));
    assert.throws(() => inspect(forged));
  }
});

await test("Check job, suite, head, completion and artifact API-window authorities are mandatory", () => {
  for (const change of [
    (value) => {
      delete value.canonicalCheck;
    },
    (value) => {
      checkOf(value).head_sha = "b".repeat(40);
    },
    (value) => {
      checkOf(value).check_suite.id += 1;
    },
    (value) => {
      checkOf(value).id += 1;
    },
    (value) => {
      checkOf(value).details_url += "1";
    },
    (value) => {
      checkOf(value).name = "foreign observer";
    },
    (value) => {
      checkOf(value).status = "in_progress";
    },
    (value) => {
      checkOf(value).conclusion = "failure";
    },
    (value) => {
      value.check_run_url += "1";
    },
    (value) => {
      checkOf(value).completed_at = "2026-11-01T09:09:24Z";
    },
    (value) => {
      value.completed_at = "2026-11-01T09:09:24Z";
    },
  ] satisfies ((value: Job) => void)[]) {
    const forged = structuredClone(job);
    change(forged);
    assert.throws(() => inspect(forged));
  }
  for (const created_at of ["2026-11-01T09:05:05Z", "2026-11-01T09:09:31Z"])
    assert.throws(() => inspect(job, { ...artifact, created_at }));
});

await test("coherent runner timestamps are never corrected by a fixed offset", () => {
  const shifted = structuredClone(job);
  for (const step of shifted.steps) {
    for (const key of ["started_at", "completed_at"] as const)
      if (step[key]) step[key] = new Date(Date.parse(step[key] ?? "") - 17 * 60_000).toISOString();
  }
  const result = inspect(shifted);
  assert.ok("runnerLifecycle" in result.timing);
  assert.equal(result.timing.runnerLifecycle.setup.startedAt, "2026-11-01T07:48:07.000Z");
  assert.equal(result.timing.apiJob.startedAt, job.started_at);
});

await test("agreeing clocks retain original strict upload window instead of weaker API bounds", () => {
  const agreeing = structuredClone(job);
  agreeing.started_at = "2026-11-01T08:05:06Z";
  agreeing.completed_at = "2026-11-01T08:09:30Z";
  const result = inspect(agreeing, { ...artifact, created_at: "2026-11-01T08:09:25Z" });
  assert.equal(result.timing.authority, "actions-job-envelope");
  assert.throws(() => inspect(agreeing, { ...artifact, created_at: "2026-11-01T08:09:29Z" }));
  const oneForeign = structuredClone(agreeing);
  stepOf(oneForeign, command).started_at = "2026-11-01T08:05:05Z";
  assert.throws(() => inspect(oneForeign, { ...artifact, created_at: "2026-11-01T08:09:25Z" }));
});

await test("selection still rejects foreign run, attempt, head, digest and artifact identity", () => {
  const jobs = [job];
  const artifacts = [artifact];
  for (const [observer, name] of [
    ["ssr-pug", "Run L4 SSR and pug L1 differential corpora"],
    ["reach", "Measure Davinci production reach on the hydrated corpus"],
  ]) {
    const worker = structuredClone(job);
    worker.id += jobs.length;
    worker.name = worker.name.replace("(dom)", "(" + observer + ")");
    worker.check_run_url = worker.check_run_url.replace(
      String(retainedCheck.id),
      String(retainedCheck.id + jobs.length),
    );
    worker.canonicalCheck = {
      ...checkOf(worker),
      id: checkOf(worker).id + jobs.length,
      name: worker.name,
      url: worker.check_run_url,
      details_url: checkOf(worker).details_url.replace(String(job.id), String(worker.id)),
    };
    stepOf(worker, name).name = "Inactive sibling observer";
    stepOf(worker, command).name = name;
    jobs.push(worker);
    artifacts.push({
      ...artifact,
      id: artifact.id + jobs.length,
      name: artifact.name.replace("-dom-", "-" + observer + "-"),
    });
  }
  const selected = selectCanonicalWorkers(context, run, jobs, artifacts);
  assert.equal(selected.workers.length, 3);
  assert.deepEqual(
    selected.workers.map(
      (worker: { timing: { execution: { authority: string } } }) =>
        worker.timing.execution.authority,
    ),
    Array(3).fill("linked-check-api-and-runner-lifecycle"),
  );
  const carried = structuredClone(firstJob(jobs));
  carried.id += 100;
  carried.run_attempt = 2;
  checkOf(carried).id += 100;
  carried.check_run_url = checkOf(carried).url = checkOf(carried).url.replace(
    String(retainedCheck.id),
    String(checkOf(carried).id),
  );
  checkOf(carried).details_url = checkOf(carried).details_url.replace(
    String(job.id),
    String(carried.id),
  );
  const later = { ...context, attempt: 2 };
  const laterRun = { ...run, run_attempt: 2 };
  assert.equal(
    selectCanonicalWorkers(later, laterRun, [...jobs, carried], artifacts).workers[0]
      .executionAttempt,
    1,
  );
  checkOf(carried).started_at = "2026-11-01T10:00:00Z";
  checkOf(carried).completed_at = "2026-11-01T10:10:00Z";
  assert.throws(() => selectCanonicalWorkers(later, laterRun, [...jobs, carried], artifacts));
  for (const change of [
    (value) => {
      firstJob(value).run_id += 1;
    },
    (value) => {
      firstJob(value).run_attempt += 1;
    },
    (value) => {
      firstJob(value).head_sha = "b".repeat(40);
    },
  ] satisfies ((value: Job[]) => void)[]) {
    const forged = structuredClone(jobs);
    change(forged);
    assert.throws(() => selectCanonicalWorkers(context, run, forged, artifacts));
  }
  for (const change of [
    (value) => {
      firstArtifact(value).workflow_run.id += 1;
    },
    (value) => {
      firstArtifact(value).workflow_run.head_sha = "b".repeat(40);
    },
    (value) => {
      firstArtifact(value).digest = "missing";
    },
    (value) => {
      firstArtifact(value).name = firstArtifact(value).name.replace("-1-", "-2-");
    },
  ] satisfies ((value: Artifact[]) => void)[]) {
    const forged = structuredClone(artifacts);
    change(forged);
    assert.throws(() => selectCanonicalWorkers(context, run, jobs, forged));
  }
});

const reverseJob: Job = {
  ...(decode(reversedJobBase64) as Job),
  canonicalCheck: decode(reversedCheckBase64) as Check,
};
const reverseArtifact = decode(reversedArtifactBase64) as Artifact;
const reverseContext = {
  ...context,
  runId: reverseJob.run_id,
  sha: reverseJob.head_sha,
  providerSha: reverseJob.head_sha,
};
const reverseRun = {
  ...run,
  id: reverseJob.run_id,
  head_sha: reverseJob.head_sha,
  check_suite_id: checkOf(reverseJob).check_suite.id,
};
const inspectReverse = (value: Job = reverseJob, item: Artifact = reverseArtifact) => {
  const result = completedCanonicalWorker(value, command, reverseContext, reverseRun);
  validateCanonicalUpload(item, result);
  return result;
};

await test("retained reversed starts use linked completion and exact runner upload without rewriting", () => {
  assert.equal(digest(reversedJobBase64), reversedProvenance.jobProjectionSha256);
  assert.equal(digest(reversedCheckBase64), reversedProvenance.checkSha256);
  assert.equal(digest(reversedArtifactBase64), reversedProvenance.selectedArtifactSha256);
  const result = inspectReverse();
  assert.equal(result.timing.authority, "linked-completion-and-runner-upload");
  assert.ok("apiCheck" in result.timing && "runnerLifecycle" in result.timing);
  assert.deepEqual(result.timing.apiJob, {
    startedAt: "2026-11-01T09:57:37Z",
    completedAt: "2026-11-01T09:01:53Z",
  });
  assert.equal(result.timing.apiCheck.startedAt, reverseJob.started_at);
  assert.equal(result.timing.apiCheck.completedAt, reverseJob.completed_at);
  assert.equal(result.timing.runnerLifecycle.setup.startedAt, "2026-11-01T08:57:38Z");
  assert.equal(result.timing.runnerLifecycle.complete.completedAt, "2026-11-01T09:01:50Z");
  assert.equal(result.upload.started_at, "2026-11-01T09:01:45Z");
  assert.equal(result.upload.completed_at, "2026-11-01T09:01:49Z");
  assert.equal(reverseArtifact.created_at, "2026-11-01T09:01:48Z");
  assert.equal(reverseJob.started_at, "2026-11-01T09:57:37Z");
});

await test("reversed starts cannot borrow missing, foreign or incoherent completion authority", () => {
  for (const change of [
    (value) => {
      delete value.canonicalCheck;
    },
    (value) => {
      checkOf(value).id += 1;
    },
    (value) => {
      checkOf(value).head_sha = "b".repeat(40);
    },
    (value) => {
      checkOf(value).check_suite.id += 1;
    },
    (value) => {
      checkOf(value).details_url += "1";
    },
    (value) => {
      checkOf(value).conclusion = "failure";
    },
    (value) => {
      checkOf(value).started_at = "2026-11-01T09:57:36Z";
    },
    (value) => {
      checkOf(value).completed_at = "2026-11-01T09:01:54Z";
    },
    (value) => {
      value.completed_at = checkOf(value).completed_at = "2026-11-01T09:01:49Z";
    },
    (value) => {
      stepOf(value, "Set up job").name = "foreign setup";
    },
    (value) => {
      value.steps.push({ ...stepOf(value, "Complete job"), number: 99 });
    },
    (value) => {
      stepOf(value, command).started_at = "2026-11-01T08:57:37Z";
    },
    (value) => {
      stepOf(value, command).completed_at = "2026-11-01T09:01:49Z";
    },
    (value) => {
      stepOf(value, command).conclusion = "skipped";
    },
    (value) => {
      stepOf(value, "Complete job").started_at = "2026-11-01T08:57:37Z";
    },
  ] satisfies ((value: Job) => void)[]) {
    const forged = structuredClone(reverseJob);
    change(forged);
    assert.throws(() => inspectReverse(forged));
  }
  for (const created_at of ["2026-11-01T09:01:44Z", "2026-11-01T09:01:50Z", "2026-11-01T09:57:37Z"])
    assert.throws(() => inspectReverse(reverseJob, { ...reverseArtifact, created_at }));
});
