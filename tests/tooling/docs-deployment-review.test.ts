import assert from "node:assert/strict";
import { test } from "node:test";
import * as metadata from "../../tools/support/compat/github/docs-deployment-api.ts";
import { publicationState } from "../../tools/support/compat/github/docs-deployment-history.ts";
import { publishedReceipt } from "../../tools/support/compat/github/docs-deployment-policy.ts";
import { history } from "./support/docs-deployment-history.ts";
import { custodyFixture } from "./support/docs-deployment-custody.ts";
import { build, repositoryId, run, sourceSha } from "./support/docs-deployment.ts";

test("an older unretained rerun without an environment must prove its pending writer guard", async () => {
  const { api, lists, values, receipt } = history();
  const queued = run({
    id: 88001,
    run_attempt: 2,
    path: ".github/workflows/deploy-docs.yml",
    event: "workflow_run",
    status: "queued",
    conclusion: null,
  });
  assert.deepEqual(receipt.outstanding.publisherAttempts, []);
  values.set("/actions/runs/88001/attempts/2", queued);
  lists.set("/actions/runs/88001/attempts/2/jobs", []);
  const guardedHash = "a".repeat(64);
  for (const status of ["queued", "in_progress", "waiting", "pending", "requested"]) {
    queued.status = status;
    const path = "/actions/workflows/deploy-docs.yml/runs?branch=main&status=" + status;
    lists.set(path, [queued]);
    api.workflowHash = async () => "b".repeat(64);
    await assert.rejects(
      () => publicationState(api, repositoryId, 0, guardedHash),
      /pending writer must prove/,
    );
    api.workflowHash = async () => guardedHash;
    const current = await publicationState(api, repositoryId, 0, guardedHash);
    assert.equal(current.floor?.sourceSha, sourceSha);
    assert.deepEqual(current.outstanding.publisherAttempts, [{ runId: 88001, attempt: 2 }]);
    lists.set(path, []);
  }
  assert(
    !values.has("/actions/runs/88001/attempts/1"),
    "Newly discovered old active runs do not force obsolete attempts into the bounded journal",
  );
  lists.set("/actions/workflows/deploy-docs.yml/runs?branch=main&status=queued", [queued]);
  await assert.rejects(() => {
    api.workflowHash = async () => "b".repeat(64);
    return publicationState(api, repositoryId, 0, guardedHash, false);
  }, /pending writer must prove/);
});

test("newer skipped and unstarted candidate custody retains the previous authenticated floor", async () => {
  for (const conclusion of ["skipped", null]) {
    const { api, lists, values, deployment } = history();
    const candidate = custodyFixture(88001);
    candidate.deployment.id = 100;
    candidate.job.id = candidate.receipt.publisher.jobId = 77001;
    candidate.publisher.status = conclusion === null ? "queued" : "completed";
    candidate.publisher.conclusion = conclusion;
    candidate.job.status = candidate.publisher.status;
    candidate.job.steps = [
      {
        name: "Deploy to GitHub Pages",
        conclusion,
        started_at: null,
        completed_at: conclusion === null ? null : "2026-10-10T01:40:00Z",
      },
    ];
    values.set("/actions/runs/88001/attempts/1", candidate.publisher);
    values.set("/actions/jobs/77001", candidate.job);
    lists.set("/actions/runs/88001/attempts/1/jobs", [candidate.job]);
    values.set(
      "/deployments?environment=vize-docs-source-custody&task=vize-docs-pages-v1&per_page=100&page=1",
      [candidate.deployment, deployment],
    );
    const guardedHash = "a".repeat(64);
    api.workflowHash = async () => guardedHash;
    const current = await publicationState(api, repositoryId, 0, guardedHash);
    assert.equal(current.floor?.sourceSha, sourceSha);
    assert.deepEqual(
      current.outstanding.publisherAttempts,
      conclusion === null ? [{ runId: 88001, attempt: 1 }] : [],
    );
    if (conclusion === null) {
      api.workflowHash = async () => "b".repeat(64);
      await assert.rejects(
        () => publicationState(api, repositoryId, 0, guardedHash),
        /pending writer must prove/,
      );
      api.workflowHash = async () => guardedHash;
      candidate.job.steps[0].started_at = "2026-10-10T01:40:00Z";
      await assert.rejects(
        () => publicationState(api, repositoryId, 0, guardedHash),
        /later Pages effect lacks authenticated/,
      );
    }
  }
});

test("receipt serialization uses runtime-independent code-unit key order", () => {
  const value = { z: [{ b: 1, a: 2 }], ä: 3, Z: 4, A: { z: 1, Z: 2 } };
  assert.equal(metadata.stableJson(value), '{"A":{"Z":2,"z":1},"Z":4,"z":[{"a":2,"b":1}],"ä":3}');
});

test("terminal Pages metadata refresh is bounded and never upgrades an uncertain or skipped effect", async () => {
  const refresh = metadata.terminalPagesMetadata;
  assert.equal(typeof refresh, "function");
  const fixture = () => custodyFixture().job;
  let reads = 0;
  let pauses = 0;
  const read = async () => {
    reads++;
    const job = fixture();
    if (reads < 3) job.steps.at(-1)!.conclusion = job.steps.at(-1)!.completed_at = null;
    return { job };
  };
  const result = await refresh(read, async () => {
    pauses++;
  });
  assert.equal(result.job.steps.at(-1)!.conclusion, "success");
  assert.equal(reads, 3);
  assert.equal(pauses, 2);
  reads = pauses = 0;
  const stale = await refresh(
    async () => {
      reads++;
      const job = fixture();
      job.steps.at(-1)!.conclusion = job.steps.at(-1)!.completed_at = null;
      return { job };
    },
    async () => {
      pauses++;
    },
  );
  assert.equal(reads, 6);
  assert.equal(pauses, 5);
  assert.equal(stale.job.steps.at(-1)!.conclusion, null);
  const candidate = custodyFixture();
  assert.throws(
    () =>
      publishedReceipt(candidate.receipt, candidate.publisher, stale.job, build(), candidate.pages),
    /uncertain Pages effect/,
  );
  for (const conclusion of ["skipped", "failure"]) {
    reads = pauses = 0;
    const terminal = await refresh(
      async () => {
        reads++;
        const job = fixture();
        job.steps.at(-1)!.conclusion = conclusion;
        return { job };
      },
      async () => {
        pauses++;
      },
    );
    assert.equal(terminal.job.steps.at(-1)!.conclusion, conclusion);
    assert.equal(reads, 1);
    assert.equal(pauses, 0);
    if (conclusion === "skipped")
      assert.equal(
        publishedReceipt(
          candidate.receipt,
          candidate.publisher,
          terminal.job,
          build(),
          candidate.pages,
        ),
        null,
      );
    else
      assert.throws(
        () =>
          publishedReceipt(
            candidate.receipt,
            candidate.publisher,
            terminal.job,
            build(),
            candidate.pages,
          ),
        /uncertain Pages effect/,
      );
  }
});
