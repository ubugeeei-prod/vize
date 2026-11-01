import assert from "node:assert/strict";
import {
  appendFileSync,
  cpSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { githubApiPages, githubApiRequest } from "./release-preflight-github.mjs";
import {
  artifactRoot,
  corpusPlan,
  observers,
  sameCorpus,
  sha256,
} from "./canonical-corpus-identity.mjs";
import { validateObserverArtifact } from "./canonical-corpus-observer.mjs";
import { validateArtifactsConcurrently } from "./canonical-corpus-artifact-batch.ts";
import { readCanonicalJobs } from "./canonical-corpus-jobs.ts";

const commands = {
  dom: "Run L2 DOM differential corpus",
  "ssr-pug": "Run L4 SSR and pug L1 differential corpora",
  reach: "Measure Davinci production reach on the hydrated corpus",
};
const providerKeys = ["repositoryId", "providerSha", "providerRepositoryId"];
const positive = (value, label) => assert(Number.isSafeInteger(value) && value > 0, label);

export function canonicalTimestamp(value) {
  const match =
    typeof value === "string"
      ? /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})(?:\.(\d{1,3}))?Z$/.exec(value)
      : null;
  assert(match, "Invalid canonical execution timestamp");
  const parsed = Date.parse(value);
  const expected = `${match[1]}.${(match[2] ?? "").padEnd(3, "0")}Z`;
  assert(
    Number.isFinite(parsed) && new Date(parsed).toISOString() === expected,
    "Invalid canonical execution timestamp",
  );
  return parsed;
}
const window = (value) => {
  const start = canonicalTimestamp(value.started_at);
  const end = canonicalTimestamp(value.completed_at);
  assert(end >= start, "Invalid canonical execution window");
  return { start, end };
};
const contains = (outer, inner) => inner.start >= outer.start && inner.end <= outer.end;
const rawWindow = (value) => ({ startedAt: value.started_at, completedAt: value.completed_at });
const stepRecord = (step) => ({ name: step.name, number: step.number, ...rawWindow(step) });

function lifecycleStep(job, name) {
  const matches = job.steps.filter((step) => step.name === name);
  assert.equal(matches.length, 1, "Missing or duplicate canonical lifecycle: " + name);
  const step = matches[0];
  assert.equal(step.status, "completed", "Canonical lifecycle is incomplete");
  assert.equal(step.conclusion, "success", "Canonical lifecycle did not succeed");
  window(step);
  return step;
}

function linkedCheck(job, context, run) {
  const check = job.canonicalCheck;
  assert(check && typeof check === "object", "Missing canonical clock Check authority");
  positive(check.id, "Invalid canonical clock Check ID");
  positive(run.check_suite_id, "Missing canonical clock check suite");
  assert.equal(check.check_suite?.id, run.check_suite_id, "Foreign canonical clock check suite");
  assert.equal(check.head_sha, context.providerSha, "Foreign canonical clock Check head");
  assert.equal(check.name, job.name, "Foreign canonical clock Check name");
  const checkUrl = "https://api.github.com/repos/" + context.repository + "/check-runs/" + check.id;
  assert.equal(check.url, checkUrl, "Foreign canonical clock Check URL");
  assert.equal(job.check_run_url, checkUrl, "Foreign canonical clock job Check");
  assert.equal(
    check.details_url,
    "https://github.com/" +
      context.repository +
      "/actions/runs/" +
      context.runId +
      "/job/" +
      job.id,
    "Foreign canonical clock Check job",
  );
  assert.equal(check.status, "completed", "Canonical clock Check is incomplete");
  assert.equal(check.conclusion, "success", "Canonical clock Check did not succeed");
  return check;
}

export function completedCanonicalWorker(job, command, context, run) {
  assert.equal(job.status, "completed", "Latest canonical observer is incomplete");
  assert.equal(job.conclusion, "success", "Latest canonical observer did not succeed");
  const apiWindow = {
    start: canonicalTimestamp(job.started_at),
    end: canonicalTimestamp(job.completed_at),
  };
  const reversed = apiWindow.end < apiWindow.start;
  assert(Array.isArray(job.steps), "Invalid canonical execution window");
  const numbers = job.steps.map((step) => step.number);
  numbers.forEach((number) => positive(number, "Invalid canonical step number"));
  assert.equal(new Set(numbers).size, numbers.length, "Duplicate canonical step");
  let previous = 0;
  const required = [
    "Plan canonical corpus identity",
    "Select and hydrate full fixture corpus",
    "Capture canonical corpus identity",
    command,
    "Record required observer evidence",
    "Upload required observer evidence",
  ].map((name) => {
    const matches = job.steps.filter((step) => step.name === name);
    assert.equal(matches.length, 1, "Missing canonical step: " + name);
    const step = matches[0];
    assert.equal(step.status, "completed", "Canonical step is incomplete");
    assert.equal(step.conclusion, "success", "Canonical step did not succeed");
    assert(step.number > previous, "Canonical steps are out of order");
    window(step);
    previous = step.number;
    return step;
  });
  const upload = required.at(-1);
  const base = { apiJob: rawWindow(job), requiredSteps: required.map(stepRecord) };
  if (!reversed && required.every((step) => contains(apiWindow, window(step))))
    return { upload, timing: { authority: "actions-job-envelope", ...base, contradiction: null } };

  // The Actions/Checks/artifact API clock can contradict the runner's step
  // clock. Retain both domains; never infer or apply a timestamp offset.
  const setup = lifecycleStep(job, "Set up job");
  const complete = lifecycleStep(job, "Complete job");
  assert.equal(setup.number, Math.min(...numbers), "Foreign canonical setup lifecycle");
  assert.equal(complete.number, Math.max(...numbers), "Foreign canonical complete lifecycle");
  const runner = { start: window(setup).end, end: window(complete).start };
  assert(runner.end >= runner.start, "Reversed canonical runner lifecycle");
  previous = runner.start;
  for (const step of required) {
    const current = window(step);
    assert(
      step.number > setup.number && step.number < complete.number && contains(runner, current),
      "Foreign canonical runner step window",
    );
    assert(current.start >= previous, "Overlapping or reversed canonical runner steps");
    previous = current.end;
  }
  const check = linkedCheck(job, context, run);
  if (reversed) {
    // Starts can be unusable while the linked API completion and the artifact's
    // original runner upload window still provide independent ordering anchors.
    assert.equal(check.started_at, job.started_at, "Unmatched canonical reversed start");
    assert.equal(check.completed_at, job.completed_at, "Unmatched canonical completion anchor");
    assert(window(complete).end <= apiWindow.end, "Foreign canonical completion anchor");
  } else {
    window(check);
    assert(
      runner.end < apiWindow.start || runner.start > apiWindow.end,
      "Canonical clocks are not independently disjoint",
    );
  }
  return {
    upload,
    timing: {
      authority: reversed
        ? "linked-completion-and-runner-upload"
        : "linked-check-api-and-runner-lifecycle",
      ...base,
      apiCheck: {
        id: check.id,
        url: check.url,
        detailsUrl: check.details_url,
        ...rawWindow(check),
      },
      runnerLifecycle: { setup: stepRecord(setup), complete: stepRecord(complete) },
      contradiction: reversed
        ? "API starts follow their linked completions; starts are unusable, no offset applied"
        : "API execution envelope and runner lifecycle are disjoint; no offset applied",
    },
  };
}

export function validateCanonicalUpload(artifact, worker) {
  const created = canonicalTimestamp(artifact.created_at);
  if (
    worker.timing.authority === "actions-job-envelope" ||
    worker.timing.authority === "linked-completion-and-runner-upload"
  ) {
    assert(
      created >= canonicalTimestamp(worker.upload.started_at) &&
        created <= canonicalTimestamp(worker.upload.completed_at),
      "Foreign canonical upload window",
    );
    if (worker.timing.authority === "linked-completion-and-runner-upload")
      for (const anchor of [worker.timing.apiJob, worker.timing.apiCheck])
        assert(
          created <= canonicalTimestamp(anchor.completedAt),
          "Foreign canonical completion upload",
        );
    return;
  }
  for (const anchor of [worker.timing.apiJob, worker.timing.apiCheck]) {
    assert(
      created >= canonicalTimestamp(anchor.startedAt) &&
        created <= canonicalTimestamp(anchor.completedAt),
      "Foreign canonical API artifact window",
    );
  }
}

function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, canonical(value[key])]),
    );
  return value;
}
function execution(job) {
  return JSON.stringify(
    canonical(
      Object.fromEntries(
        [
          "status",
          "conclusion",
          "started_at",
          "completed_at",
          "steps",
          "runner_id",
          "runner_name",
          "runner_group_id",
          "runner_group_name",
          "labels",
        ].map((key) => [key, job[key]]),
      ),
    ),
  );
}

export function selectCanonicalWorkers(context, run, jobs, artifacts) {
  const { runId, attempt, sha, providerSha, repository } = context;
  positive(runId, "Invalid canonical run ID");
  positive(attempt, "Invalid canonical attempt");
  assert(/^[0-9a-f]{40}$/.test(sha), "Invalid candidate SHA");
  assert(
    /^[0-9a-f]{40}$/.test(providerSha) && !/^0+$/.test(providerSha),
    "Invalid provider head SHA",
  );
  if (context.event !== "pull_request")
    assert.equal(providerSha, sha, "Foreign canonical provider head");
  assert.equal(repository, "ubugeeei-prod/vize", "Foreign canonical repository");
  assert.equal(run.id, runId, "Foreign canonical workflow run");
  assert.equal(run.run_attempt, attempt, "Canonical workflow attempt changed");
  assert.equal(run.head_sha, providerSha, "Foreign canonical provider head");
  assert.equal(run.event, context.event, "Canonical event changed");
  assert(
    ["pull_request", "merge_group", "schedule", "workflow_dispatch", "push"].includes(run.event),
    "Unexpected canonical event",
  );
  assert(
    [".github/workflows/check.yml", ".github/workflows/real-project-matrix.yml"].includes(run.path),
    "Unexpected canonical caller",
  );
  assert.equal(run.repository?.full_name, repository, "Foreign canonical repository");
  positive(run.repository?.id, "Missing canonical repository ID");
  positive(context.repositoryId, "Missing canonical base repository identity");
  positive(context.providerRepositoryId, "Missing canonical head repository identity");
  assert.equal(run.repository.id, context.repositoryId, "Foreign canonical base repository");
  assert.equal(
    run.head_repository?.id,
    context.providerRepositoryId,
    "Foreign canonical head repository",
  );
  if (context.event !== "pull_request")
    assert.equal(
      context.providerRepositoryId,
      context.repositoryId,
      "Foreign canonical head repository",
    );
  assert(Array.isArray(jobs) && Array.isArray(artifacts), "Missing canonical provider inventory");
  const groups = Object.fromEntries(observers.map((observer) => [observer, []]));
  const ids = new Set();
  for (const job of jobs) {
    const match = /(?:^| \/ )canonical observer \(([^)]+)\)$/.exec(job.name ?? "");
    if (!match) continue;
    assert(observers.includes(match[1]), "Unknown canonical worker");
    positive(job.id, "Invalid canonical job ID");
    assert(!ids.has(job.id), "Duplicate canonical job ID");
    ids.add(job.id);
    assert.equal(job.run_id, runId, "Foreign canonical worker run");
    assert.equal(job.head_sha, providerSha, "Foreign canonical worker provider head");
    positive(job.run_attempt, "Invalid canonical worker attempt");
    assert(job.run_attempt <= attempt, "Future canonical worker");
    const group = groups[match[1]];
    assert(
      !group.some((other) => other.run_attempt === job.run_attempt),
      "Duplicate canonical attempt",
    );
    group.push(job);
  }
  const workers = observers.map((observer) => {
    const group = groups[observer].sort((left, right) => right.run_attempt - left.run_attempt);
    assert(group.length, `Missing canonical worker ${observer}`);
    const latest = group[0];
    const latestExecution = completedCanonicalWorker(latest, commands[observer], context, run); // Never borrow green behind a newer failure or pending job.
    const signature = execution(latest);
    const original = group.filter((job) => execution(job) === signature).at(-1);
    const originalExecution = completedCanonicalWorker(original, commands[observer], context, run);
    const artifactName = `canonical-corpus-${observer}-${runId}-${original.run_attempt}-${sha}`;
    const matches = artifacts.filter((artifact) => artifact.name === artifactName);
    assert.equal(matches.length, 1, "Missing or duplicate canonical artifact");
    const artifact = matches[0];
    positive(artifact.id, "Missing canonical artifact ID");
    positive(artifact.size_in_bytes, "Empty canonical artifact");
    assert.equal(artifact.expired, false, "Expired canonical artifact");
    assert(/^sha256:[0-9a-f]{64}$/.test(artifact.digest), "Missing canonical artifact digest");
    assert.equal(artifact.workflow_run?.id, runId, "Foreign canonical artifact run");
    assert.equal(
      artifact.workflow_run?.head_sha,
      providerSha,
      "Foreign canonical artifact provider head",
    );
    for (const [field, expected] of [
      ["repository_id", context.repositoryId],
      ["head_repository_id", context.providerRepositoryId],
    ])
      assert.equal(
        artifact.workflow_run?.[field],
        expected,
        "Foreign canonical artifact repository",
      );
    validateCanonicalUpload(artifact, latestExecution);
    validateCanonicalUpload(artifact, originalExecution);
    return {
      observer,
      latestJobId: latest.id,
      latestReportedAttempt: latest.run_attempt,
      executionJobId: original.id,
      executionAttempt: original.run_attempt,
      executionSha256: sha256(signature),
      artifactId: artifact.id,
      artifactName,
      artifactDigest: artifact.digest,
      artifactCreatedAt: artifact.created_at,
      timing: { latest: latestExecution.timing, execution: originalExecution.timing },
    };
  });
  assert.equal(
    new Set(workers.map((worker) => worker.artifactId)).size,
    observers.length,
    "Duplicate canonical artifacts",
  );
  return { schema: "vize.canonical-worker-selection", version: 1, ...context, workers };
}

export const validateArtifactsInProcess = (workers, root) =>
  workers.map((worker) =>
    validateObserverArtifact(worker.observer, join(root, worker.artifactName)),
  );

export function verifyCanonicalArtifacts(
  selection,
  root,
  plan,
  validateArtifacts = validateArtifactsInProcess,
) {
  for (const key of ["repository", "runId", "attempt", "event", "sha", ...providerKeys])
    assert.deepEqual(selection[key], plan[key], `Foreign canonical selection ${key}`);
  const entries = readdirSync(root, { withFileTypes: true });
  assert(
    entries.every((entry) => entry.isDirectory()),
    "Foreign canonical artifact entry",
  );
  assert.deepEqual(
    entries.map((entry) => entry.name).sort(),
    selection.workers.map((worker) => worker.artifactName).sort(),
    "Incomplete canonical artifacts",
  );
  const validated = validateArtifacts(selection.workers, root);
  assert.equal(validated.length, selection.workers.length, "Incomplete canonical validation");
  const receipts = selection.workers.map((worker, index) => {
    const receipt = validated[index];
    assert.equal(receipt.observer, worker.observer, "Foreign canonical observer receipt");
    for (const key of [
      "schema",
      "version",
      "repository",
      "runId",
      "event",
      "sha",
      ...providerKeys,
      "tree",
      "gitlinksSha256",
      "modulesSha256",
    ])
      assert.deepEqual(receipt.identity[key], plan[key], `Foreign canonical ${key}`);
    assert.equal(
      receipt.identity.attempt,
      worker.executionAttempt,
      "Foreign canonical execution attempt",
    );
    assert.deepEqual(receipt.identity.gitlinks, plan.gitlinks, "Foreign canonical gitlink owners");
    return { ...receipt.identity, attempt: selection.attempt };
  });
  for (const receipt of receipts.slice(1)) sameCorpus(receipts[0], receipt);
  return receipts[0];
}

export function requireCanonicalObservers(needs) {
  assert(needs && Object.hasOwn(needs, "canonical-observers"), "Missing canonical observer tier");
  assert.equal(
    needs["canonical-observers"].result,
    "success",
    "Required canonical observer tier did not succeed",
  );
}

async function readSelection(context, env) {
  assert(env.GH_TOKEN, "Canonical finalizer requires its read-only Actions token");
  const options = {
    apiUrl: env.GITHUB_API_URL,
    serverUrl: env.GITHUB_SERVER_URL,
    repository: context.repository,
    token: env.GH_TOKEN,
  };
  const resource = `actions/runs/${context.runId}`;
  const readRun = async () => JSON.parse((await githubApiRequest({ ...options, resource })).body);
  const before = await readRun();
  const [jobs, artifacts] = await Promise.all([
    readCanonicalJobs(options, before),
    githubApiPages({ ...options, resource: `${resource}/artifacts`, collection: "artifacts" }),
  ]);
  const selected = selectCanonicalWorkers(context, before, jobs, artifacts);
  const after = await readRun();
  assert.equal(after.check_suite_id, before.check_suite_id, "Canonical check suite changed");
  assert.deepEqual(
    selectCanonicalWorkers(context, after, jobs, artifacts),
    selected,
    "Canonical workflow identity changed during selection",
  );
  return selected;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [mode, selectionPath, downloadedRoot] = process.argv.slice(2);
  assert(
    ["select", "verify"].includes(mode) && selectionPath,
    "Expected canonical select|verify RECEIPT [ROOT]",
  );
  if (mode === "select") requireCanonicalObservers(JSON.parse(process.env.NEEDS_JSON ?? "null"));
  const plan = corpusPlan(process.cwd());
  const context = Object.fromEntries(
    ["repository", "runId", "attempt", "event", "sha", ...providerKeys].map((key) => [
      key,
      plan[key],
    ]),
  );
  const selection = await readSelection(context, process.env);
  mkdirSync(artifactRoot, { recursive: true });
  if (mode === "select") {
    writeFileSync(selectionPath, `${JSON.stringify(selection)}\n`);
    assert(process.env.GITHUB_OUTPUT, "Missing canonical finalizer output");
    appendFileSync(
      process.env.GITHUB_OUTPUT,
      `artifact-ids=${selection.workers.map((worker) => worker.artifactId).join(",")}\n`,
    );
  } else {
    assert(downloadedRoot, "Missing canonical artifact root");
    assert.deepEqual(
      selection,
      JSON.parse(readFileSync(selectionPath, "utf8")),
      "Canonical workers changed after download",
    );
    const validate = validateArtifactsConcurrently;
    const identity = verifyCanonicalArtifacts(selection, downloadedRoot, plan, validate);
    const dom = selection.workers.find((worker) => worker.observer === "dom");
    cpSync(join(downloadedRoot, dom.artifactName), artifactRoot, { recursive: true });
    writeFileSync(join(artifactRoot, "workers.json"), `${JSON.stringify(selection)}\n`);
    writeFileSync(join(artifactRoot, "corpus-identity.json"), `${JSON.stringify(identity)}\n`);
  }
}
