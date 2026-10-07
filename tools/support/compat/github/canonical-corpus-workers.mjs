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
  expectedFiles,
  observers,
  sameCorpus,
  sha256,
} from "./canonical-corpus-identity.mjs";
import { validateObserverArtifact } from "./canonical-corpus-observer.mjs";

const commands = {
  dom: "Run L2 DOM differential corpus",
  "ssr-pug": "Run L4 SSR and pug L1 differential corpora",
  reach: "Measure Davinci production reach on the hydrated corpus",
};
const providerKeys = ["repositoryId", "providerSha", "providerRepositoryId"];
const positive = (value, label) => assert(Number.isSafeInteger(value) && value > 0, label);
const timestamp = (value) => {
  assert(
    typeof value === "string" && value.endsWith("Z") && Number.isFinite(Date.parse(value)),
    "Invalid canonical execution timestamp",
  );
  return Date.parse(value);
};
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
function completedWorker(job, observer) {
  assert.equal(job.status, "completed", "Latest canonical observer is incomplete");
  assert.equal(job.conclusion, "success", "Latest canonical observer did not succeed");
  const start = timestamp(job.started_at);
  const end = timestamp(job.completed_at);
  assert(end >= start && Array.isArray(job.steps), "Invalid canonical execution window");
  const numbers = job.steps.map((step) => step.number);
  numbers.forEach((number) => positive(number, "Invalid canonical step number"));
  assert.equal(new Set(numbers).size, numbers.length, "Duplicate canonical step");
  let previous = 0;
  for (const name of [
    "Plan canonical corpus identity",
    "Select and hydrate full fixture corpus",
    "Capture canonical corpus identity",
    commands[observer],
    "Record required observer evidence",
    "Upload required observer evidence",
  ]) {
    const matches = job.steps.filter((step) => step.name === name);
    assert.equal(matches.length, 1, `Missing canonical step: ${name}`);
    const step = matches[0];
    assert.equal(step.status, "completed", "Canonical step is incomplete");
    assert.equal(step.conclusion, "success", "Canonical step did not succeed");
    assert(step.number > previous, "Canonical steps are out of order");
    assert(
      timestamp(step.started_at) >= start &&
        timestamp(step.completed_at) >= timestamp(step.started_at) &&
        timestamp(step.completed_at) <= end,
      "Foreign canonical step window",
    );
    previous = step.number;
  }
  return job.steps.find((step) => step.name === "Upload required observer evidence");
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
    completedWorker(latest, observer); // Never borrow green behind a newer failure or pending job.
    const signature = execution(latest);
    const original = group.filter((job) => execution(job) === signature).at(-1);
    const upload = completedWorker(original, observer);
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
    assert(
      timestamp(artifact.created_at) >= timestamp(upload.started_at) &&
        timestamp(artifact.created_at) <= timestamp(upload.completed_at),
      "Foreign canonical upload window",
    );
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
    };
  });
  assert.equal(
    new Set(workers.map((worker) => worker.artifactId)).size,
    observers.length,
    "Duplicate canonical artifacts",
  );
  return { schema: "vize.canonical-worker-selection", version: 1, ...context, workers };
}

export function verifyCanonicalArtifacts(selection, root, plan) {
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
  const receipts = selection.workers.map((worker) => {
    const receipt = validateObserverArtifact(worker.observer, join(root, worker.artifactName));
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
    assert.equal(receipt.identity.files, expectedFiles, "Canonical corpus shrank");
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
    repository: context.repository,
    token: env.GH_TOKEN,
  };
  const resource = `actions/runs/${context.runId}`;
  const readRun = async () => JSON.parse((await githubApiRequest({ ...options, resource })).body);
  const before = await readRun();
  const [jobs, artifacts] = await Promise.all([
    githubApiPages({
      ...options,
      resource: `${resource}/jobs`,
      collection: "jobs",
      query: { filter: "all" },
    }),
    githubApiPages({ ...options, resource: `${resource}/artifacts`, collection: "artifacts" }),
  ]);
  const selected = selectCanonicalWorkers(context, before, jobs, artifacts);
  assert.deepEqual(
    selectCanonicalWorkers(context, await readRun(), jobs, artifacts),
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
    const identity = verifyCanonicalArtifacts(selection, downloadedRoot, plan);
    const dom = selection.workers.find((worker) => worker.observer === "dom");
    cpSync(join(downloadedRoot, dom.artifactName), artifactRoot, { recursive: true });
    writeFileSync(join(artifactRoot, "workers.json"), `${JSON.stringify(selection)}\n`);
    writeFileSync(join(artifactRoot, "corpus-identity.json"), `${JSON.stringify(identity)}\n`);
  }
}
