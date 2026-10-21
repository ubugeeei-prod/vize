import assert from "node:assert/strict";
import { test } from "node:test";
import { readCanonicalJobs } from "../../tools/support/compat/github/canonical-corpus-jobs.ts";

const options = {
  apiUrl: "https://api.github.com",
  serverUrl: "https://github.com",
  repository: "ubugeeei-prod/vize",
  token: "fixture",
};
const run = { id: 12, check_suite_id: 34, head_sha: "a".repeat(40) };
const api = `${options.apiUrl}/repos/${options.repository}`;
const details = `${options.serverUrl}/${options.repository}/actions/runs/12/job/`;
const check = (id: number, jobId: number, observer = "dom") => ({
  id,
  name: `PR source checks / canonical-corpus / canonical observer (${observer})`,
  check_suite: { id: 34 },
  head_sha: run.head_sha,
  url: `${api}/check-runs/${id}`,
  details_url: `${details}${jobId}`,
});
type Check = ReturnType<typeof check>;
const job = (entry: Check, attempt = 1) => ({
  id: Number(entry.details_url.split("/").at(-1)),
  check_run_url: entry.url,
  run_id: 12,
  run_attempt: attempt,
  name: entry.name,
  head_sha: run.head_sha,
  status: "completed",
  conclusion: "success",
  steps: [{ name: "Upload required observer evidence", number: 6, conclusion: "success" }],
  runner_id: 9,
  runner_name: "worker",
  labels: ["linux"],
});
type Job = ReturnType<typeof job>;

function provider(checks: Check[], jobs: Job[]) {
  const requested: string[] = [];
  const fetchImpl: typeof fetch = async (input) => {
    const url = new URL(input instanceof Request ? input.url : input);
    requested.push(url.pathname + url.search);
    if (url.pathname.endsWith("/check-suites/34/check-runs")) {
      assert.equal(url.searchParams.get("filter"), "all");
      assert.equal(url.searchParams.get("per_page"), "100");
      const start = (Number(url.searchParams.get("page")) - 1) * 100;
      return Response.json({
        total_count: checks.length,
        check_runs: checks.slice(start, start + 100),
      });
    }
    const result = jobs.find(
      (entry) => url.pathname === `/repos/${options.repository}/actions/jobs/${entry.id}`,
    );
    assert(result, `Unexpected API request: ${url.pathname}`);
    return Response.json(result);
  };
  return { fetchImpl, requested };
}

await test("all pages and attempts use distinct Actions job IDs and preserve full job metadata", async () => {
  const observers = [check(201, 301), check(202, 302), check(203, 303, "ssr-pug")];
  const jobs = observers.map((entry, index) => job(entry, index === 1 ? 2 : 1));
  const unrelated = Array.from({ length: 100 }, (_, index) => ({
    ...check(index + 1, index + 1),
    name: "unrelated check",
  }));
  const fixture = provider([...unrelated, ...observers], jobs);
  assert.deepEqual(await readCanonicalJobs({ ...options, ...fixture }, run), jobs);
  assert.equal(fixture.requested.length, 5);
  assert(fixture.requested.every((path) => !path.includes("/actions/runs/12/jobs")));
});

await test("inventory retains unknown, pending and repeated observer attempts for strict selection", async () => {
  const checks = [check(1, 21), check(2, 22), check(3, 23, "unknown")];
  const jobs = checks.map((entry, index) => job(entry, index + 1));
  jobs[1].status = "queued";
  jobs[1].conclusion = "";
  jobs[1].steps = [];
  const fixture = provider(checks, jobs);
  assert.deepEqual(await readCanonicalJobs({ ...options, ...fixture }, run), jobs);
});

await test("eventually consistent observer completion is read from the same exact job", async () => {
  const entry = check(1, 21);
  const complete = job(entry);
  const pending = { ...complete, status: "in_progress", conclusion: "", steps: [] };
  const fixture = provider([entry], [complete]);
  const read = fixture.fetchImpl;
  const waits: number[] = [];
  let jobReads = 0;
  const fetchImpl: typeof fetch = async (input) => {
    const url = new URL(input instanceof Request ? input.url : input);
    if (url.pathname.endsWith("/actions/jobs/21")) {
      jobReads += 1;
      return Response.json(jobReads === 1 ? pending : complete);
    }
    return read(input);
  };
  assert.deepEqual(
    await readCanonicalJobs(
      {
        ...options,
        fetchImpl,
        sleep: async (milliseconds) => {
          waits.push(milliseconds);
        },
      },
      run,
    ),
    [complete],
  );
  assert.equal(jobReads, 2);
  assert.deepEqual(waits, [1_000]);
});

await test("persistent pending observer metadata remains fail-closed after a bounded wait", async () => {
  const entry = check(1, 21);
  const pending = { ...job(entry), status: "in_progress", conclusion: "", steps: [] };
  const fixture = provider([entry], [pending]);
  const waits: number[] = [];
  let jobReads = 0;
  const fetchImpl: typeof fetch = async (input) => {
    const url = new URL(input instanceof Request ? input.url : input);
    if (url.pathname.endsWith("/actions/jobs/21")) jobReads += 1;
    return fixture.fetchImpl(input);
  };
  assert.deepEqual(
    await readCanonicalJobs(
      {
        ...options,
        fetchImpl,
        sleep: async (milliseconds) => {
          waits.push(milliseconds);
        },
      },
      run,
    ),
    [pending],
  );
  assert.equal(jobReads, 10);
  assert.deepEqual(waits, Array.from({ length: 9 }, () => 1_000));
});

await test("foreign suites, heads, check URLs and duplicate check IDs are rejected", async () => {
  for (const change of [
    { check_suite: { id: 35 } },
    { head_sha: "b".repeat(40) },
    { url: `${api}/check-runs/2` },
    { id: 0 },
  ]) {
    const entry = { ...check(1, 21), ...change };
    await assert.rejects(readCanonicalJobs({ ...options, ...provider([entry], []) }, run));
  }
  const entry = check(1, 21);
  await assert.rejects(
    readCanonicalJobs({ ...options, ...provider([entry, entry], []) }, run),
    /Duplicate canonical check ID/,
  );
});

await test("job links cannot select another host, repository, run, credential or noncanonical ID", async () => {
  for (const details_url of [
    `${details}21`.replace("github.com", "foreign.example"),
    `${details}21`.replace("ubugeeei-prod/vize", "other/repo"),
    `${details}21`.replace("/runs/12/", "/runs/13/"),
    `${details}21`.replace("https://", "https://user@"),
    `${details}21?attempt=1`,
    `${details}21#fragment`,
    `${details}021`,
    `${details}21/extra`,
  ]) {
    const entry = { ...check(1, 21), details_url };
    await assert.rejects(readCanonicalJobs({ ...options, ...provider([entry], []) }, run));
  }
});

await test("individual job responses must match the exact check, run, head and observer", async () => {
  const entry = check(1, 21);
  for (const change of [
    { check_run_url: `${api}/check-runs/2` },
    { run_id: 13 },
    { head_sha: "b".repeat(40) },
    { name: "canonical observer (reach)" },
  ]) {
    const fixture = provider([entry], [{ ...job(entry), ...change }]);
    await assert.rejects(
      readCanonicalJobs({ ...options, ...fixture }, run),
      /Foreign canonical job/,
    );
  }
});

await test("a missing check suite cannot use another inventory endpoint", async () => {
  for (const check_suite_id of [0, -1, NaN]) {
    const fixture = provider([], []);
    await assert.rejects(readCanonicalJobs({ ...options, ...fixture }, { ...run, check_suite_id }));
    assert.equal(fixture.requested.length, 0);
  }
});
