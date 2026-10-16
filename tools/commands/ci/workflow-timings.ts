/**
 * Read Actions wall time and runner time without downloading logs or artifacts.
 * node tools/commands/ci/workflow-timings.ts --attempt 1 38805236717 38805184162
 * node tools/commands/ci/workflow-timings.ts --output /tmp/ci-timings.json RUN_ID
 *
 * Retry endpoints repeat successful jobs from earlier attempts with a new
 * run_attempt field. Their original timestamps identify them as reused jobs;
 * they are not charged again to the requested attempt's runner time.
 */
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

type Step = {
  name: string;
  conclusion: string | null;
  started_at: string | null;
  completed_at: string | null;
};
type Job = Step & {
  id: number;
  status: string;
  html_url: string;
  steps: Step[];
};
type Run = {
  id: number;
  name: string;
  html_url: string;
  event: string;
  head_sha: string;
  status: string;
  conclusion: string | null;
  run_attempt: number;
  created_at: string;
  run_started_at: string;
  updated_at: string;
};

const timestamp = (value: string | null) => (value ? Date.parse(value) : NaN);
const seconds = (from: number, to: number) => Math.round(Math.max(0, to - from) / 1000);

export function summarizeWorkflowTiming(run: Run, jobs: readonly Job[], now = Date.now(), top = 3) {
  const created = timestamp(run.created_at);
  const started = timestamp(run.run_started_at);
  if (!Number.isFinite(created) || !Number.isFinite(started)) {
    throw new Error(`Run ${run.id} has no valid creation or attempt-start timestamp`);
  }
  const executed = jobs.filter(
    (job) => job.conclusion !== "skipped" && Number.isFinite(timestamp(job.started_at)),
  );
  const reused = executed.filter(
    (job) => timestamp(job.started_at) < started && timestamp(job.completed_at) <= started,
  );
  const reusedIds = new Set(reused.map((job) => job.id));
  const current = executed.filter((job) => !reusedIds.has(job.id));
  const lastJob = Math.max(
    ...executed.map((job) => timestamp(job.completed_at)).filter(Number.isFinite),
  );
  const completed =
    run.status === "completed"
      ? Math.max(started, Number.isFinite(lastJob) ? lastJob : timestamp(run.updated_at))
      : now;
  const jobSeconds = (job: Job) =>
    seconds(
      timestamp(job.started_at),
      Number.isFinite(timestamp(job.completed_at)) ? timestamp(job.completed_at) : completed,
    );
  const detail = (job: Job) => ({
    id: job.id,
    name: job.name,
    conclusion: job.conclusion,
    url: job.html_url,
    startedAt: job.started_at,
    completedAt: job.completed_at,
    wallSeconds: jobSeconds(job),
    startOffsetSeconds: seconds(started, timestamp(job.started_at)),
    completionOffsetSeconds: seconds(started, timestamp(job.completed_at) || completed),
    longestSteps: job.steps
      .filter(
        (step) => step.conclusion !== "skipped" && Number.isFinite(timestamp(step.started_at)),
      )
      .map((step) => ({
        name: step.name,
        conclusion: step.conclusion,
        wallSeconds: seconds(timestamp(step.started_at), timestamp(step.completed_at) || completed),
      }))
      .sort((a, b) => b.wallSeconds - a.wallSeconds)
      .slice(0, 3),
  });
  const end = (job: Job) => timestamp(job.completed_at) || completed;
  return {
    runId: run.id,
    workflow: run.name,
    event: run.event,
    sha: run.head_sha,
    attempt: run.run_attempt,
    status: run.status,
    conclusion: run.conclusion,
    url: `${run.html_url}/attempts/${run.run_attempt}`,
    createdAt: run.created_at,
    attemptStartedAt: run.run_started_at,
    observedThrough: new Date(completed).toISOString(),
    // Overall wall time includes human/retry delays. Compare attempt 1 for a
    // clean before/after baseline, and keep later attempt time separate.
    endToEndSeconds: seconds(created, completed),
    attemptSeconds: seconds(started, completed),
    runnerMinutes: Math.round(current.reduce((sum, job) => sum + jobSeconds(job), 0) / 6) / 10,
    executedJobs: current.length,
    reusedJobs: reused.length,
    // This is the observed terminal tail, not an inferred dependency graph.
    latestCompletingJobs: [...current]
      .sort((a, b) => end(b) - end(a))
      .slice(0, top)
      .map(detail),
    longestJobs: [...current]
      .sort((a, b) => jobSeconds(b) - jobSeconds(a))
      .slice(0, top)
      .map(detail),
  };
}

function github<T>(endpoint: string, paginate = false): T {
  const args = ["api", endpoint, ...(paginate ? ["--paginate", "--slurp"] : [])];
  return JSON.parse(
    execFileSync("gh", args, { encoding: "utf8", maxBuffer: 32 * 1024 * 1024 }),
  ) as T;
}

function main(args: string[]) {
  let repo = "ubugeeei-prod/vize";
  let attempt = "latest";
  let output: string | undefined;
  let top = 3;
  const runIds: string[] = [];
  for (let index = 0; index < args.length; index++) {
    const argument = args[index];
    if (argument === "--repo") repo = args[++index] ?? "";
    else if (argument === "--attempt") attempt = args[++index] ?? "";
    else if (argument === "--output") output = args[++index] ?? "";
    else if (argument === "--top") top = Number(args[++index]);
    else if (argument === "--help" || argument === "-h") {
      console.log(
        "Usage: node tools/commands/ci/workflow-timings.ts [--repo OWNER/REPO] [--attempt latest|N] [--top 3] [--output FILE] RUN_ID...",
      );
      return;
    } else if (/^[1-9][0-9]*$/.test(argument)) runIds.push(argument);
    else throw new Error(`Unknown argument: ${argument}`);
  }
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repo)) throw new Error("Expected OWNER/REPO");
  if (attempt !== "latest" && !/^[1-9][0-9]*$/.test(attempt))
    throw new Error("Expected attempt latest or a positive number");
  if (!Number.isInteger(top) || top < 1 || top > 20)
    throw new Error("Expected --top between 1 and 20");
  if (output === "") throw new Error("Expected --output file path");
  if (!runIds.length) throw new Error("Supply at least one workflow run ID");
  const summaries = runIds.map((id) => {
    const base = `repos/${repo}/actions/runs/${id}`;
    const run = github<Run>(attempt === "latest" ? base : `${base}/attempts/${attempt}`);
    const pages = github<Array<{ jobs: Job[] }>>(
      `${base}/attempts/${run.run_attempt}/jobs?per_page=100`,
      true,
    );
    return summarizeWorkflowTiming(
      run,
      pages.flatMap((page) => page.jobs),
      Date.now(),
      top,
    );
  });
  if (output) {
    writeFileSync(output, `${JSON.stringify(summaries, null, 2)}\n`);
    console.log(`Saved ${summaries.length} workflow timing summaries to ${output}`);
  } else {
    for (const summary of summaries) console.log(JSON.stringify(summary));
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  try {
    main(process.argv.slice(2));
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
