import assert from "node:assert/strict";
import { execFile, spawnSync } from "node:child_process";
import { createServer } from "node:http";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  qualifyCheck,
  operatorDeadline,
  type Api,
} from "../../tools/support/release/qualify_check/main.ts";

type Row = Record<string, unknown>;
const candidate = {
  repository: "owner/repo",
  branch: "release/v0.500.0",
  head: "a".repeat(40),
  cut: "b".repeat(40),
};
// Independently authored whole full-Check contract; no production name generator supplies these.
const required = [
  "test-scripts",
  "test-js-packages",
  "build-js-packages",
  "full-js-report",
  "Full Rust workspace and doctests",
  "clippy-and-test",
  "full-rust-report",
  ...[
    "vize_armature",
    "vize_atelier_core",
    "vize_atelier_dom",
    "vize_atelier_sfc",
    "vize_atelier_ssr",
    "vize_atelier_vapor",
    "vize_carton",
    "vize_croquis",
    "vize_fresco",
    "vize_musea",
    "vize_relief",
    "vize_l1_to_l2",
  ].map((name) => `cargo-semver-checks (${name})`),
];
const run = (changes: Row = {}): Row => ({
  id: 71,
  run_attempt: 1,
  head_sha: candidate.head,
  head_branch: candidate.branch,
  name: "Check",
  path: ".github/workflows/check.yml",
  event: "workflow_dispatch",
  display_title: `Check full @ ${candidate.head}`,
  status: "completed",
  conclusion: "success",
  run_started_at: "2026-10-17T00:00:00Z",
  ...changes,
});
const jobs = (): Row[] =>
  required.map((name, index) => ({
    id: index + 100,
    name,
    run_id: 71,
    run_attempt: 1,
    head_sha: candidate.head,
    status: "completed",
    conclusion: "success",
  }));
function harness(snapshots: Row[][], observed: Row[] | (() => Row[]) = jobs()) {
  let time = 1000;
  let index = 0;
  let refReads = 0;
  let moveAt = Infinity;
  const writes: { resource: string; body: unknown }[] = [];
  const logs: string[] = [];
  const calls: { resource: string; collection: string; query?: Record<string, string> }[] = [];
  const api: Api = {
    read: async (resource) => {
      assert.equal(resource, `git/ref/heads/${candidate.branch}`);
      return {
        ref: `refs/heads/${candidate.branch}`,
        object: { type: "commit", sha: ++refReads >= moveAt ? candidate.cut : candidate.head },
      };
    },
    pages: async (resource, collection, query) => {
      calls.push({ resource, collection, query });
      if (resource === "actions/runs")
        return structuredClone(snapshots[Math.min(index++, snapshots.length - 1)]);
      assert.equal(resource, "actions/runs/71/jobs");
      assert.equal(collection, "jobs");
      assert.deepEqual(query, { filter: "latest" });
      return structuredClone(typeof observed === "function" ? observed() : observed);
    },
    dispatch: async (resource, body) => {
      writes.push({ resource, body });
    },
  };
  return {
    writes,
    logs,
    calls,
    move: (at: number) => {
      moveAt = at;
    },
    options: {
      api,
      deadline: 100_000,
      now: () => time,
      sleep: async (delay: number) => {
        time += delay;
      },
      log: (message: string) => logs.push(message),
    },
  };
}

test("cold H dispatches only Check once, waits for the complete exact candidate and preserves its URL", async () => {
  const f = harness([[], [run({ status: "in_progress", conclusion: null })], [run()]]);
  const result = await qualifyCheck(candidate, f.options);
  assert.equal(result.id, 71);
  assert.deepEqual(f.writes, [
    {
      resource: "actions/workflows/check.yml/dispatches",
      body: { ref: candidate.branch, inputs: { "release-qualification": true } },
    },
  ]);
  assert.equal(f.logs.filter((line) => line.startsWith("Dispatching")).length, 1);
  assert.ok(f.logs.some((line) => line.includes("https://github.com/owner/repo/actions/runs/71")));
  assert.match(f.logs.at(-1)!, /^Qualified frozen [a-f0-9]{40}:/);
  assert.equal(f.calls.filter((call) => call.resource === "actions/runs").length, 4);
  for (const call of f.calls.filter((call) => call.resource === "actions/runs"))
    assert.deepEqual(call.query, { head_sha: candidate.head });
});

test("an already qualified H is rechecked without a second dispatch", async () => {
  assert.equal(required.length, 19);
  const f = harness([[run()]]);
  await qualifyCheck(candidate, f.options);
  assert.deepEqual(f.writes, []);
  assert.equal(f.calls.filter((call) => call.resource.endsWith("/jobs")).length, 1);
});

test("every one of seven full jobs and twelve release SemVer jobs rejects absence, duplication and non-success", async () => {
  for (const name of required) {
    for (const conclusion of ["failure", "cancelled", "skipped", "missing", "duplicate"]) {
      let observed = jobs();
      if (conclusion === "missing") observed = observed.filter((job) => job.name !== name);
      else if (conclusion === "duplicate")
        observed.push({ ...observed.find((job) => job.name === name)! });
      else observed.find((job) => job.name === name)!.conclusion = conclusion;
      const f = harness([[run()]], observed);
      await assert.rejects(
        qualifyCheck(candidate, f.options),
        (error: unknown) => error instanceof Error,
        `${name}/${conclusion}`,
      );
      assert.deepEqual(f.writes, []);
      assert.equal(
        f.logs.some((line) => line.startsWith("Qualified")),
        false,
      );
    }
  }
});

test("a terminal required-job failure stops an in-progress run instead of waiting for publication", async () => {
  const observed = jobs();
  observed[0].conclusion = "failure";
  const f = harness([[run({ status: "in_progress", conclusion: null })]], observed);
  await assert.rejects(qualifyCheck(candidate, f.options), /test-scripts=completed\/failure/);
  assert.deepEqual(f.writes, []);
});

test("wrong source, ref, workflow, trigger and title cannot stand in for a candidate's full Check", async () => {
  for (const changes of [
    { head_sha: candidate.cut },
    { head_branch: "main" },
    { head_branch: "release/v0.499.0" },
    { path: ".github/workflows/other.yml" },
    { event: "push" },
    { event: "schedule" },
    { display_title: `Check pull_request @ ${candidate.head}` },
  ]) {
    const f = harness([[run(changes)], [run()]]);
    await qualifyCheck(candidate, f.options);
    assert.equal(f.writes.length, 1);
  }
});

test("a newer failed or pending exact run never falls back to an older green run", async () => {
  for (const status of ["completed", "in_progress"]) {
    const newer = run({
      id: 72,
      status,
      conclusion: status === "completed" ? "failure" : null,
      run_started_at: "2026-10-17T00:01:00Z",
    });
    const f = harness([[run()], [run()], [run(), newer]]);
    await assert.rejects(
      qualifyCheck(candidate, f.options),
      /Check: (completed\/failure|in_progress)/,
    );
    assert.deepEqual(f.writes, []);
  }
});

test("rerun-failed preserves prior successful required jobs alongside the freshly repaired job", async () => {
  const observed = jobs();
  observed[0].run_attempt = 3;
  // All other successful jobs remain observations from attempt 1 in the latest effective graph.
  const f = harness([[run({ run_attempt: 3 })]], observed);
  const result = await qualifyCheck(candidate, f.options);
  assert.equal(result.run_attempt, 3);
  assert.deepEqual(f.writes, []);
  assert.equal(observed.filter((job) => job.run_attempt === 1).length, 18);
});

test("an old failed job cannot abort or satisfy a pending repair before its latest replacement appears", async () => {
  const prior = jobs();
  prior[0].conclusion = "failure";
  const repaired = jobs();
  repaired[0].run_attempt = 2;
  let reads = 0;
  const pending = run({ run_attempt: 2, status: "in_progress", conclusion: null });
  const f = harness([[pending], [pending], [run({ run_attempt: 2 })]], () =>
    ++reads === 1 ? prior : repaired,
  );
  await qualifyCheck(candidate, f.options);
  assert.equal(reads, 2);
  assert.deepEqual(f.writes, []);
  await assert.rejects(
    qualifyCheck(candidate, harness([[run({ run_attempt: 2 })]], prior).options),
    /test-scripts.*failure/,
  );
});

test("job evidence from another source, run or attempt refuses qualification", async () => {
  for (const changes of [
    { run_id: 72 },
    { run_attempt: 2 },
    { run_attempt: 0 },
    { run_attempt: -1 },
    { run_attempt: 1.5 },
    { run_attempt: "1" },
    { head_sha: candidate.cut },
  ]) {
    const observed = jobs();
    Object.assign(observed[0], changes);
    await assert.rejects(
      qualifyCheck(candidate, harness([[run()]], observed).options),
      /Check job belongs/,
    );
  }
});

test("moving the frozen release branch before dispatch or after successful jobs refuses handoff", async () => {
  for (const [snapshots, moveAt] of [
    [[], 1],
    [[run()], 4],
  ] as const) {
    const f = harness([[...snapshots]]);
    f.move(moveAt);
    await assert.rejects(qualifyCheck(candidate, f.options), /frozen release branch moved/);
    assert.deepEqual(f.writes, []);
  }
});

test("the inherited operator deadline cannot be extended and interrupts pending qualification", async () => {
  assert.equal(
    operatorDeadline(
      {
        VIZE_RELEASE_OPERATOR_TIMEOUT_SECONDS: "60",
        VIZE_RELEASE_OPERATOR_DEADLINE_UNIX_MILLIS: "2000",
      },
      1000,
    ),
    2000,
  );
  for (const env of [
    { VIZE_RELEASE_OPERATOR_TIMEOUT_SECONDS: "28801" },
    { VIZE_RELEASE_OPERATOR_TIMEOUT_SECONDS: "0" },
    { VIZE_RELEASE_OPERATOR_DEADLINE_UNIX_MILLIS: "1000" },
    { VIZE_RELEASE_OPERATOR_DEADLINE_UNIX_MILLIS: "9007199254740992" },
  ])
    assert.throws(() => operatorDeadline(env, 1000));
  const f = harness([[run({ status: "in_progress", conclusion: null })]]);
  f.options.deadline = 2000;
  await assert.rejects(qualifyCheck(candidate, f.options), /operator deadline expired/);
  assert.deepEqual(f.writes, []);
});

test("the actual CLI rejects an incomplete candidate before any authenticated action", () => {
  const result = spawnSync(
    process.execPath,
    [fileURLToPath(new URL("../../tools/support/release/qualify_check/main.ts", import.meta.url))],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 1);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /Usage: main.ts <repository> <release\/vTAG> <H> <C>/);
});

test("the actual CLI bounds transient API retry sleep by the inherited operator deadline", async (t) => {
  let requests = 0;
  const server = createServer((_request, response) => {
    requests++;
    response.writeHead(503, { "content-type": "application/json" });
    response.end('{"message":"transient"}');
  });
  await new Promise<void>((done) => server.listen(0, "127.0.0.1", done));
  t.after(() => server.close());
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  const started = Date.now();
  const entry = fileURLToPath(
    new URL("../../tools/support/release/qualify_check/main.ts", import.meta.url),
  );
  await assert.rejects(
    promisify(execFile)(
      process.execPath,
      [entry, candidate.repository, candidate.branch, candidate.head, candidate.cut],
      {
        timeout: 4000,
        env: {
          ...process.env,
          GITHUB_TOKEN: "unit-token",
          GITHUB_API_URL: `http://127.0.0.1:${address.port}`,
          VIZE_RELEASE_OPERATOR_TIMEOUT_SECONDS: "60",
          VIZE_RELEASE_OPERATOR_DEADLINE_UNIX_MILLIS: String(started + 1000),
        },
      },
    ),
    (error: unknown) => {
      assert.ok(error && typeof error === "object" && "stderr" in error);
      assert.match(String(error.stderr), /operator deadline expired/);
      return true;
    },
  );
  assert.equal(requests, 1, "no API retry may start after the inherited deadline");
  assert.ok(
    Date.now() - started < 2500,
    "existing transport's 31-second retry backoff must be bounded",
  );
});
