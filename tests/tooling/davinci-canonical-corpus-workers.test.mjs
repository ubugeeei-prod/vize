import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { test } from "node:test";
import {
  forcedCheckoutArgs,
  validateHydration,
} from "../../tools/support/compat/github/canonical-corpus-hydration.mjs";
import {
  expectedFiles,
  observers,
  sha256,
} from "../../tools/support/compat/github/canonical-corpus-identity.mjs";
import {
  observerLogs,
  validateObserverLog,
} from "../../tools/support/compat/github/canonical-corpus-observer.mjs";
import {
  requireCanonicalObservers,
  selectCanonicalWorkers,
  verifyCanonicalArtifacts,
} from "../../tools/support/compat/github/canonical-corpus-workers.mjs";

const sha = "a".repeat(40);
const context = {
  repositoryId: 99,
  providerRepositoryId: 99,
  providerSha: sha,
  repository: "ubugeeei-prod/vize",
  runId: 12,
  attempt: 1,
  sha,
  event: "merge_group",
};
const run = {
  head_repository: { id: 99 },
  id: 12,
  run_attempt: 1,
  head_sha: sha,
  event: "merge_group",
  repository: {
    full_name: context.repository,
    id: 99,
  },
  path: ".github/workflows/check.yml",
};
const commands = [
  "Run L2 DOM differential corpus",
  "Run L4 SSR and pug L1 differential corpora",
  "Measure Davinci production reach on the hydrated corpus",
];
const jobs = observers.map((observer, index) => ({
  id: index + 1,
  run_id: 12,
  run_attempt: 1,
  head_sha: sha,
  name: `PR source checks / canonical-corpus / canonical observer (${observer})`,
  status: "completed",
  conclusion: "success",
  started_at: "2026-10-07T11:01:00Z",
  completed_at: "2026-10-07T11:05:00Z",
  runner_id: 8,
  labels: ["linux"],
  steps: [
    "Plan canonical corpus identity",
    "Select and hydrate full fixture corpus",
    "Capture canonical corpus identity",
    commands[index],
    "Record required observer evidence",
    "Upload required observer evidence",
  ].map((name, number) => ({
    name,
    number: number + 1,
    status: "completed",
    conclusion: "success",
    started_at: "2026-10-07T11:02:00Z",
    completed_at: "2026-10-07T11:04:00Z",
  })),
}));
const artifacts = observers.map((observer, index) => ({
  id: index + 101,
  name: `canonical-corpus-${observer}-12-1-${sha}`,
  size_in_bytes: 1000,
  expired: false,
  digest: `sha256:${"b".repeat(64)}`,
  created_at: "2026-10-07T11:03:00Z",
  workflow_run: { id: 12, head_sha: sha, repository_id: 99, head_repository_id: 99 },
}));
const select = (
  currentJobs = jobs,
  currentArtifacts = artifacts,
  currentRun = run,
  current = context,
) => selectCanonicalWorkers(current, currentRun, currentJobs, currentArtifacts);

test("complete latest canonical workers retain official upload identity and carried-execution custody", () => {
  const selected = select();
  assert.deepEqual(
    selected.workers.map((worker) => worker.observer),
    observers,
  );
  assert.deepEqual(
    selected.workers.map((worker) => worker.artifactId),
    [101, 102, 103],
  );
  const carried = structuredClone(jobs).map((job) => ({ ...job, id: job.id + 20, run_attempt: 2 }));
  const retry = select(
    [...jobs, ...carried],
    artifacts,
    { ...run, run_attempt: 2 },
    { ...context, attempt: 2 },
  );
  assert.deepEqual(
    retry.workers.map((worker) => worker.latestReportedAttempt),
    [2, 2, 2],
  );
  assert.deepEqual(
    retry.workers.map((worker) => worker.executionAttempt),
    [1, 1, 1],
  );
  assert.deepEqual(
    retry.workers.map((worker) => worker.artifactName),
    artifacts.map((artifact) => artifact.name),
  );
  const providerSha = "8e982edbd8cf31ee5404157757b4f1efe47663ae";
  const prContext = { ...context, event: "pull_request", providerSha, providerRepositoryId: 199 };
  const prRun = {
    ...run,
    event: prContext.event,
    head_sha: providerSha,
    head_repository: { id: 199 },
  };
  const prJobs = jobs.map((job) => ({ ...job, head_sha: providerSha }));
  const prArtifacts = artifacts.map((artifact) => ({
    ...artifact,
    workflow_run: { ...artifact.workflow_run, head_sha: providerSha, head_repository_id: 199 },
  }));
  const pr = select(prJobs, prArtifacts, prRun, prContext);
  assert.equal(pr.sha, sha, "Candidate merge checkout remains distinct from provider PR head");
  assert.equal(pr.providerSha, providerSha);
  assert.deepEqual(
    pr.workers.map((worker) => worker.artifactName),
    artifacts.map((artifact) => artifact.name),
  );
  assert.throws(() => select(prJobs, prArtifacts, { ...prRun, head_sha: sha }, prContext));
  assert.throws(() => select(jobs, prArtifacts, prRun, prContext));
  assert.throws(() => select(prJobs, artifacts, prRun, prContext));
});

test("an older green observer cannot hide a latest failure, skip, cancellation or incomplete command", () => {
  for (const conclusion of ["failure", "skipped", "cancelled", "", null]) {
    const latest = { ...structuredClone(jobs[0]), id: 20, run_attempt: 2, conclusion };
    assert.throws(() =>
      select([...jobs, latest], artifacts, { ...run, run_attempt: 2 }, { ...context, attempt: 2 }),
    );
  }
  for (const mutation of [
    (value) => value.shift(),
    (value) => {
      value[0].status = "in_progress";
    },
    (value) => {
      value[0].head_sha = "c".repeat(40);
    },
    (value) => {
      value[0].run_id = 11;
    },
    (value) => {
      value[0].run_attempt = 2;
    },
    (value) => {
      value[0].steps.splice(3, 1);
    },
    (value) => {
      value[0].steps[3].conclusion = "skipped";
    },
    (value) => {
      value[0].steps[3].started_at = "2026-10-07T10:00:00Z";
    },
    (value) => {
      value[0].steps[3].number = 1;
    },
    (value) => value.push(structuredClone(value[0])),
    (value) => {
      value[0].name = "canonical observer (foreign)";
    },
  ]) {
    const forged = structuredClone(jobs);
    mutation(forged);
    assert.throws(() => select(forged));
  }
});

test("foreign, expired, duplicate and non-upload-bound artifacts fail closed", () => {
  for (const mutation of [
    (value) => value.pop(),
    (value) => value.push(structuredClone(value[0])),
    (value) => {
      value[0].expired = true;
    },
    (value) => {
      value[0].size_in_bytes = 0;
    },
    (value) => {
      value[0].digest = "missing";
    },
    (value) => {
      value[0].workflow_run.id = 11;
    },
    (value) => {
      value[0].workflow_run.head_sha = "c".repeat(40);
    },
    (value) => {
      value[0].workflow_run.repository_id = 7;
    },
    (value) => {
      value[0].workflow_run.head_repository_id = 7;
    },
    (value) => {
      value[0].created_at = "2026-10-07T11:05:00Z";
    },
  ]) {
    const forged = structuredClone(artifacts);
    mutation(forged);
    assert.throws(() => select(jobs, forged));
  }
  for (const key of ["id", "run_attempt", "head_sha", "event", "path"])
    assert.throws(() => select(jobs, artifacts, { ...run, [key]: "foreign" }));
  for (const result of ["failure", "cancelled", "skipped", "queued", "in_progress", null])
    assert.throws(() => requireCanonicalObservers({ "canonical-observers": { result } }));
  for (const missing of [null, {}, { unrelated: { result: "success" } }])
    assert.throws(() => requireCanonicalObservers(missing));
  requireCanonicalObservers({ "canonical-observers": { result: "success" } });
});

test("finalization rejects replaced full bytes, fixture identity and incomplete observer directories", () => {
  const root = mkdtempSync(join(tmpdir(), "canonical-artifacts-"));
  const selected = select();
  const gitlinks = Array.from({ length: 147 }, (_, index) => ({
    path: `tests/_fixtures/_git/project-${String(index).padStart(3, "0")}`,
    sha: "d".repeat(40),
  }));
  const files = Array.from({ length: expectedFiles }, (_, index) => [
    `project-${index}/Original.vue`,
    "e".repeat(64),
    31,
  ]);
  const identity = {
    schema: "vize.canonical-corpus-identity",
    version: 1,
    ...context,
    tree: "f".repeat(40),
    gitlinks,
    gitlinksSha256: sha256(JSON.stringify(gitlinks)),
    modulesSha256: "c".repeat(64),
    files: expectedFiles,
    filesSha256: sha256(JSON.stringify(files)),
  };
  const selectedText = gitlinks.map((row) => row.path).join("\n") + "\n";
  const statusText = gitlinks.map((row) => ` ${row.sha} ${row.path}`).join("\n") + "\n";
  const write = (path, value) => writeFileSync(path, JSON.stringify(value) + "\n");
  try {
    for (const worker of selected.workers) {
      const path = join(root, worker.artifactName);
      mkdirSync(path);
      const bytes = readFileSync(
        `tests/tooling/fixtures/canonical-observer-logs/${worker.observer}.log`,
      );
      write(join(path, "identity.json"), identity);
      write(join(path, "files.json"), files);
      writeFileSync(join(path, "selected-gitlinks.txt"), selectedText);
      writeFileSync(join(path, "submodule-status.txt"), statusText);
      write(join(path, "hydration.json"), {
        ...identity,
        schema: "vize.canonical-corpus-hydration",
        args: forcedCheckoutArgs(gitlinks),
        status: 0,
        statusStatus: 0,
        stdoutSha256: sha256(""),
        stderrSha256: sha256(""),
        statusStderrSha256: sha256(""),
      });
      for (const name of [
        "hydration-stdout.log",
        "hydration-stderr.log",
        "hydration-status-stderr.log",
      ])
        writeFileSync(join(path, name), "");
      writeFileSync(join(path, observerLogs[worker.observer]), bytes);
      write(join(path, "observer.json"), {
        schema: "vize.canonical-corpus-observer",
        version: 1,
        identity,
        hydrationSha256: validateHydration(path, identity),
        observer: worker.observer,
        outcome: "success",
        logSha256: sha256(bytes),
        selectedSha256: sha256(selectedText),
        statusSha256: sha256(statusText),
        counters: validateObserverLog(worker.observer, bytes),
      });
    }
    assert.deepEqual(verifyCanonicalArtifacts(selected, root, identity), identity);
    const path = join(root, selected.workers[1].artifactName);
    for (const key of ["sha", "tree", "runId", "attempt", "repository", "gitlinksSha256"])
      assert.throws(() =>
        verifyCanonicalArtifacts(selected, root, { ...identity, [key]: "foreign" }),
      );
    const originalFiles = readFileSync(join(path, "files.json"));
    const changed = structuredClone(files);
    changed[0][1] = "b".repeat(64);
    write(join(path, "files.json"), changed);
    assert.throws(
      () => verifyCanonicalArtifacts(selected, root, identity),
      /manifest was replaced/,
    );
    const snapshot = readFileSync(join(path, "identity.json"));
    const receipt = readFileSync(join(path, "observer.json"));
    const forgedIdentity = { ...identity, filesSha256: sha256(JSON.stringify(changed)) };
    write(join(path, "identity.json"), forgedIdentity);
    write(join(path, "observer.json"), { ...JSON.parse(receipt), identity: forgedIdentity });
    assert.throws(() => verifyCanonicalArtifacts(selected, root, identity), /filesSha256 changed/);
    writeFileSync(join(path, "identity.json"), snapshot);
    writeFileSync(join(path, "observer.json"), receipt);
    writeFileSync(join(path, "files.json"), originalFiles);
    const originalLog = readFileSync(join(path, "ssr-pug.log"));
    writeFileSync(
      join(path, "ssr-pug.log"),
      Buffer.concat([originalLog, Buffer.from("foreign bytes\n")]),
    );
    assert.throws(() => verifyCanonicalArtifacts(selected, root, identity), /log was replaced/);
    writeFileSync(join(path, "ssr-pug.log"), originalLog);
    mkdirSync(join(root, "unselected-worker"));
    assert.throws(
      () => verifyCanonicalArtifacts(selected, root, identity),
      /Incomplete canonical artifacts/,
    );
    rmSync(join(root, "unselected-worker"), { recursive: true });
    rmSync(path, { recursive: true });
    assert.throws(
      () => verifyCanonicalArtifacts(selected, root, identity),
      /Incomplete canonical artifacts/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
