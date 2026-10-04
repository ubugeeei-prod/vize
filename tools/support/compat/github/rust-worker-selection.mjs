import assert from "node:assert/strict";
import { createHash } from "node:crypto";

const workerPrefix = "PR source checks / PR Rust Clippy, tests, and fixtures / Rust tests (";
const requiredSteps = [
  "Verify Rust archive identity",
  "Run Rust test shard without rebuilding",
  "Verify actual typechecker fixture observations",
  "Upload Rust shard results",
];

function positiveInteger(value, label) {
  assert(Number.isSafeInteger(value) && value > 0, `${label} must be a positive integer`);
}

function timestamp(value, label) {
  assert(typeof value === "string" && value.endsWith("Z"), `${label} must be a UTC timestamp`);
  const result = Date.parse(value);
  assert(Number.isFinite(result), `${label} must be a UTC timestamp`);
  return result;
}

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value != null && typeof value === "object")
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, canonical(value[key])]),
    );
  return value;
}

// GitHub creates new job IDs for carried successful jobs on a failed-only rerun.
// Their reported run_attempt changes; the complete original execution does not.
function execution(job) {
  return JSON.stringify(
    canonical({
      status: job.status,
      conclusion: job.conclusion,
      started_at: job.started_at,
      completed_at: job.completed_at,
      steps: job.steps,
      runner_id: job.runner_id,
      runner_name: job.runner_name,
      runner_group_id: job.runner_group_id,
      runner_group_name: job.runner_group_name,
      labels: job.labels,
    }),
  );
}

function requireCompletedWorker(job) {
  assert.equal(job.status, "completed", `Latest worker ${job.name} is incomplete`);
  assert.equal(job.conclusion, "success", `Latest worker ${job.name} did not succeed`);
  const start = timestamp(job.started_at, "Worker start");
  const end = timestamp(job.completed_at, "Worker completion");
  assert(end >= start, "Worker execution window is reversed");
  assert(Array.isArray(job.steps), "Worker has no execution steps");
  const numbers = job.steps.map((step) => step.number);
  numbers.forEach((number) => positiveInteger(number, "Step number"));
  assert.equal(new Set(numbers).size, numbers.length, "Duplicate worker step number");
  let previous = 0;
  for (const name of requiredSteps) {
    const matches = job.steps.filter((step) => step.name === name);
    assert.equal(matches.length, 1, `Worker must execute ${name} exactly once`);
    const step = matches[0];
    assert.equal(step.status, "completed", `${name} is incomplete`);
    assert.equal(step.conclusion, "success", `${name} did not succeed`);
    assert(step.number > previous, "Required worker steps are out of order");
    const from = timestamp(step.started_at, `${name} start`);
    const to = timestamp(step.completed_at, `${name} completion`);
    assert(from >= start && to >= from && to <= end, `${name} is outside the worker execution`);
    previous = step.number;
  }
  return job.steps.find((step) => step.name === "Upload Rust shard results");
}

export function selectRustWorkers(context, run, jobs, artifacts) {
  const { runId, attempt, sha, repository } = context;
  positiveInteger(runId, "Run ID");
  positiveInteger(attempt, "Run attempt");
  assert(/^[0-9a-f]{40}$/.test(sha), "Invalid source SHA");
  assert.equal(run.id, runId, "Foreign workflow run");
  assert.equal(run.run_attempt, attempt, "Workflow attempt changed");
  assert.equal(run.head_sha, sha, "Foreign workflow source");
  assert.equal(run.event, "merge_group", "Full Rust workers require merge_group");
  assert.equal(run.path, ".github/workflows/check.yml", "Unexpected Rust caller workflow");
  assert.equal(run.repository?.full_name, repository, "Foreign workflow repository");
  positiveInteger(run.repository?.id, "Repository ID");
  assert(
    Array.isArray(jobs) && Array.isArray(artifacts),
    "Missing official job/artifact inventory",
  );
  const groups = [[], [], [], []];
  const jobIds = new Set();
  for (const job of jobs) {
    if (typeof job.name !== "string" || !job.name.startsWith(workerPrefix)) continue;
    const match = /^([1-4])\/4\)$/.exec(job.name.slice(workerPrefix.length));
    assert(match, `Unexpected Rust worker name: ${job.name}`);
    positiveInteger(job.id, "Worker job ID");
    assert(!jobIds.has(job.id), "Duplicate worker job ID");
    jobIds.add(job.id);
    assert.equal(job.run_id, runId, "Foreign worker run");
    assert.equal(job.head_sha, sha, "Foreign worker source");
    positiveInteger(job.run_attempt, "Worker attempt");
    assert(job.run_attempt <= attempt, "Future worker attempt");
    const group = groups[Number(match[1]) - 1];
    assert(
      !group.some((other) => other.run_attempt === job.run_attempt),
      "Duplicate worker attempt",
    );
    group.push(job);
  }
  const workers = groups.map((group, index) => {
    const shard = index + 1;
    assert(group.length > 0, `Missing Rust worker ${shard}`);
    group.sort((left, right) => right.run_attempt - left.run_attempt);
    const latest = group[0];
    requireCompletedWorker(latest); // Never fall back behind a failed/incomplete latest outcome.
    const signature = execution(latest);
    const original = group.filter((job) => execution(job) === signature).at(-1);
    const upload = requireCompletedWorker(original);
    const name = `rust-test-shard-${shard}-${runId}-${original.run_attempt}`;
    const matches = artifacts.filter((artifact) => artifact.name === name);
    assert.equal(matches.length, 1, `Expected exactly one artifact: ${name}`);
    const artifact = matches[0];
    positiveInteger(artifact.id, "Artifact ID");
    positiveInteger(artifact.size_in_bytes, "Artifact size");
    assert.equal(artifact.expired, false, "Rust worker artifact expired");
    assert(/^sha256:[0-9a-f]{64}$/.test(artifact.digest), "Missing Rust worker artifact digest");
    assert.equal(artifact.workflow_run?.id, runId, "Foreign artifact run");
    assert.equal(artifact.workflow_run?.head_sha, sha, "Foreign artifact source");
    assert.equal(
      artifact.workflow_run?.repository_id,
      run.repository.id,
      "Foreign artifact repository",
    );
    assert.equal(
      artifact.workflow_run?.head_repository_id,
      run.repository.id,
      "Foreign artifact head repository",
    );
    const created = timestamp(artifact.created_at, "Artifact creation");
    assert(
      created >= timestamp(upload.started_at, "Upload start") &&
        created <= timestamp(upload.completed_at, "Upload completion"),
      "Artifact was not created during the original successful upload",
    );
    return {
      shard,
      latestJobId: latest.id,
      latestReportedAttempt: latest.run_attempt,
      executionJobId: original.id,
      executionAttempt: original.run_attempt,
      startedAt: original.started_at,
      completedAt: original.completed_at,
      executionSha256: createHash("sha256").update(signature).digest("hex"),
      artifactId: artifact.id,
      artifactName: name,
      artifactDigest: artifact.digest,
      artifactCreatedAt: artifact.created_at,
    };
  });
  assert.equal(
    new Set(workers.map((worker) => worker.artifactId)).size,
    4,
    "Duplicate Rust artifact ID",
  );
  return {
    schema: "vize.rust-worker-selection",
    version: 1,
    runId,
    attempt,
    sourceSha: sha,
    repository,
    event: run.event,
    workflowPath: run.path,
    workers,
  };
}

export function verifyRustWorkerDirectories(selection, entries) {
  assert.deepEqual(
    entries
      .map((entry) => {
        assert(entry.isDirectory(), "Rust artifact inventory contains a non-directory");
        return entry.name;
      })
      .sort(),
    selection.workers.map((worker) => worker.artifactName).sort(),
    "Downloaded Rust artifact inventory differs from the four selected workers",
  );
}
