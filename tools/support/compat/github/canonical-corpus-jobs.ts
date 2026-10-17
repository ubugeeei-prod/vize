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

// The run-wide jobs endpoint can fail while Checks and individual Actions jobs
// remain available. Checks supplies only the inventory; full Actions job records
// still supply every attempt, step, execution window and runner identity.
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
  const workers = checks
    .filter((check) => /(?:^| \/ )canonical observer \(([^)]+)\)$/.test(check.name ?? ""))
    .map((check) => {
      const checkId = check.id;
      if (typeof checkId !== "number" || !Number.isSafeInteger(checkId) || checkId <= 0) {
        throw new Error("Invalid canonical check ID");
      }
      assert(!ids.has(checkId), "Duplicate canonical check ID");
      ids.add(checkId);
      assert.equal(check.check_suite?.id, run.check_suite_id, "Foreign canonical check suite");
      assert.equal(check.head_sha, run.head_sha, "Foreign canonical check head");
      const checkUrl = `${options.apiUrl}/repos/${options.repository}/check-runs/${checkId}`;
      assert.equal(check.url, checkUrl, "Foreign canonical check URL");
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
      const response = await githubApiRequest({
        ...options,
        resource: `actions/jobs/${jobId}`,
      });
      const job = JSON.parse(response.body);
      assert.equal(job.id, jobId, "Foreign canonical job ID");
      assert.equal(job.check_run_url, checkUrl, "Foreign canonical job check");
      assert.equal(job.run_id, run.id, "Foreign canonical job run");
      assert.equal(job.head_sha, run.head_sha, "Foreign canonical job head");
      assert.equal(job.name, check.name, "Foreign canonical job name");
      return job;
    }),
  );
}
