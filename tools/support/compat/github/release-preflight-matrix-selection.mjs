import { githubApiPages } from "./release-preflight-github.mjs";
import { requiredRealProjectMatrixShardCount } from "./release-preflight-matrix-evidence.mjs";

export async function currentMatrixArtifacts({ run, currentJobs, api }) {
  const artifacts = await githubApiPages({
    ...api,
    resource: `actions/runs/${run.id}/artifacts`,
    collection: "artifacts",
  });
  const selection = await selectCurrentMatrixArtifacts({
    run,
    artifacts,
    currentJobs,
    readPreviousJobs: (attempt) =>
      githubApiPages({
        ...api,
        resource: `actions/runs/${run.id}/attempts/${attempt}/jobs`,
        collection: "jobs",
      }),
  });
  console.log(`Real Project Matrix artifact selection: ${JSON.stringify(selection.provenance)}`);
  return selection.artifacts;
}

export async function selectCurrentMatrixArtifacts({
  run,
  artifacts,
  currentJobs,
  readPreviousJobs,
}) {
  const attempt = positiveInteger(run, "run_attempt");
  positiveInteger(run, "id");
  if (!/^[a-f0-9]{40}$/.test(text(run, "head_sha"))) {
    throw new Error("Real Project Matrix selection requires an exact source SHA");
  }
  if (
    text(run, "path") !== ".github/workflows/real-project-matrix.yml" ||
    text(run, "status") !== "completed" ||
    text(run, "conclusion") !== "success"
  ) {
    throw new Error("Real Project Matrix selection requires a successful selected run");
  }
  const history = [];
  for (let previous = 1; previous < attempt; previous++) {
    history.push([previous, await readPreviousJobs(previous)]);
  }
  history.push([attempt, currentJobs]);
  const selected = { artifacts: [], provenance: [] };
  for (let shard = 0; shard < requiredRealProjectMatrixShardCount; shard++) {
    const name = `real-project-matrix-${shard}`;
    const jobName = `real projects (${shard}/${requiredRealProjectMatrixShardCount})`;
    const current = exactlyOneJob(currentJobs, jobName);
    assertJob(run, current, attempt);
    if (text(current, "conclusion") !== "success") {
      throw new Error(`${name} current producing job is not successful`);
    }
    const executions = [];
    for (const [snapshot, jobs] of history) {
      for (const job of jobs.filter((job) => job.name === jobName)) {
        if (typeof job.started_at !== "string") continue;
        assertJob(run, job, snapshot);
        const old = executions.find((old) => sameExecution(old, job));
        if (old != null) {
          if (positiveInteger(old, "run_attempt") === snapshot) {
            throw new Error(`${name} has ambiguous historical job identities`);
          }
        } else executions.push(job);
      }
    }
    const producer = executions.find((job) => sameExecution(job, current));
    if (producer == null) throw new Error(`${name} has no authenticated producing execution`);
    const carried = positiveInteger(producer, "run_attempt") !== attempt;
    if (carried !== timestamp(current, "started_at") < timestamp(run, "run_started_at")) {
      throw new Error(`${name} carried-forward execution does not match selected attempt`);
    }
    const matches = [];
    const historicalArtifacts = [];
    for (const artifact of artifacts.filter((artifact) => artifact.name === name)) {
      assertArtifactSource(run, artifact);
      const created = timestamp(artifact, "created_at");
      const owners = executions.filter(
        (job) =>
          timestamp(job, "started_at") <= created && created <= timestamp(job, "completed_at"),
      );
      if (owners.length !== 1) {
        throw new Error(`${name} artifact has unknown or ambiguous producing execution`);
      }
      if (sameExecution(owners[0], producer)) {
        if (artifact.expired !== false) {
          throw new Error(`${name} selected artifact is expired or has no expiry state`);
        }
        matches.push(artifact);
      }
      historicalArtifacts.push(artifact);
    }
    if (matches.length !== 1) {
      throw new Error(
        `${name} must have exactly one artifact from the successful current execution; found ${matches.length}`,
      );
    }
    selected.artifacts.push(matches[0]);
    selected.provenance.push({
      artifact: matches[0],
      producingJob: producer,
      observedJob: current,
      selectedAttempt: attempt,
      carriedForward: carried,
      historicalArtifacts,
    });
  }
  return selected;
}

function exactlyOneJob(jobs, name) {
  const matches = jobs.filter((job) => job.name === name);
  if (matches.length !== 1) {
    throw new Error(`${name} must have exactly one current job; found ${matches.length}`);
  }
  return matches[0];
}

function assertJob(run, job, attempt) {
  positiveInteger(job, "id");
  if (
    positiveInteger(job, "run_id") !== positiveInteger(run, "id") ||
    positiveInteger(job, "run_attempt") !== attempt ||
    text(job, "head_sha") !== text(run, "head_sha") ||
    text(job, "status") !== "completed"
  ) {
    throw new Error("Real Project Matrix job is not bound to the selected run/attempt/source");
  }
  const start = timestamp(job, "started_at");
  const end = timestamp(job, "completed_at");
  if (start > end || start < timestamp(run, "created_at") || end > timestamp(run, "updated_at")) {
    throw new Error("Real Project Matrix job has inconsistent execution timestamps");
  }
  text(job, "conclusion");
}

function sameExecution(left, right) {
  return ["name", "run_id", "head_sha", "status", "conclusion", "started_at", "completed_at"].every(
    (field) => left[field] === right[field],
  );
}

function assertArtifactSource(run, artifact) {
  positiveInteger(artifact, "id");
  const source = artifact.workflow_run;
  if (
    source == null ||
    positiveInteger(source, "id") !== positiveInteger(run, "id") ||
    text(source, "head_sha") !== text(run, "head_sha") ||
    text(source, "head_branch") !== text(run, "head_branch")
  ) {
    throw new Error("Real Project Matrix artifact is not bound to the selected run/source");
  }
}

function positiveInteger(value, field) {
  const number = value?.[field];
  if (!Number.isSafeInteger(number) || number <= 0) {
    throw new Error(`Matrix evidence requires positive ${field}`);
  }
  return number;
}

function text(value, field) {
  const content = value?.[field];
  if (typeof content !== "string" || content.length === 0) {
    throw new Error(`Matrix evidence requires ${field}`);
  }
  return content;
}

function timestamp(value, field) {
  const valueText = text(value, field);
  const parsed = Date.parse(valueText);
  if (
    !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/.test(valueText) ||
    !Number.isFinite(parsed) ||
    new Date(parsed).toISOString() !== valueText.replace("Z", ".000Z")
  ) {
    throw new Error(`Matrix evidence has invalid ${field} timestamp`);
  }
  return parsed;
}
