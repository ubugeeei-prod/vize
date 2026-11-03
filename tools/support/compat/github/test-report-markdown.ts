/**
 * The Check test report as markdown: test cases by product area first, then the
 * jobs that ran them, collapsed. A pure function of the run's jobs and case packets.
 */
import { areaFor, areaRank } from "./test-areas.ts";
import type { Outcome, TestCasePacket } from "./test-cases.ts";

export const REPORT_JOB = "test-report";
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
export interface ReportInput {
  readonly jobs: readonly WorkflowJob[];
  /** Null when the case packets could not be downloaded. */
  readonly packets: readonly TestCasePacket[] | null;
  readonly gate: Gate;
  readonly run: RunIdentity;
  readonly serverUrl: string;
  readonly repository: string;
  readonly workflow: string;
}

export function runUrl({ serverUrl, repository, run }: Omit<ReportInput, "jobs" | "packets">) {
  return `${serverUrl}/${repository}/actions/runs/${run.runId}/attempts/${run.runAttempt}`;
}

export function escape(text: string): string {
  return text
    .replace(/[\\[\]|*_`]/g, "\\$&")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/\r?\n/g, " ");
}

function link(text: string, url: string | null): string {
  return url === null ? escape(text) : `[${escape(text)}](${url})`;
}

const count = (value: number) => value.toLocaleString("en-US");

type Bucket = "passed" | "failed" | "skipped" | "other";
const BUCKETS: readonly Bucket[] = ["passed", "failed", "skipped", "other"];
const OUTCOMES: readonly Outcome[] = ["passed", "failed", "skipped"];

function bucket(job: WorkflowJob): Bucket {
  if (job.conclusion === "success") return "passed";
  if (job.conclusion === "skipped") return "skipped";
  return job.conclusion !== null && FAILED.has(job.conclusion) ? "failed" : "other";
}

// A packet names its job by the tail that reusable-workflow callers prefix.
function producer(jobs: readonly WorkflowJob[], packet: TestCasePacket): WorkflowJob {
  const found = jobs.filter(
    (job) => job.name === packet.job || job.name.endsWith(` / ${packet.job}`),
  );
  if (found.length !== 1) throw new Error(`${found.length} jobs match case packet ${packet.job}`);
  return found[0];
}

function caseLines(input: ReportInput, jobs: readonly WorkflowJob[]): string[] {
  const { packets } = input;
  if (packets === null)
    return [`Test case reports could not be downloaded; the [run](${runUrl(input)}) has the logs.`];
  const totals: Record<Outcome, number> = { passed: 0, failed: 0, skipped: 0 };
  const areas = new Map<string, Record<Outcome, number>>();
  const failures: string[] = [];
  for (const packet of packets) {
    const job = producer(jobs, packet);
    for (const test of packet.cases) {
      const area = areaFor(test.path, test.name);
      const row = areas.get(area) ?? { passed: 0, failed: 0, skipped: 0 };
      row[test.outcome] += 1;
      totals[test.outcome] += 1;
      areas.set(area, row);
      if (test.outcome === "failed")
        failures.push(`- ${escape(area)}: ${escape(test.name)} · ${link(packet.job, job.url)}`);
    }
  }
  const sum = (row: Record<Outcome, number>) => row.passed + row.failed + row.skipped;
  const cells = (row: Record<Outcome, number>) =>
    [...OUTCOMES.map((key) => count(row[key])), count(sum(row))].join(" | ");
  const lines = [
    `**${count(sum(totals))} test cases: ${OUTCOMES.map((key) => `${count(totals[key])} ${key}`).join(", ")}**`,
    "",
    "| Area | Passed | Failed | Skipped | Total |",
    "| --- | ---: | ---: | ---: | ---: |",
    ...[...areas]
      .sort(([a], [b]) => areaRank(a) - areaRank(b) || a.localeCompare(b))
      .map(([area, row]) => `| ${escape(area)} | ${cells(row)} |`),
    `| **Total** | ${cells(totals)} |`,
    "",
    `Counted from ${packets.length} test job reports. Pull requests run only the tests their ` +
      "changes can affect; the merge queue runs all of them.",
  ];
  if (failures.length) lines.push("", "### Failed test cases", "", ...failures.sort());
  return lines;
}

function jobLines(jobs: readonly WorkflowJob[], packets: readonly TestCasePacket[]): string[] {
  const suiteName = (job: string) => job.replace(/ \([^()]*\)$/, "");
  const reported = new Set(packets.map((packet) => producer(jobs, packet)));
  type Row = Record<Bucket | "reports", number>;
  const suites = new Map<string, Row>();
  const totals = { passed: 0, failed: 0, skipped: 0, other: 0 };
  for (const job of jobs) {
    const name = suiteName(job.name);
    const row = suites.get(name) ?? { passed: 0, failed: 0, skipped: 0, other: 0, reports: 0 };
    row[bucket(job)] += 1;
    row.reports += reported.has(job) ? 1 : 0;
    totals[bucket(job)] += 1;
    suites.set(name, row);
  }
  const ran = ([, row]: [string, Row]) => row.passed + row.failed + row.other > 0;
  const rank = (row: Row) => (row.failed ? 0 : row.other ? 1 : 2);
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
    `<details${problems.length ? " open" : ""}>`,
    `<summary>${jobs.length} jobs: ${BUCKETS.map((key) => `${totals[key]} ${key}`).join(", ")}</summary>`,
    "",
    "| Suite | Passed | Failed | Skipped | Other | Case reports |",
    "| --- | ---: | ---: | ---: | ---: | ---: |",
    ...rows.map(
      ([name, row]) =>
        `| ${escape(name)} | ${[...BUCKETS, "reports" as const].map((key) => row[key]).join(" | ")} |`,
    ),
  ];
  if (notRun.length) lines.push("", `Not run: ${notRun.join(", ")}`);
  if (problems.length) lines.push("", "Failed and unfinished jobs:", "", ...problems);
  lines.push("", "</details>");
  return lines;
}

export function renderTestReport(input: ReportInput): string {
  const { run, serverUrl, repository } = input;
  const reportJob = input.jobs.find((job) => job.name === REPORT_JOB);
  if (!reportJob) throw new Error(`The run has no ${REPORT_JOB} job`);
  const jobs = input.jobs.filter((job) => job !== reportJob);
  const repo = `${serverUrl}/${repository}`;
  return [
    `## Test report: ${input.gate === "success" ? "passed" : "failed"}`,
    "",
    `Head [\`${run.headSha.slice(0, 12)}\`](${repo}/commit/${run.headSha}) · ` +
      `[${escape(input.workflow)} run ${run.runId}, attempt ${run.runAttempt}](${runUrl(input)}) · ` +
      `gate ${link(REPORT_JOB, reportJob.url)}`,
    "",
    ...caseLines(input, jobs),
    "",
    ...jobLines(jobs, input.packets ?? []),
  ].join("\n");
}
