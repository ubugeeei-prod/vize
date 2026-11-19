// Keeps one tracking issue open while the full Check on the default branch is
// red. The daily `schedule` run and manual `Check full` dispatches are the only
// runs that execute the full suite, and no pull request waits on them, so a
// failure there otherwise reaches nobody: GitHub mails only the last editor of
// the cron line. The issue notifies every repository watcher, collects one
// comment per further red run, and closes itself on the next green run.
import fs from "node:fs";
import { pathToFileURL } from "node:url";

export type Api = (
  endpoint: string,
  options?: { method?: string; body?: unknown },
) => Promise<unknown>;

export type CompletedRun = {
  id: number;
  event: string;
  conclusion: string | null;
  head_branch: string;
  head_sha: string;
  html_url: string;
  created_at: string;
};

type Job = { name: string; conclusion: string | null; html_url: string };
type Issue = { number: number; body: string | null; pull_request?: unknown };

export const marker = "<!-- vize-full-check-alert -->";
export const issueTitle = "Full Check is red on main";
const fullEvents = new Set(["schedule", "workflow_dispatch"]);
const redConclusions = new Set(["failure", "timed_out", "startup_failure", "action_required"]);
const failedJobConclusions = new Set(["failure", "timed_out", "startup_failure"]);
// Aggregators only restate a producer's red result; listing them hides the cause.
const aggregator =
  /(?:^|\/ )(?:test-report|full-js-report|full-rust-report|clippy-and-test|test-js-packages|test-scripts)$/u;

export type Decision = "ignore" | "report-red" | "report-green";

export function decide(run: CompletedRun, defaultBranch: string): Decision {
  if (!fullEvents.has(run.event) || run.head_branch !== defaultBranch) return "ignore";
  if (run.conclusion === "success") return "report-green";
  if (run.conclusion != null && redConclusions.has(run.conclusion)) return "report-red";
  // Cancelled and skipped runs prove nothing either way.
  return "ignore";
}

export function failedProducers(jobs: readonly Job[]): Job[] {
  return jobs.filter(
    (job) =>
      job.conclusion != null &&
      failedJobConclusions.has(job.conclusion) &&
      !aggregator.test(job.name),
  );
}

function cell(text: string): string {
  return text.replaceAll("|", "\\|").replaceAll("\n", " ");
}

export function renderRed(run: CompletedRun, jobs: readonly Job[]): string {
  const failed = failedProducers(jobs);
  const lines = [
    `Full Check [run ${run.id}](${run.html_url}) (\`${run.event}\`, ${run.created_at}) at \`${run.head_sha.slice(0, 10)}\` concluded \`${run.conclusion}\`.`,
    "",
  ];
  if (failed.length === 0) {
    lines.push("No producer job failed; only an aggregate gate is red. Open the run for details.");
  } else {
    lines.push(
      `${failed.length} failing producer job(s):`,
      "",
      "| Job | Result |",
      "| --- | --- |",
    );
    for (const job of failed)
      lines.push(`| [${cell(job.name)}](${job.html_url}) | ${job.conclusion} |`);
  }
  lines.push(
    "",
    "The full suite runs only on the daily schedule and on manual dispatch, so pull requests do not see these failures. Triage each job (regression, stale pin, flaky environment or test bug) before re-pinning anything.",
  );
  return lines.join("\n");
}

export function renderGreen(run: CompletedRun): string {
  return `Full Check [run ${run.id}](${run.html_url}) (\`${run.event}\`) at \`${run.head_sha.slice(0, 10)}\` is green again. Closing.`;
}

async function listAll<T>(api: Api, endpoint: string, key?: string): Promise<T[]> {
  const items: T[] = [];
  for (let page = 1; ; page += 1) {
    const separator = endpoint.includes("?") ? "&" : "?";
    const response = await api(`${endpoint}${separator}per_page=100&page=${page}`);
    const batch = (key ? (response as Record<string, T[]>)[key] : response) as T[];
    items.push(...batch);
    if (batch.length < 100) return items;
  }
}

export async function findTrackingIssue(api: Api, repository: string): Promise<Issue | undefined> {
  const issues = await listAll<Issue>(
    api,
    `repos/${repository}/issues?state=open&creator=${encodeURIComponent("github-actions[bot]")}`,
  );
  return issues.find((issue) => issue.pull_request == null && issue.body?.includes(marker));
}

export type Outcome = { decision: Decision; action: string; issue?: number };

export async function reportFullCheck(
  api: Api,
  repository: string,
  run: CompletedRun,
  defaultBranch: string,
  assignees: readonly string[] = [],
): Promise<Outcome> {
  const decision = decide(run, defaultBranch);
  if (decision === "ignore") return { decision, action: "none" };
  const prefix = `repos/${repository}`;
  const open = await findTrackingIssue(api, repository);
  if (decision === "report-green") {
    if (!open) return { decision, action: "none" };
    await api(`${prefix}/issues/${open.number}/comments`, {
      method: "POST",
      body: { body: renderGreen(run) },
    });
    await api(`${prefix}/issues/${open.number}`, {
      method: "PATCH",
      body: { state: "closed", state_reason: "completed" },
    });
    return { decision, action: "closed", issue: open.number };
  }
  const jobs = await listAll<Job>(
    api,
    `${prefix}/actions/runs/${run.id}/jobs?filter=latest`,
    "jobs",
  );
  const report = renderRed(run, jobs);
  if (open) {
    await api(`${prefix}/issues/${open.number}/comments`, {
      method: "POST",
      body: { body: report },
    });
    return { decision, action: "commented", issue: open.number };
  }
  const created = (await api(`${prefix}/issues`, {
    method: "POST",
    body: {
      title: issueTitle,
      body: `${marker}\n${report}`,
      ...(assignees.length > 0 ? { assignees } : {}),
    },
  })) as { number: number };
  return { decision, action: "opened", issue: created.number };
}

function required(name: string): string {
  const value = process.env[name];
  if (!value) throw new Error(`${name} is required`);
  return value;
}

async function main() {
  const repository = required("GITHUB_REPOSITORY");
  if (!/^[\w.-]+\/[\w.-]+$/u.test(repository)) throw new Error("GITHUB_REPOSITORY is malformed");
  const token = required("GH_TOKEN");
  const event = JSON.parse(fs.readFileSync(required("GITHUB_EVENT_PATH"), "utf8")) as {
    workflow_run: CompletedRun;
    repository: { default_branch: string };
  };
  const assignees = (process.env.FULL_CHECK_ALERT_ASSIGNEES ?? "")
    .split(",")
    .map((login) => login.trim())
    .filter((login) => /^[\w-]+$/u.test(login));
  const api: Api = async (endpoint, options = {}) => {
    const response = await fetch(`https://api.github.com/${endpoint}`, {
      method: options.method ?? "GET",
      signal: AbortSignal.timeout(30_000),
      headers: {
        Authorization: `Bearer ${token}`,
        Accept: "application/vnd.github+json",
        "X-GitHub-Api-Version": "2022-11-28",
      },
      ...(options.body ? { body: JSON.stringify(options.body) } : {}),
    });
    if (!response.ok) throw new Error(`GitHub ${response.status}: ${endpoint}`);
    const body = await response.text();
    return body ? (JSON.parse(body) as unknown) : null;
  };
  const outcome = await reportFullCheck(
    api,
    repository,
    event.workflow_run,
    event.repository.default_branch,
    assignees,
  );
  const summary = `Full Check alert: ${outcome.decision}, ${outcome.action}${outcome.issue ? ` #${outcome.issue}` : ""}.`;
  console.log(summary);
  if (process.env.GITHUB_STEP_SUMMARY)
    fs.appendFileSync(process.env.GITHUB_STEP_SUMMARY, `${summary}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
