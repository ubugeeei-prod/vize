import assert from "node:assert/strict";
import { test } from "node:test";
import { publicationState } from "../../tools/support/compat/github/docs-deployment-history.ts";
import { custodyFixture } from "./support/docs-deployment-custody.ts";
import { repositoryId, run, sourceSha } from "./support/docs-deployment.ts";
import type { PublisherJob } from "../../tools/support/compat/github/docs-deployment-policy.ts";
import { legacyWorkflowHash } from "../../tools/support/compat/github/docs-deployment-custody.ts";

function history() {
  // The authenticated publisher is independent of the missing migration anchor.
  const value = custodyFixture(99001);
  const { api, deployment, publisher, job, receipt, values } = value;
  const lists = new Map<string, unknown[]>([
    [
      "/actions/workflows/deploy-docs.yml/runs?branch=main&created=" +
        encodeURIComponent(">=1970-01-01T00:00:00Z"),
      [],
    ],
    [
      "/actions/workflows/deploy-docs.yml/runs?branch=main&created=" +
        encodeURIComponent(">=" + receipt.outstanding.since),
      [],
    ],
  ]);
  values.set(
    "/deployments?environment=vize-docs-source-custody&task=vize-docs-pages-v1&per_page=100&page=1",
    [deployment],
  );
  values.set("/deployments?environment=github-pages&per_page=100&page=1", []);
  const requests: string[] = [];
  const json = api.json.bind(api);
  api.json = async <T>(path: string): Promise<T> => {
    requests.push(path);
    assert(
      !path.includes("/actions/runs/38013719817"),
      "Missing obsolete legacy metadata cannot block current custody",
    );
    return await json<T>(path);
  };
  api.list = async <T>(path: string): Promise<T[]> => {
    requests.push(path);
    assert(lists.has(path), "Only journal-bounded primary history: " + path);
    return lists.get(path) as T[];
  };
  api.manifest = async () => {
    assert.fail("Current authentic custody never re-downloads old archives");
  };
  return { ...value, lists, requests, publisher, job };
}

test("new authenticated actual custody supersedes missing legacy metadata and unrelated old history", async () => {
  const { api, publisher, requests } = history();
  const first = await publicationState(api, repositoryId, 0);
  assert.equal(first.floor?.sourceSha, sourceSha);
  assert.equal(first.floor?.receipt.publisher.runId, publisher.id);
  assert.deepEqual(first.outstanding.publisherAttempts, []);
  assert.deepEqual(first.outstanding.environmentIds, []);
  assert(requests.every((path) => !path.includes("38013719817")));
  assert.equal(
    requests.filter((path) => path.startsWith("/deployments?environment=github-pages")).length,
    1,
  );
  const { api: failedCleanup, publisher: actual } = history();
  actual.conclusion = "failure";
  assert.equal(
    (await publicationState(failedCleanup, repositoryId, 0)).floor?.sourceSha,
    sourceSha,
  );
});

test("a publisher queued before the anchor remains checked until its real terminal Pages effect", async () => {
  const { api, receipt, values, lists } = history();
  const older = run({
    id: 88001,
    path: ".github/workflows/deploy-docs.yml",
    event: "workflow_run",
    status: "in_progress",
    conclusion: null,
  });
  const job: PublisherJob = {
    id: 77001,
    run_id: older.id,
    run_attempt: 1,
    head_sha: older.head_sha,
    status: "in_progress",
    steps: [
      { name: "Deploy to GitHub Pages", conclusion: null, started_at: null, completed_at: null },
    ],
  };
  receipt.outstanding.publisherAttempts.push({ runId: older.id, attempt: 1 });
  values.set("/actions/runs/" + older.id + "/attempts/1", older);
  lists.set("/actions/runs/" + older.id + "/attempts/1/jobs", [job]);
  api.workflowHash = async () => legacyWorkflowHash;
  assert.deepEqual((await publicationState(api, repositoryId, 0)).outstanding.publisherAttempts, [
    { runId: older.id, attempt: 1 },
  ]);
  older.status = "completed";
  older.conclusion = "failure";
  job.status = "completed";
  job.steps[0] = {
    ...job.steps[0],
    started_at: "2026-10-10T01:40:00Z",
    completed_at: "2026-10-10T01:40:10Z",
    conclusion: "success",
  };
  await assert.rejects(
    () => publicationState(api, repositoryId, 0),
    /later Pages effect lacks authenticated/,
  );
  job.steps[0].conclusion = "failure";
  await assert.rejects(
    () => publicationState(api, repositoryId, 0),
    /later Pages effect lacks authenticated/,
  );
  job.steps[0].conclusion = "skipped";
  assert.deepEqual(
    (await publicationState(api, repositoryId, 0)).outstanding.publisherAttempts,
    [],
  );
});

test("an older pending unknown environment and new unknown writers cannot disappear across the anchor", async () => {
  const { api, receipt, values, lists, deployment } = history();
  const older = {
    ...deployment,
    id: 555,
    environment: "github-pages",
    created_at: "2026-10-10T00:00:00Z",
  };
  receipt.outstanding.environmentIds.push(older.id);
  values.set("/deployments/" + older.id, older);
  lists.set("/deployments/" + older.id + "/statuses", [{ state: "waiting", log_url: "" }]);
  await assert.rejects(() => publicationState(api, repositoryId, 0), /without primary job custody/);
  const untrusted = run({ id: 333, path: ".github/workflows/unknown.yml", event: "workflow_run" });
  const effect: PublisherJob = {
    id: 444,
    run_id: untrusted.id,
    run_attempt: 1,
    head_sha: older.sha,
    status: "completed",
    steps: [
      {
        name: "Deploy to GitHub Pages",
        started_at: "2026-10-10T01:40:00Z",
        completed_at: "2026-10-10T01:40:01Z",
        conclusion: "success",
      },
    ],
  };
  values.set("/actions/jobs/444", effect);
  values.set("/actions/runs/333/attempts/1", untrusted);
  lists.set("/deployments/" + older.id + "/statuses", [
    { state: "success", log_url: "https://github.com/ubugeeei-prod/vize/actions/runs/333/job/444" },
  ]);
  await assert.rejects(
    () => publicationState(api, repositoryId, 0),
    /later Pages environment effect/,
  );
  receipt.outstanding.environmentIds.length = 0;
  values.set("/deployments?environment=github-pages&per_page=100&page=1", [
    { ...older, created_at: "2026-10-10T01:40:00Z" },
  ]);
  await assert.rejects(
    () => publicationState(api, repositoryId, 0),
    /later Pages environment effect/,
  );
});

test("missing current authenticated metadata never falls back to the older legacy floor", async () => {
  const { api, receipt, values } = history();
  values.delete("/actions/artifacts/" + receipt.pagesArtifact.id);
  await assert.rejects(() => publicationState(api, repositoryId, 0), /Only exact primary identity/);
});

test("pending guarded publishers proceed while pending unguarded publishers refuse before publication", async () => {
  const { api, receipt, values, lists } = history();
  const queued = run({
    id: 88001,
    path: ".github/workflows/deploy-docs.yml",
    event: "workflow_run",
    status: "queued",
    conclusion: null,
  });
  receipt.outstanding.publisherAttempts.push({ runId: queued.id, attempt: 1 });
  values.set("/actions/runs/" + queued.id + "/attempts/1", queued);
  lists.set("/actions/runs/" + queued.id + "/attempts/1/jobs", []);
  const guardedHash = "a".repeat(64);
  api.workflowHash = async () => guardedHash;
  assert.deepEqual(
    (await publicationState(api, repositoryId, 0, guardedHash)).outstanding.publisherAttempts,
    [{ runId: queued.id, attempt: 1 }],
  );
  api.workflowHash = async () => "b".repeat(64);
  await assert.rejects(
    () => publicationState(api, repositoryId, 0, guardedHash),
    /pending writer must prove its whole-job lock/,
  );
});

test("a truly absent site audits pending writers without requiring a nonexistent legacy floor", async () => {
  const { api, publisher, job, values, lists, deployment, requests } = history();
  const empty = await publicationState(api, repositoryId, 0, null, false);
  assert.equal(empty.floor, null);
  assert.deepEqual(empty.outstanding.environmentIds, []);
  assert(
    requests.every(
      (path) => !path.includes("vize-docs-source-custody") && !path.includes("38013719817"),
    ),
  );
  const environment = { ...deployment, id: 555, environment: "github-pages" };
  values.set("/deployments?environment=github-pages&per_page=100&page=1", [environment]);
  lists.set("/deployments/555/statuses", [{ state: "waiting", log_url: "" }]);
  await assert.rejects(
    () => publicationState(api, repositoryId, 0, null, false),
    /without primary job custody/,
  );
  lists.set("/deployments/555/statuses", [
    {
      state: "waiting",
      log_url:
        "https://github.com/ubugeeei-prod/vize/actions/runs/" + publisher.id + "/job/" + job.id,
    },
  ]);
  publisher.status = job.status = "queued";
  publisher.conclusion = null;
  job.steps = [
    { name: "Deploy to GitHub Pages", conclusion: null, started_at: null, completed_at: null },
  ];
  api.workflowHash = async () => legacyWorkflowHash;
  assert.deepEqual(
    (await publicationState(api, repositoryId, 0, null, false)).outstanding.environmentIds,
    [555],
  );
  api.workflowHash = async () => "a".repeat(64);
  await assert.rejects(
    () => publicationState(api, repositoryId, 0, null, false),
    /pending writer must prove/,
  );
  job.steps[0].started_at = "2026-10-10T01:36:00Z";
  await assert.rejects(
    () => publicationState(api, repositoryId, 0, null, false),
    /uncertain actual Pages effect/,
  );
});

test("site absence cannot hide queued publisher attempts that have no environment record yet", async () => {
  const { api, values, lists } = history();
  const queued = run({
    id: 88001,
    path: ".github/workflows/deploy-docs.yml",
    event: "workflow_run",
    status: "queued",
    conclusion: null,
  });
  lists.set(
    "/actions/workflows/deploy-docs.yml/runs?branch=main&created=" +
      encodeURIComponent(">=1970-01-01T00:00:00Z"),
    [queued],
  );
  values.set("/actions/runs/88001/attempts/1", queued);
  lists.set("/actions/runs/88001/attempts/1/jobs", []);
  const guardedHash = "a".repeat(64);
  api.workflowHash = async () => "b".repeat(64);
  await assert.rejects(
    () => publicationState(api, repositoryId, 0, guardedHash, false),
    /pending writer must prove/,
  );
  for (const hash of [legacyWorkflowHash, guardedHash]) {
    api.workflowHash = async () => hash;
    const result = await publicationState(api, repositoryId, 0, guardedHash, false);
    assert.equal(result.floor, null);
    assert.deepEqual(result.outstanding.publisherAttempts, [{ runId: 88001, attempt: 1 }]);
    assert.deepEqual(result.outstanding.environmentIds, []);
  }
  queued.status = "in_progress";
  lists.set("/actions/runs/88001/attempts/1/jobs", [
    {
      id: 77001,
      run_id: 88001,
      run_attempt: 1,
      head_sha: queued.head_sha,
      status: "in_progress",
      steps: [
        {
          name: "Deploy to GitHub Pages",
          conclusion: null,
          started_at: "2026-10-10T01:36:00Z",
          completed_at: null,
        },
      ],
    },
  ]);
  await assert.rejects(
    () => publicationState(api, repositoryId, 0, guardedHash, false),
    /uncertain actual Pages effect/,
  );
});
