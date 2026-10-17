import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  bootstrapRequiredWorkflowRuns,
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
import { githubApiPages, githubApiRequest } from "../../compat/github/release-preflight-github.mjs";

type Row = Record<string, unknown>;
type CheckedRun = Row & {
  id: number;
  run_attempt: number;
  status: string;
  conclusion: string | null;
};
function assertRun(run: Row): asserts run is CheckedRun {
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
export type Api = {
  read: (resource: string) => Promise<Row>;
  pages: (resource: string, collection: string, query?: Record<string, string>) => Promise<Row[]>;
  dispatch: (resource: string, body: unknown) => Promise<void>;
};
type Options = {
  api: Api;
  deadline: number;
  now?: () => number;
  sleep?: (milliseconds: number) => Promise<void>;
  log?: (message: string) => void;
};

export function operatorDeadline(environment: NodeJS.ProcessEnv, now = Date.now()) {
  const seconds = environment.VIZE_RELEASE_OPERATOR_TIMEOUT_SECONDS ?? "28800";
  assert.match(seconds, /^[1-9][0-9]*$/u, "valid operator timeout required");
  assert.ok(Number(seconds) <= 28_800, "operator timeout exceeds existing eight-hour budget");
  let deadline = now + Number(seconds) * 1000;
  const inherited = environment.VIZE_RELEASE_OPERATOR_DEADLINE_UNIX_MILLIS;
  if (inherited !== undefined) {
    assert.match(inherited, /^[1-9][0-9]*$/u, "valid inherited operator deadline required");
    assert.ok(Number.isSafeInteger(Number(inherited)), "exact inherited deadline required");
    deadline = Math.min(deadline, Number(inherited));
  }
  assert.ok(deadline > now, "operator deadline expired before Check qualification");
  return deadline;
}

/** Source qualification finishes before a new publishing run is allowed to exist. */
export async function qualifyCheck(candidate: Candidate, options: Options) {
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
  const branchStillFrozen = async () => {
    remaining();
    const ref = await options.api.read(`git/ref/heads/${candidate.branch}`);
    assert.equal(ref.ref, `refs/heads/${candidate.branch}`);
    const object = ref.object as Row;
    assert.equal(object?.type, "commit");
    assert.equal(
      object?.sha,
      candidate.head,
      "frozen release branch moved during Check qualification",
    );
    remaining();
  };
  const plans = createReleaseGateDispatchPlans({
    ref: candidate.branch,
    headSha: candidate.head,
    baseSha: candidate.cut,
  });
  const qualifiers = releaseGateRunQualifiers(plans);
  let previous = "";
  const listRuns = async () => {
    await branchStillFrozen();
    const runs = await options.api.pages("actions/runs", "workflow_runs", {
      head_sha: candidate.head,
    });
    remaining();
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
  const jobsFor = async (run: Row) => {
    assertRun(run);
    remaining();
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
    remaining();
    return jobs;
  };
  const selected = await bootstrapRequiredWorkflowRuns({
    sha: candidate.head,
    dispatchPlans: plans,
    listRuns,
    dispatchWorkflow: async (plan: { workflowId: string; ref: string; inputs: unknown }) => {
      await branchStillFrozen();
      log(
        `Dispatching full Check for frozen ${candidate.head} on ${candidate.branch}; publishing has not started.`,
      );
      await options.api.dispatch(`actions/workflows/${plan.workflowId}/dispatches`, {
        ref: plan.ref,
        inputs: plan.inputs,
      });
      remaining();
    },
    requiredJobFailure: async (_workflow: string, run: Row) => {
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
    },
    timeoutMs: remaining(),
    now,
    sleep,
    onWait: () => remaining(),
  });
  const run = selected.get("Check") as Row;
  assertRun(run);
  assertRequiredWorkflowJobs("Check", await jobsFor(run), run);
  // A newer failed/pending attempt must not be hidden by the run just inspected.
  const latest = selectRequiredWorkflowRuns(
    await listRuns(),
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
  await branchStillFrozen();
  log(
    `Qualified frozen ${candidate.head}: https://github.com/${candidate.repository}/actions/runs/${run.id}; the publishing preflight must recheck this live evidence.`,
  );
  return run;
}

async function main() {
  assert.equal(process.argv.length, 6, "Usage: main.ts <repository> <release/vTAG> <H> <C>");
  const [repository, branch, head, cut] = process.argv.slice(2);
  const deadline = operatorDeadline(process.env);
  const token = process.env.GITHUB_TOKEN || process.env.GH_TOKEN;
  assert.ok(token, "authenticated GitHub token required");
  const apiUrl = process.env.GITHUB_API_URL ?? "https://api.github.com";
  const checked = () => {
    const milliseconds = deadline - Date.now();
    assert.ok(milliseconds > 0, "operator deadline expired during GitHub request");
    return Math.min(30_000, milliseconds);
  };
  const boundedSleep = async (milliseconds: number) => {
    checked();
    await new Promise((done) => setTimeout(done, Math.min(milliseconds, deadline - Date.now())));
    checked();
  };
  const base = { apiUrl, repository, token, sleep: boundedSleep };
  const boundedFetch: typeof fetch = (url, init) =>
    fetch(url, {
      ...init,
      signal: AbortSignal.any([
        ...(init?.signal ? [init.signal] : []),
        AbortSignal.timeout(checked()),
      ]),
    });
  const api: Api = {
    read: async (resource) =>
      JSON.parse((await githubApiRequest({ ...base, resource, fetchImpl: boundedFetch })).body),
    pages: (resource, collection, query = {}) =>
      githubApiPages({ ...base, resource, collection, query, fetchImpl: boundedFetch }),
    dispatch: async (resource, body) => {
      await githubApiRequest({ ...base, resource, method: "POST", body, fetchImpl: boundedFetch });
    },
  };
  await qualifyCheck({ repository, branch, head, cut }, { api, deadline });
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}
