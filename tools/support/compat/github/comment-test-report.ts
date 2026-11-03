/**
 * Writes the Check run's test report to the step summary and, for a
 * same-repository pull request, to one PR comment that later runs update in place.
 * The gate step decides the required check; this only reports its outcome.
 */
import { appendFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { asArray, asNumber, asRecord, asString, parseJson } from "../../json-value.ts";

export const TEST_REPORT_MARKER = "<!-- vize-test-report -->";
const BOT = "github-actions[bot]";
const REPORT_JOB = "test-report";
// GitHub rejects comment bodies over 65,536 characters.
const MAX_COMMENT_LENGTH = 65_000;
const STATE =
  /^<!-- vize-test-report-run sha=([0-9a-f]{40,64}) run=([1-9]\d*) attempt=([1-9]\d*) -->$/;
const FAILED = new Set(["failure", "timed_out", "startup_failure", "action_required"]);

export type Gate = "success" | "failure";
export interface RunIdentity {
  readonly headSha: string;
  readonly runId: number;
  readonly runAttempt: number;
}
export interface WorkflowStep {
  readonly name: string;
  readonly number: number;
  readonly conclusion: string | null;
}
export interface WorkflowJob {
  readonly name: string;
  readonly status: string;
  readonly conclusion: string | null;
  readonly url: string | null;
  readonly steps: readonly WorkflowStep[];
}
export interface IssueComment {
  readonly id: number;
  readonly login: string | null;
  readonly body: string;
}
export interface ReportInput {
  readonly jobs: readonly WorkflowJob[];
  readonly gate: Gate;
  readonly run: RunIdentity;
  readonly serverUrl: string;
  readonly repository: string;
  readonly workflow: string;
}
export interface ReportEnv extends Omit<ReportInput, "jobs"> {
  readonly apiUrl: string;
  readonly token: string;
  readonly summaryPath: string;
  /** Null when the token cannot write: fork pull requests and merge groups. */
  readonly pullRequest: number | null;
}

export function isManagedComment(comment: IssueComment): boolean {
  return (
    comment.login === BOT &&
    (comment.body === TEST_REPORT_MARKER || comment.body.startsWith(`${TEST_REPORT_MARKER}\n`))
  );
}

function stateLine(run: RunIdentity): string {
  return `<!-- vize-test-report-run sha=${run.headSha} run=${run.runId} attempt=${run.runAttempt} -->`;
}

/** The run that wrote a managed comment, from the line after the marker. */
export function commentRun(body: string): RunIdentity | null {
  const match = STATE.exec(body.split("\n", 2)[1] ?? "");
  return match
    ? { headSha: match[1], runId: Number(match[2]), runAttempt: Number(match[3]) }
    : null;
}

/** Only a run for the PR's current head writes, and never over a later run or attempt. */
export function mayReplace(
  run: RunIdentity,
  prHeadSha: string,
  existing: RunIdentity | null,
): boolean {
  if (run.headSha !== prHeadSha) return false;
  if (existing === null || existing.headSha !== run.headSha) return true;
  return (
    existing.runId < run.runId ||
    (existing.runId === run.runId && existing.runAttempt <= run.runAttempt)
  );
}

type Bucket = "passed" | "failed" | "skipped" | "other";
const BUCKETS: readonly Bucket[] = ["passed", "failed", "skipped", "other"];

function bucket(job: WorkflowJob): Bucket {
  if (job.conclusion === "success") return "passed";
  if (job.conclusion === "skipped") return "skipped";
  return job.conclusion !== null && FAILED.has(job.conclusion) ? "failed" : "other";
}

function escape(text: string): string {
  return text
    .replace(/[\\[\]|*_`]/g, "\\$&")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/\r?\n/g, " ");
}

function link(text: string, url: string | null): string {
  return url === null ? escape(text) : `[${escape(text)}](${url})`;
}

// Matrix legs share a suite: "Rust tests (Compiler)" belongs to "Rust tests".
function suiteName(job: string): string {
  return job.replace(/ \([^()]*\)$/, "");
}

function runUrl({ serverUrl, repository, run }: Omit<ReportInput, "jobs">): string {
  return `${serverUrl}/${repository}/actions/runs/${run.runId}/attempts/${run.runAttempt}`;
}

export function renderTestReport(input: ReportInput): string {
  const { run, serverUrl, repository } = input;
  const reportJob = input.jobs.find((job) => job.name === REPORT_JOB);
  if (!reportJob) throw new Error(`The run has no ${REPORT_JOB} job`);
  const jobs = input.jobs.filter((job) => job !== reportJob);
  const repo = `${serverUrl}/${repository}`;
  const totals = { passed: 0, failed: 0, skipped: 0, other: 0 };
  const suites = new Map<string, Record<Bucket, number>>();
  for (const job of jobs) {
    const name = suiteName(job.name);
    const row = suites.get(name) ?? { passed: 0, failed: 0, skipped: 0, other: 0 };
    row[bucket(job)] += 1;
    totals[bucket(job)] += 1;
    suites.set(name, row);
  }
  const ran = ([, row]: [string, Record<Bucket, number>]) =>
    row.passed + row.failed + row.other > 0;
  const rank = (row: Record<Bucket, number>) => (row.failed ? 0 : row.other ? 1 : 2);
  const rows = [...suites]
    .filter(ran)
    .sort(([a, left], [b, right]) => rank(left) - rank(right) || a.localeCompare(b));
  const notRun = [...suites]
    .filter((suite) => !ran(suite))
    .map(([name]) => escape(name))
    .sort((a, b) => a.localeCompare(b));
  const problems = jobs
    .filter((job) => bucket(job) === "failed" || bucket(job) === "other")
    .map((job) => {
      const steps = job.steps
        .filter((step) => step.conclusion !== null && FAILED.has(step.conclusion))
        .map((step) => `, step ${link(step.name, job.url && `${job.url}#step:${step.number}:1`)}`);
      return `- ${job.conclusion ?? job.status}: ${link(job.name, job.url)}${steps.join("")}`;
    });
  const lines = [
    `## Test report: ${input.gate === "success" ? "passed" : "failed"}`,
    "",
    `Head [\`${run.headSha.slice(0, 12)}\`](${repo}/commit/${run.headSha}) · ` +
      `[${escape(input.workflow)} run ${run.runId}, attempt ${run.runAttempt}]` +
      `(${runUrl(input)})`,
    `Gate: ${link(REPORT_JOB, reportJob.url)}`,
    "",
    `${jobs.length} jobs: ${BUCKETS.map((key) => `${totals[key]} ${key}`).join(", ")}`,
    "",
    "| Suite | Passed | Failed | Skipped | Other |",
    "| --- | ---: | ---: | ---: | ---: |",
    ...rows.map(
      ([name, row]) => `| ${escape(name)} | ${BUCKETS.map((key) => row[key]).join(" | ")} |`,
    ),
  ];
  if (notRun.length) lines.push("", `Not run: ${notRun.join(", ")}`);
  if (problems.length) lines.push("", "### Failed and unfinished jobs", "", ...problems);
  return lines.join("\n");
}

function commentBody(run: RunIdentity, report: string, runUrl: string): string {
  const body = `${TEST_REPORT_MARKER}\n${stateLine(run)}\n${report}`;
  if (body.length <= MAX_COMMENT_LENGTH) return body;
  const note = `\n\nTruncated: the [run summary](${runUrl}) holds the whole report.`;
  const cut = body.lastIndexOf("\n", MAX_COMMENT_LENGTH - note.length);
  return `${body.slice(0, cut)}${note}`;
}

function required(env: Readonly<Record<string, string | undefined>>, name: string): string {
  const value = env[name];
  if (!value) throw new Error(`${name} is required`);
  return value;
}

function positive(env: Readonly<Record<string, string | undefined>>, name: string): number {
  const value = required(env, name);
  if (!/^[1-9]\d*$/.test(value)) throw new Error(`${name} must be a positive integer`);
  return Number(value);
}

export function readEnv(env: Readonly<Record<string, string | undefined>>): ReportEnv {
  const gate = required(env, "TEST_REPORT_GATE");
  if (gate !== "success" && gate !== "failure")
    throw new Error(`TEST_REPORT_GATE must be success or failure, not ${gate}`);
  const headSha = required(env, "TEST_REPORT_HEAD_SHA");
  if (!/^[0-9a-f]{40,64}$/.test(headSha)) throw new Error("TEST_REPORT_HEAD_SHA must be a commit");
  const pullRequest = env.TEST_REPORT_PR_NUMBER ?? "";
  if (pullRequest !== "" && !/^[1-9]\d*$/.test(pullRequest))
    throw new Error("TEST_REPORT_PR_NUMBER must be empty or a pull request number");
  return {
    gate,
    run: {
      headSha,
      runId: positive(env, "GITHUB_RUN_ID"),
      runAttempt: positive(env, "GITHUB_RUN_ATTEMPT"),
    },
    serverUrl: required(env, "GITHUB_SERVER_URL"),
    repository: required(env, "GITHUB_REPOSITORY"),
    workflow: required(env, "GITHUB_WORKFLOW"),
    apiUrl: required(env, "GITHUB_API_URL"),
    token: required(env, "GITHUB_TOKEN"),
    summaryPath: required(env, "GITHUB_STEP_SUMMARY"),
    pullRequest: pullRequest === "" ? null : Number(pullRequest),
  };
}

function nullableString(value: unknown, label: string): string | null {
  return value === null ? null : asString(value, label);
}

function parseJob(value: unknown): WorkflowJob {
  const job = asRecord(value, "job");
  return {
    name: asString(job.name, "job.name"),
    status: asString(job.status, "job.status"),
    conclusion: nullableString(job.conclusion, "job.conclusion"),
    url: nullableString(job.html_url, "job.html_url"),
    steps: asArray(job.steps ?? [], "job.steps").map((entry) => {
      const step = asRecord(entry, "step");
      return {
        name: asString(step.name, "step.name"),
        number: asNumber(step.number, "step.number"),
        conclusion: nullableString(step.conclusion, "step.conclusion"),
      };
    }),
  };
}

function parseComment(value: unknown): IssueComment {
  const comment = asRecord(value, "comment");
  const user = comment.user === null ? null : asRecord(comment.user, "comment.user");
  return {
    id: asNumber(comment.id, "comment.id"),
    login: user === null ? null : asString(user.login, "comment.user.login"),
    body: nullableString(comment.body ?? null, "comment.body") ?? "",
  };
}

function github(env: ReportEnv, fetchImpl: typeof fetch) {
  async function request(method: string, target: string, body?: unknown) {
    // Link headers are absolute; the token never leaves the API host.
    const url = target.startsWith("/") ? `${env.apiUrl}${target}` : target;
    if (!url.startsWith(`${env.apiUrl}/`)) throw new Error(`Refusing ${url} outside ${env.apiUrl}`);
    const response = await fetchImpl(url, {
      method,
      headers: {
        Accept: "application/vnd.github+json",
        Authorization: `Bearer ${env.token}`,
        "Content-Type": "application/json",
        "X-GitHub-Api-Version": "2022-11-28",
      },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    const text = await response.text();
    if (!response.ok) throw new Error(`GitHub API ${method} ${url}: ${response.status} ${text}`);
    const next = /<([^>]+)>;\s*rel="next"/.exec(response.headers.get("link") ?? "")?.[1] ?? null;
    return { data: parseJson(text), next };
  }
  async function pages(target: string, items: (data: unknown) => unknown[]) {
    const all: unknown[] = [];
    for (let next: string | null = target; next !== null;) {
      const page = await request("GET", next);
      all.push(...items(page.data));
      next = page.next;
    }
    return all;
  }
  return { request, pages };
}

/** Returns what it did with the PR comment; the step summary is always written. */
export async function publishTestReport(env: ReportEnv, fetchImpl: typeof fetch): Promise<string> {
  const api = github(env, fetchImpl);
  const repo = `/repos/${env.repository}`;
  const jobs = await api.pages(
    `${repo}/actions/runs/${env.run.runId}/jobs?filter=latest&per_page=100`,
    (data) => asArray(asRecord(data, "jobs page").jobs, "jobs"),
  );
  const report = renderTestReport({ ...env, jobs: jobs.map(parseJob) });
  appendFileSync(env.summaryPath, `${report}\n`);
  if (env.pullRequest === null)
    return "No PR comment: fork pull requests and merge groups get only the step summary";

  const comments = await api.pages(
    `${repo}/issues/${env.pullRequest}/comments?per_page=100`,
    (data) => asArray(data, "comments"),
  );
  const existing = comments.map(parseComment).find(isManagedComment);
  // Read last so the comparison sees the newest head.
  const pull = asRecord(
    (await api.request("GET", `${repo}/pulls/${env.pullRequest}`)).data,
    "pull",
  );
  const prHead = asString(asRecord(pull.head, "pull.head").sha, "pull.head.sha");
  const written = existing ? commentRun(existing.body) : null;
  if (!mayReplace(env.run, prHead, written)) {
    const kept = existing ? `test report comment ${existing.id}` : "no test report comment";
    const by = written ? `, comment from run ${written.runId} attempt ${written.runAttempt}` : "";
    return `Kept ${kept}: superseded (PR head ${prHead}${by})`;
  }
  const body = commentBody(env.run, report, runUrl(env));
  if (existing) {
    await api.request("PATCH", `${repo}/issues/comments/${existing.id}`, { body });
    return `Updated test report comment ${existing.id}`;
  }
  const created = asRecord(
    (await api.request("POST", `${repo}/issues/${env.pullRequest}/comments`, { body })).data,
    "comment",
  );
  return `Created test report comment ${asNumber(created.id, "comment.id")}`;
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) console.log(await publishTestReport(readEnv(process.env), fetch));
