import assert from "node:assert/strict";
import { test } from "node:test";
import { readRustWorkerSelection } from "../../tools/support/compat/github/select-rust-workers.mjs";
import { PendingRustWorkerMetadata } from "../../tools/support/compat/github/rust-worker-selection.mjs";
import { context, fixture } from "./_helpers/rust-worker-selection-fixture.mjs";

function staleStep(value, shard = 1, step = 1, status = "in_progress") {
  const latest = value.jobs.find((job) => job.id === 200 + shard);
  Object.assign(latest.steps[step], {
    status,
    conclusion: null,
    completed_at: null,
    ...(status === "queued" ? { started_at: null } : {}),
  });
  return value;
}

function transport(snapshots, afterRun) {
  let runReads = 0;
  const calls = [];
  const waits = [];
  const get = (index) => snapshots[Math.min(index, snapshots.length - 1)];
  return {
    calls,
    waits,
    options: {
      apiUrl: "https://api.example.invalid",
      repository: context.repository,
      token: "synthetic-test-only",
      waitImpl: async (milliseconds) => waits.push(milliseconds),
      fetchImpl: async (url) => {
        calls.push(url.pathname + url.search);
        let body;
        if (url.pathname.endsWith("/jobs") || url.pathname.endsWith("/artifacts")) {
          const value = get(Math.floor((runReads - 1) / 2));
          const collection = url.pathname.endsWith("/jobs") ? "jobs" : "artifacts";
          if (collection === "jobs") assert.equal(url.searchParams.get("filter"), "all");
          const start = (Number(url.searchParams.get("page")) - 1) * 100;
          body = { [collection]: value[collection].slice(start, start + 100) };
        } else {
          const index = Math.floor(runReads / 2);
          const value = get(index);
          body = runReads++ % 2 && afterRun ? afterRun(value.run, index) : value.run;
        }
        return new Response(JSON.stringify(body));
      },
    },
  };
}

await test("a completed-success worker's stale step is freshly observed, never marked complete", async () => {
  for (const [step, status] of [
    [1, "in_progress"],
    [1, "queued"],
    [3, "in_progress"],
    [3, "queued"],
  ]) {
    const pending = staleStep(fixture(), 1, step, status);
    const api = transport([pending, fixture()]);
    const result = await readRustWorkerSelection(context, api.options);
    assert.deepEqual(api.waits, [2000]);
    assert.equal(pending.jobs[4].steps[step].status, status);
    assert.equal(pending.jobs[4].steps[step].completed_at, null);
    assert.deepEqual(
      result.workers.map((worker) => worker.latestJobId),
      [201, 202, 203, 204],
    );
    assert.deepEqual(
      result.workers.map((worker) => worker.artifactId),
      [2001, 1002, 1003, 1004],
    );
    assert.equal(api.calls.filter((url) => url.includes("/jobs?")).length, 2);
  }
});

await test("all inventories are repaginated at most six times before pending metadata refuses", async () => {
  const value = staleStep(fixture());
  value.jobs.push(
    ...Array.from({ length: 100 - value.jobs.length }, () => ({ name: "unrelated" })),
  );
  value.artifacts.push(
    ...Array.from({ length: 100 - value.artifacts.length }, () => ({ name: "unrelated" })),
  );
  const api = transport([value]);
  await assert.rejects(readRustWorkerSelection(context, api.options), PendingRustWorkerMetadata);
  assert.deepEqual(api.waits, Array(5).fill(2000));
  for (const resource of ["jobs", "artifacts"]) {
    for (const page of [1, 2]) {
      assert.equal(
        api.calls.filter(
          (url) =>
            url.includes(`/${resource}?`) &&
            new URL(url, "https://api.example.invalid").searchParams.get("page") === String(page),
        ).length,
        6,
      );
    }
  }
});

await test("a later-page terminal red, cancelled or skipped worker is never retried behind pending metadata", async () => {
  for (const conclusion of ["failure", "cancelled", "skipped", "timed_out"]) {
    const value = staleStep(fixture());
    const red = value.jobs.pop();
    red.conclusion = conclusion;
    value.jobs.push(
      ...Array.from({ length: 100 - value.jobs.length }, () => ({ name: "unrelated" })),
      red,
    );
    const api = transport([value]);
    await assert.rejects(
      readRustWorkerSelection(context, api.options),
      /Latest worker.*did not succeed/,
    );
    assert.deepEqual(api.waits, []);
  }
});

await test("missing, foreign or duplicate artifact and malformed worker identities never authorize retries", async () => {
  for (const mutate of [
    (value) => value.artifacts.splice(0, 1),
    (value) => {
      value.artifacts[0].workflow_run.head_sha = "b".repeat(40);
    },
    (value) => value.artifacts.push(structuredClone(value.artifacts[0])),
    (value) => {
      value.jobs[7].head_sha = "b".repeat(40);
    },
    (value) => value.jobs.push(structuredClone(value.jobs[7])),
    (value) => {
      value.jobs[4].steps[1].number = 1;
    },
    (value) => {
      value.jobs[4].steps[1].conclusion = "failure";
    },
    (value) => {
      value.jobs[4].steps[1].started_at = "2026-10-04T03:00:00Z";
    },
    (value) => {
      value.jobs[4].steps[1].status = "unknown";
    },
  ]) {
    const value = staleStep(fixture());
    mutate(value);
    const api = transport([value]);
    await assert.rejects(
      readRustWorkerSelection(context, api.options),
      (error) => !(error instanceof PendingRustWorkerMetadata),
    );
    assert.deepEqual(api.waits, []);
  }
});

await test("pending required steps cannot borrow older green artifacts or incomplete worker jobs", async () => {
  for (const mutate of [
    (value) => staleStep(value, 2),
    (value) => {
      value.jobs[4].status = "in_progress";
      value.jobs[4].conclusion = null;
    },
    (value) => {
      value.jobs[4].steps[1].started_at = null;
    },
  ]) {
    const value = staleStep(fixture());
    mutate(value);
    const api = transport([value]);
    await assert.rejects(
      readRustWorkerSelection(context, api.options),
      (error) => !(error instanceof PendingRustWorkerMetadata),
    );
    assert.deepEqual(api.waits, []);
  }
});

await test("run, source, repository, latest job or artifact changes during retry refuse the whole selection", async () => {
  for (const mutate of [
    (value) => {
      value.run.run_attempt = 3;
    },
    (value) => {
      value.run.head_sha = "b".repeat(40);
    },
    (value) => {
      value.run.repository.id = 43;
      value.artifacts.forEach((artifact) => {
        artifact.workflow_run.repository_id = 43;
        artifact.workflow_run.head_repository_id = 43;
      });
    },
    (value) => {
      value.jobs[4].id = 901;
    },
    (value) => {
      value.artifacts[0].digest = `sha256:${"b".repeat(64)}`;
    },
  ]) {
    const next = fixture();
    mutate(next);
    const api = transport([staleStep(fixture()), next]);
    await assert.rejects(
      readRustWorkerSelection(context, api.options),
      /changed|Foreign workflow source/,
    );
    assert.deepEqual(api.waits, [2000]);
  }
});

await test("a pending read's post-inventory run identity and eventual full timestamps remain mandatory", async () => {
  const api = transport([staleStep(fixture())], (run) => ({ ...run, run_attempt: 3 }));
  await assert.rejects(readRustWorkerSelection(context, api.options), /Workflow attempt changed/);
  assert.deepEqual(api.waits, []);
  const invalid = fixture();
  invalid.jobs[4].steps[1].completed_at = null;
  const next = transport([staleStep(fixture()), invalid]);
  await assert.rejects(readRustWorkerSelection(context, next.options), /UTC timestamp/);
  assert.deepEqual(next.waits, [2000]);
});
