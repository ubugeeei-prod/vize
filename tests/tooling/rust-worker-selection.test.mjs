import assert from "node:assert/strict";
import { test } from "node:test";
import { readRustWorkerSelection } from "../../tools/support/compat/github/select-rust-workers.mjs";
import { verifyRustWorkerDirectories } from "../../tools/support/compat/github/rust-worker-selection.mjs";

import {
  artifact,
  context,
  fixture,
  job,
  select,
  time,
} from "./_helpers/rust-worker-selection-fixture.mjs";

test("failed-only rerun binds latest outcomes to original unchanged executions and immutable IDs", () => {
  const value = fixture();
  value.jobs[0].conclusion = "cancelled";
  value.artifacts = value.artifacts.filter((item) => item.id !== 1001);
  const result = select(value);
  assert.deepEqual(
    result.workers.map((worker) => [
      worker.shard,
      worker.latestJobId,
      worker.latestReportedAttempt,
      worker.executionJobId,
      worker.executionAttempt,
      worker.artifactId,
      worker.artifactName,
    ]),
    [
      [1, 201, 2, 201, 2, 2001, "rust-test-shard-1-123-2"],
      [2, 202, 2, 102, 1, 1002, "rust-test-shard-2-123-1"],
      [3, 203, 2, 103, 1, 1003, "rust-test-shard-3-123-1"],
      [4, 204, 2, 104, 1, 1004, "rust-test-shard-4-123-1"],
    ],
  );
  assert(result.workers.every((worker) => /^[a-f0-9]{64}$/.test(worker.executionSha256)));
});

test("sparse genuine jobs and full reruns retain four complete workers", () => {
  const sparse = fixture();
  sparse.jobs = sparse.jobs.filter((worker) => worker.id < 202);
  assert.deepEqual(
    select(sparse).workers.map((worker) => worker.executionAttempt),
    [2, 1, 1, 1],
  );
  const full = fixture();
  full.jobs = [...full.jobs.slice(0, 4), ...[1, 2, 3, 4].map((shard) => job(shard, 2))];
  full.artifacts.push(...[2, 3, 4].map((shard) => artifact(shard, 2)));
  assert.deepEqual(
    select(full).workers.map((worker) => worker.artifactId),
    [2001, 2002, 2003, 2004],
  );
});

test("a latest failure or incomplete outcome cannot fall back to an old successful artifact", () => {
  for (const conclusion of ["failure", "cancelled", "timed_out", "skipped", "neutral", null]) {
    const value = fixture();
    value.jobs[4].conclusion = conclusion;
    assert.throws(() => select(value), /Latest worker.*did not succeed/);
  }
  for (const status of ["queued", "in_progress", "waiting"]) {
    const value = fixture();
    value.jobs[4].status = status;
    assert.throws(() => select(value), /Latest worker.*is incomplete/);
  }
});

test("carried reuse requires exact original times, all steps, outcome and runner metadata", () => {
  for (const change of [
    (worker) => worker.steps.push({ ...worker.steps[0], number: 5, name: "different step" }),
    (worker) => {
      worker.started_at = time(2, 0);
      worker.completed_at = time(2, 10);
      worker.steps.forEach((step) => {
        step.started_at = step.started_at.replace("T01", "T02");
        step.completed_at = step.completed_at.replace("T01", "T02");
      });
    },
    (worker) => {
      worker.runner_id += 1;
    },
    (worker) => {
      worker.labels.push("changed");
    },
  ]) {
    const value = fixture();
    change(value.jobs[5]);
    assert.throws(() => select(value), /Expected exactly one artifact: rust-test-shard-2-123-2/);
  }
  const failedOriginal = fixture();
  failedOriginal.jobs[1].conclusion = "failure";
  assert.throws(
    () => select(failedOriginal),
    /Expected exactly one artifact: rust-test-shard-2-123-2/,
  );
});

test("new successful execution without its own artifact cannot fall back", () => {
  const value = fixture();
  value.artifacts = value.artifacts.filter((item) => item.id !== 2001);
  assert.throws(() => select(value), /Expected exactly one artifact: rust-test-shard-1-123-2/);
});

test("missing, duplicated, future and foreign workers cannot manufacture four workers", () => {
  for (const change of [
    (value) => {
      value.jobs = value.jobs.filter((worker) => !worker.name.endsWith("(4/4)"));
    },
    (value) => {
      value.jobs.push(structuredClone(value.jobs[4]));
    },
    (value) => {
      value.jobs.push({ ...value.jobs[4], id: 999 });
    },
    (value) => {
      value.jobs[4].run_attempt = 3;
    },
    (value) => {
      value.jobs[4].head_sha = "b".repeat(40);
    },
    (value) => {
      value.jobs[4].run_id += 1;
    },
    (value) => {
      value.jobs[4].name = value.jobs[4].name.replace("(1/4)", "(5/4)");
    },
  ])
    assert.throws(() => {
      const value = fixture();
      change(value);
      select(value);
    });
});

test("successful worker steps must execute archive, complete tests, capture and upload in order", () => {
  for (const change of [
    (worker) => {
      worker.steps[1].conclusion = "skipped";
    },
    (worker) => {
      worker.steps[2].status = "in_progress";
    },
    (worker) => {
      worker.steps.pop();
    },
    (worker) => {
      worker.steps[2].number = 1;
    },
    (worker) => {
      worker.steps[1].number = 8;
    },
    (worker) => {
      worker.steps[3].completed_at = time(2, 59);
    },
  ])
    assert.throws(() => {
      const value = fixture();
      change(value.jobs[4]);
      select(value);
    });
});

test("artifacts require unique IDs, exact run/source/repository, digest, lifetime and original upload", () => {
  for (const change of [
    (value) => {
      value.artifacts.push(structuredClone(value.artifacts[0]));
    },
    (value) => {
      value.artifacts[0].id = 1002;
    },
    (value) => {
      value.artifacts[0].expired = true;
    },
    (value) => {
      value.artifacts[0].size_in_bytes = 0;
    },
    (value) => {
      delete value.artifacts[0].digest;
    },
    (value) => {
      value.artifacts[0].digest = "sha256:wrong";
    },
    (value) => {
      value.artifacts[0].workflow_run.id += 1;
    },
    (value) => {
      value.artifacts[0].workflow_run.head_sha = "b".repeat(40);
    },
    (value) => {
      value.artifacts[0].workflow_run.repository_id += 1;
    },
    (value) => {
      value.artifacts[0].workflow_run.head_repository_id += 1;
    },
    (value) => {
      value.artifacts[0].created_at = time(1, 7);
    },
    (value) => {
      value.artifacts[0].created_at = time(2, 9);
    },
  ])
    assert.throws(() => {
      const value = fixture();
      change(value);
      select(value);
    });
});

test("workflow identity cannot drift across source, attempt, run, caller or event", () => {
  for (const change of [
    (run) => {
      run.head_sha = "b".repeat(40);
    },
    (run) => {
      run.id += 1;
    },
    (run) => {
      run.run_attempt += 1;
    },
    (run) => {
      run.event = "pull_request";
    },
    (run) => {
      run.path = ".github/workflows/other.yml";
    },
    (run) => {
      run.repository.full_name = "foreign/repo";
    },
  ])
    assert.throws(() => {
      const value = fixture();
      change(value.run);
      select(value);
    });
});

test("downloaded artifacts must be exactly the four selected directories", () => {
  const result = select(fixture());
  const entries = result.workers.map((worker) => ({
    name: worker.artifactName,
    isDirectory: () => true,
  }));
  verifyRustWorkerDirectories(result, entries);
  for (const wrong of [
    entries.slice(1),
    [...entries, entries[0]],
    [...entries.slice(1), { name: "rust-test-shard-1-123-1", isDirectory: () => true }],
    entries.map((entry, index) => ({ ...entry, isDirectory: () => index !== 0 })),
  ])
    assert.throws(() => verifyRustWorkerDirectories(result, wrong));
});

function transport(value, { jobTail = [], artifactTail = [], afterRun } = {}) {
  let runs = 0;
  const calls = [];
  return {
    calls,
    options: {
      apiUrl: "https://api.example.invalid",
      repository: context.repository,
      token: "test-only",
      fetchImpl: async (url) => {
        calls.push(url.pathname + url.search);
        let payload;
        if (url.pathname.endsWith("/jobs")) {
          assert.equal(url.searchParams.get("filter"), "all");
          payload = { jobs: url.searchParams.get("page") === "1" ? value.jobs : jobTail };
        } else if (url.pathname.endsWith("/artifacts")) {
          payload = {
            artifacts: url.searchParams.get("page") === "1" ? value.artifacts : artifactTail,
          };
        } else payload = ++runs === 1 || !afterRun ? value.run : afterRun;
        return new Response(JSON.stringify(payload));
      },
    },
  };
}

test("official all-attempt job and artifact inventories are fully paginated before selection", async () => {
  const value = fixture();
  value.jobs.push(
    ...Array.from({ length: 100 - value.jobs.length }, () => ({ name: "unrelated" })),
  );
  value.artifacts.push(
    ...Array.from({ length: 100 - value.artifacts.length }, () => ({ name: "unrelated" })),
  );
  const api = transport(value);
  const result = await readRustWorkerSelection(context, api.options);
  assert.deepEqual(
    result.workers.map((worker) => worker.artifactId),
    [2001, 1002, 1003, 1004],
  );
  assert(api.calls.some((url) => url.includes("/jobs?") && url.includes("page=2")));
  assert(api.calls.some((url) => url.includes("/artifacts?") && url.includes("page=2")));
});

test("a failed latest worker on a later API page cannot hide behind first-page success", async () => {
  const value = fixture();
  value.jobs = value.jobs.filter((worker) => worker.id < 202);
  value.jobs.push(
    ...Array.from({ length: 100 - value.jobs.length }, () => ({ name: "unrelated" })),
  );
  const api = transport(value, { jobTail: [{ ...job(2, 2), conclusion: "failure" }] });
  await assert.rejects(
    readRustWorkerSelection(context, api.options),
    /Latest worker.*did not succeed/,
  );
});

test("source or attempt changes during official inventory reads fail closed", async () => {
  for (const changed of [{ run_attempt: 3 }, { head_sha: "b".repeat(40) }]) {
    const value = fixture();
    const api = transport(value, { afterRun: { ...value.run, ...changed } });
    await assert.rejects(
      readRustWorkerSelection(context, api.options),
      /Workflow attempt changed|Foreign workflow source/,
    );
  }
});
