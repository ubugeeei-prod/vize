import assert from "node:assert/strict";
import { githubApiPages, githubApiRequest } from "./release-preflight-github.mjs";

type Options = {
  apiUrl: string;
  serverUrl: string;
  repository: string;
  token: string;
  fetchImpl?: typeof fetch;
  sleep?: (milliseconds: number) => Promise<void>;
};

type Run = { id: number; head_sha: string; check_suite_id: number };

const visibilityAttempts = 10;
const visibilityDelayMs = 1_000;
const canonicalSteps = [
  "Plan canonical corpus identity",
  "Select and hydrate full fixture corpus",
  "Capture canonical corpus identity",
  "Run L2 DOM differential corpus",
  "Run L4 SSR and pug L1 differential corpora",
  "Measure Davinci production reach on the hydrated corpus",
  "Record required observer evidence",
  "Upload required observer evidence",
] as const;

type Job = {
  id: number;
  check_run_url: string;
  run_id: number;
  run_attempt: number;
  head_sha: string;
  name: string;
  status: string;
  steps?: readonly { name?: string }[];
  [key: string]: unknown;
};

function hasCanonicalStepInventory(job: Job): boolean {
  if (!Array.isArray(job.steps)) return false;
  const names = new Set(job.steps.map((step) => step.name));
  return canonicalSteps.every((name) => names.has(name));
}

// Checks supplies the inventory. Individual Actions job records supply the exact
// attempt and runner identity, but can return a completed job with a truncated
// step list; the exact-attempt endpoint supplies the complete execution record.
export async function readCanonicalJobs(options: Options, run: Run) {
  assert(Number.isSafeInteger(run.id) && run.id > 0, "Invalid canonical run ID");
  assert(
    Number.isSafeInteger(run.check_suite_id) && run.check_suite_id > 0,
    "Missing canonical check suite",
  );
  const checks = await githubApiPages({
    ...options,
    resource: `check-suites/${run.check_suite_id}/check-runs`,
    collection: "check_runs",
    query: { filter: "all" },
  });
  const ids = new Set<number>();
  const attemptInventories = new Map<number, Promise<unknown[]>>();
  const attemptInventory = (attempt: number) => {
    let pending = attemptInventories.get(attempt);
    if (pending == null) {
      pending = githubApiPages({
        ...options,
        resource: `actions/runs/${run.id}/attempts/${attempt}/jobs`,
        collection: "jobs",
        query: { filter: "all" },
      });
      attemptInventories.set(attempt, pending);
    }
    return pending;
  };
  const workers = checks
    .filter(
      (check) =>
        typeof check.name === "string" &&
        /(?:^| \/ )canonical observer \(([^)]+)\)$/.test(check.name),
    )
    .map((check) => {
      const checkId = check.id;
      if (typeof checkId !== "number" || !Number.isSafeInteger(checkId) || checkId <= 0) {
        throw new Error("Invalid canonical check ID");
      }
      assert(!ids.has(checkId), "Duplicate canonical check ID");
      ids.add(checkId);
      const suite = check.check_suite;
      assert.ok(
        suite !== null && typeof suite === "object" && "id" in suite,
        "Foreign canonical check suite",
      );
      assert.equal(suite.id, run.check_suite_id, "Foreign canonical check suite");
      assert.equal(check.head_sha, run.head_sha, "Foreign canonical check head");
      const checkUrl = `${options.apiUrl}/repos/${options.repository}/check-runs/${checkId}`;
      assert.equal(check.url, checkUrl, "Foreign canonical check URL");
      assert.ok(typeof check.details_url === "string", "Foreign canonical job URL");
      const details = new URL(check.details_url);
      const prefix = `/${options.repository}/actions/runs/${run.id}/job/`;
      assert.equal(details.origin, new URL(options.serverUrl).origin, "Foreign canonical job host");
      assert(
        !details.username &&
          !details.password &&
          !details.search &&
          !details.hash &&
          details.pathname.startsWith(prefix),
        "Foreign canonical job URL",
      );
      const suffix = details.pathname.slice(prefix.length);
      const jobId = Number(suffix);
      assert(
        /^[1-9][0-9]*$/.test(suffix) && Number.isSafeInteger(jobId),
        "Invalid canonical job ID",
      );
      return { check, checkUrl, jobId };
    });
  return Promise.all(
    workers.map(async ({ check, checkUrl, jobId }) => {
      const sleep =
        options.sleep ??
        ((milliseconds) =>
          new Promise((resolve) => {
            setTimeout(resolve, milliseconds);
          }));
      let job;
      for (let attempt = 0; attempt < visibilityAttempts; attempt += 1) {
        const response = await githubApiRequest({
          ...options,
          resource: `actions/jobs/${jobId}`,
        });
        job = JSON.parse(response.body) as Job;
        assert.equal(job.id, jobId, "Foreign canonical job ID");
        assert.equal(job.check_run_url, checkUrl, "Foreign canonical job check");
        assert.equal(job.run_id, run.id, "Foreign canonical job run");
        assert(
          Number.isSafeInteger(job.run_attempt) && job.run_attempt > 0,
          "Foreign canonical job attempt",
        );
        assert.equal(job.head_sha, run.head_sha, "Foreign canonical job head");
        assert.equal(job.name, check.name, "Foreign canonical job name");
        if (job.status === "completed") {
          if (hasCanonicalStepInventory(job)) return { ...job, canonicalCheck: check };
          const inventory = (await attemptInventory(job.run_attempt)) as Job[];
          const matches = inventory.filter((candidate) => candidate.id === jobId);
          assert.equal(matches.length, 1, "Missing canonical attempt job");
          const attemptJob = matches[0];
          assert.equal(attemptJob.check_run_url, checkUrl, "Foreign canonical attempt check");
          assert.equal(attemptJob.run_id, run.id, "Foreign canonical attempt run");
          assert.equal(attemptJob.run_attempt, job.run_attempt, "Foreign canonical attempt number");
          assert.equal(attemptJob.head_sha, run.head_sha, "Foreign canonical attempt head");
          assert.equal(attemptJob.name, check.name, "Foreign canonical attempt name");
          job = attemptJob;
          if (hasCanonicalStepInventory(job)) return { ...job, canonicalCheck: check };
          attemptInventories.delete(job.run_attempt);
        }
        if (attempt + 1 < visibilityAttempts) await sleep(visibilityDelayMs);
      }
      assert(job, "Missing canonical job");
      return { ...job, canonicalCheck: check };
    }),
  );
}
