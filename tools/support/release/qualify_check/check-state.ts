import assert from "node:assert/strict";
import {
  createReleaseGateDispatchPlans,
  releaseGateRunQualifiers,
} from "../../compat/github/release-preflight-bootstrap.mjs";
import {
  assertRequiredWorkflowJobs,
  latestRequiredWorkflowRun,
  requiredWorkflowJobNames,
  selectRequiredWorkflowRuns,
  summarizeRequiredWorkflowJobFailures,
} from "../../compat/github/release-preflight-evidence.mjs";

export type Row = Record<string, unknown>;
export type CheckedRun = Row & {
  id: number;
  run_attempt: number;
  status: string;
  conclusion: string | null;
};
export function assertRun(run: Row): asserts run is CheckedRun {
  assert.ok(
    typeof run.id === "number" && Number.isSafeInteger(run.id) && run.id > 0,
    "positive Check run ID required",
  );
  assert.ok(
    typeof run.run_attempt === "number" &&
      Number.isSafeInteger(run.run_attempt) &&
      run.run_attempt > 0,
    "positive latest Check attempt required",
  );
  assert.equal(typeof run.status, "string", "Check run status required");
  assert.ok(
    run.conclusion === null || typeof run.conclusion === "string",
    "Check conclusion required",
  );
}
export type Candidate = { repository: string; branch: string; head: string; cut: string };
export type ReadApi = {
  read: (resource: string) => Promise<Row>;
  pages: (resource: string, collection: string, query?: Record<string, string>) => Promise<Row[]>;
};
export type ReadOptions = {
  api: ReadApi;
  deadline: number;
  now?: () => number;
  sleep?: (milliseconds: number) => Promise<void>;
  log?: (message: string) => void;
};

export type Api = ReadApi & { dispatch: (resource: string, body: unknown) => Promise<void> };
export type Options = ReadOptions & { api: Api };

/** Exact-source observation shared by the blocking operator and read-only release join. */
export function createCheckObserver(candidate: Candidate, options: ReadOptions) {
  assert.match(candidate.repository, /^[\w.-]+\/[\w.-]+$/u);
  assert.match(candidate.branch, /^release\/v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/u);
  for (const sha of [candidate.head, candidate.cut]) assert.match(sha, /^[0-9a-f]{40}$/u);
  assert.notEqual(candidate.head, candidate.cut);
  const now = options.now ?? Date.now;
  const log = options.log ?? console.log;
  const remaining = () => {
    const milliseconds = options.deadline - now();
    assert.ok(
      Number.isSafeInteger(milliseconds) && milliseconds > 0,
      "operator deadline expired during Check qualification",
    );
    return milliseconds;
  };
  const sleep = async (milliseconds: number) => {
    await (options.sleep ?? ((delay) => new Promise((done) => setTimeout(done, delay))))(
      Math.min(milliseconds, remaining()),
    );
    remaining();
  };
  const guard = (allowExpired: boolean) => {
    if (!allowExpired) remaining();
  };
  const branchStillFrozen = async (allowExpired = false) => {
    guard(allowExpired);
    const ref = await options.api.read(`git/ref/heads/${candidate.branch}`);
    assert.equal(ref.ref, `refs/heads/${candidate.branch}`);
    const object = ref.object as Row;
    assert.equal(object?.type, "commit");
    assert.equal(
      object?.sha,
      candidate.head,
      "frozen release branch moved during Check qualification",
    );
    guard(allowExpired);
  };
  const plans = createReleaseGateDispatchPlans({
    ref: candidate.branch,
    headSha: candidate.head,
    baseSha: candidate.cut,
  });
  const qualifiers = releaseGateRunQualifiers(plans);
  let previous = "";
  const listRuns = async (allowExpired = false) => {
    await branchStillFrozen(allowExpired);
    const runs = await options.api.pages("actions/runs", "workflow_runs", {
      head_sha: candidate.head,
    });
    guard(allowExpired);
    const current = latestRequiredWorkflowRun(
      runs,
      candidate.head,
      "Check",
      qualifiers.get("Check"),
    );
    if (current) {
      assertRun(current);
      const state = `${current.id}/${current.run_attempt}/${current.status}/${current.conclusion}`;
      if (state !== previous) {
        log(
          `Check qualification https://github.com/${candidate.repository}/actions/runs/${current.id} (${current.status}/${current.conclusion ?? "pending"})`,
        );
        previous = state;
      }
    }
    return runs;
  };
  const jobsFor = async (run: Row, allowExpired = false) => {
    assertRun(run);
    guard(allowExpired);
    const jobs = await options.api.pages(`actions/runs/${run.id}/jobs`, "jobs", {
      filter: "latest",
    });
    const names = new Set(requiredWorkflowJobNames("Check", run));
    const attempt = run.run_attempt;
    assert.ok(
      typeof attempt === "number" && Number.isSafeInteger(attempt) && attempt > 0,
      "positive latest Check attempt required",
    );
    for (const job of jobs.filter((value) => names.has(String(value.name)))) {
      assert.equal(job.run_id, run.id, "Check job belongs to another run");
      assert.ok(
        typeof job.run_attempt === "number" &&
          Number.isSafeInteger(job.run_attempt) &&
          job.run_attempt > 0 &&
          job.run_attempt <= attempt,
        "Check job belongs to an invalid or future attempt",
      );
      assert.equal(job.head_sha, candidate.head, "Check job belongs to another source");
    }
    guard(allowExpired);
    return jobs;
  };
  const latest = (runs: Row[]): CheckedRun | undefined => {
    const run = latestRequiredWorkflowRun(runs, candidate.head, "Check", qualifiers.get("Check"));
    if (run) assertRun(run);
    return run;
  };
  const requiredFailure = async (run: Row) => {
    const jobs = await jobsFor(run);
    const required = new Set(requiredWorkflowJobNames("Check", run));
    const selectedJobs = jobs.filter((job) => required.has(String(job.name)));
    const duplicate = selectedJobs.some(
      (job, index) => selectedJobs.findIndex((other) => other.name === job.name) !== index,
    );
    return duplicate ||
      selectedJobs.some(
        (job) =>
          job.run_attempt === run.run_attempt &&
          job.status === "completed" &&
          job.conclusion !== "success",
      )
      ? summarizeRequiredWorkflowJobFailures("Check", jobs, run)
      : undefined;
  };
  const finish = async (run: CheckedRun, allowExpired = false) => {
    assertRequiredWorkflowJobs("Check", await jobsFor(run, allowExpired), run);
    // A newer failed/pending attempt must not be hidden by the run just inspected.
    const latest = selectRequiredWorkflowRuns(
      await listRuns(allowExpired),
      candidate.head,
      undefined,
      qualifiers,
    ).get("Check") as Row;
    for (const key of [
      "id",
      "run_attempt",
      "head_sha",
      "head_branch",
      "event",
      "path",
      "display_title",
      "status",
      "conclusion",
    ])
      assert.equal(latest[key], run[key], "latest Check changed during candidate qualification");
    await branchStillFrozen(allowExpired);
    return run;
  };
  return {
    now,
    log,
    remaining,
    sleep,
    branchStillFrozen,
    plans,
    qualifiers,
    listRuns,
    jobsFor,
    latest,
    requiredFailure,
    finish,
  };
}
