// Refresh the committed tooling shard timing table from finished Actions runs:
//   node tools/support/compat/github/update-tooling-test-timings.ts <run-id>...
// Successful Tooling (...) workers and historical PR tooling scripts (i/n) jobs
// log their execution identity and per-file timings. Files a run did not execute
// keep their previous value; files no longer in the suite are dropped.
import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { stripVTControlCharacters } from "node:util";
import { toolingTestFiles } from "./plan-tooling-tests.mjs";
import {
  TOOLING_LEAD_STEPS,
  TOOLING_TIMINGS_LOG_MARKER,
  TOOLING_TIMINGS_PATH,
  validateToolingTimings,
} from "./tooling-test-shards.ts";
import type { ToolingTimings } from "./tooling-test-shards.ts";

type Tier = "pr" | "merge";
type TimingRecord = { tier: Tier; shard: string; files: Record<string, number> };
type JobStep = {
  name: string;
  conclusion: string | null;
  started_at?: string;
  completed_at?: string;
};
type Job = { id: number; name: string; conclusion: string | null; steps?: JobStep[] };
export type ToolingMeasurements = {
  tiers: Record<Tier, Record<string, number>>;
  leadSteps: Record<string, number>;
};

const LEGACY_TOOLING_JOB = /PR tooling scripts \((\d+)\/(\d+)\)$/;
const TOOLING_JOB = /(?:^| \/ )Tooling \([^\r\n]+\)$/;
// Current full-worker names differ from the historical PR lead-step names.
const currentLeadSteps: Record<string, string> = {
  "native-js-setup": "Prepare plugin runtime and test native JS setup",
  "native-attribute-values-7502": "Capture original native attribute values and fixed fresh mounts",
  "historical-sfc-documents": "Capture original historical Vue text and SFC documents",
};

export function parseToolingTimingLog(log: string): TimingRecord[] {
  const records: TimingRecord[] = [];
  for (const line of stripVTControlCharacters(log).split(/\r?\n/)) {
    const at = line.indexOf(`${TOOLING_TIMINGS_LOG_MARKER} {`);
    if (at < 0) continue;
    const record = JSON.parse(line.slice(at + TOOLING_TIMINGS_LOG_MARKER.length + 1));
    const shard =
      typeof record?.shard === "string" && /^([1-9]\d*)\/([1-9]\d*)$/.exec(record.shard);
    if (
      !["pr", "merge"].includes(record?.tier) ||
      !shard ||
      !Number.isSafeInteger(Number(shard[2])) ||
      Number(shard[1]) > Number(shard[2]) ||
      typeof record.files !== "object" ||
      record.files === null ||
      Array.isArray(record.files) ||
      !Object.values(record.files).every((ms) => Number.isSafeInteger(ms) && Number(ms) >= 0)
    ) {
      throw new Error("malformed tooling timing record");
    }
    records.push(record);
  }
  return records;
}

// Whole seconds from the Actions step API; a composite's post step counts toward it.
export function leadStepDurations(steps: readonly JobStep[]) {
  const durations: Record<string, number> = {};
  for (const { id, step } of TOOLING_LEAD_STEPS) {
    const names = [step, currentLeadSteps[id]].filter(Boolean);
    const ran = steps.filter(
      (entry) =>
        names.some((name) => entry.name === name || entry.name === `Post ${name}`) &&
        entry.conclusion === "success" &&
        entry.started_at &&
        entry.completed_at,
    );
    if (!ran.some((entry) => names.includes(entry.name))) continue;
    durations[id] = ran.reduce(
      (sum, entry) => sum + Date.parse(entry.completed_at!) - Date.parse(entry.started_at!),
      0,
    );
  }
  return durations;
}

export function measureToolingJobs(jobs: readonly Job[], logs: (id: number) => string) {
  const measured: ToolingMeasurements = { tiers: { pr: {}, merge: {} }, leadSteps: {} };
  for (const job of jobs) {
    const legacy = LEGACY_TOOLING_JOB.exec(job.name);
    if ((!legacy && !TOOLING_JOB.test(job.name)) || job.conclusion !== "success") continue;
    let shard = legacy ? `${legacy[1]}/${legacy[2]}` : undefined;
    for (const record of parseToolingTimingLog(logs(job.id))) {
      if (shard && record.shard !== shard) throw new Error(`${job.name}: shard mismatch`);
      shard = record.shard;
      Object.assign(measured.tiers[record.tier], record.files);
    }
    // A semantic runtime-capture worker can have no test files or timing record.
    // Its actual successful steps identify the lead work without inventing an index.
    if (!legacy || legacy[1] === "1")
      Object.assign(measured.leadSteps, leadStepDurations(job.steps ?? []));
  }
  return measured;
}

const sortedEntries = (values: Record<string, number>, keep: (key: string) => boolean) =>
  Object.fromEntries(
    Object.entries(values)
      .filter(([key]) => keep(key))
      .sort(([left], [right]) => (left < right ? -1 : left > right ? 1 : 0)),
  );

export function mergeToolingTimings(
  table: ToolingTimings,
  measured: ToolingMeasurements,
  files: readonly string[],
): ToolingTimings {
  const suite = new Set(files);
  return validateToolingTimings({
    schema: table.schema,
    defaultMs: table.defaultMs,
    leadSteps: { ...table.leadSteps, ...measured.leadSteps },
    tiers: {
      pr: sortedEntries({ ...table.tiers.pr, ...measured.tiers.pr }, (file) => suite.has(file)),
      merge: sortedEntries({ ...table.tiers.merge, ...measured.tiers.merge }, (file) =>
        suite.has(file),
      ),
    },
  });
}

function gh(args: string[]) {
  const result = spawnSync("gh", args, { encoding: "utf8", maxBuffer: 256 * 1024 * 1024 });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`gh ${args.join(" ")} failed:\n${result.stderr}`);
  return result.stdout;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const repoFlag = args.indexOf("--repo");
  const repo =
    repoFlag >= 0
      ? args.splice(repoFlag, 2)[1]
      : (process.env.GITHUB_REPOSITORY ?? "ubugeeei-prod/vize");
  if (args.length === 0 || args.some((run) => !/^\d+$/.test(run))) {
    throw new Error("usage: update-tooling-test-timings.ts [--repo owner/name] <run-id>...");
  }
  let table = validateToolingTimings(JSON.parse(readFileSync(TOOLING_TIMINGS_PATH, "utf8")));
  for (const run of args) {
    const jobs = gh([
      "api",
      "--paginate",
      `repos/${repo}/actions/runs/${run}/jobs?per_page=100`,
      "--jq",
      ".jobs[]",
    ])
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line) as Job);
    const measured = measureToolingJobs(jobs, (id) =>
      gh(["api", `repos/${repo}/actions/jobs/${id}/logs`]),
    );
    const counts = Object.entries(measured.tiers).map(([tier, files]) => {
      return `${tier} ${Object.keys(files).length}`;
    });
    process.stdout.write(
      `run ${run}: files ${counts.join(", ")}; lead steps ${Object.keys(measured.leadSteps).length}\n`,
    );
    table = mergeToolingTimings(table, measured, toolingTestFiles());
  }
  writeFileSync(TOOLING_TIMINGS_PATH, `${JSON.stringify(table, null, 2)}\n`);
}
