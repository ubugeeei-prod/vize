// Refresh the committed tooling shard timing table from finished Actions runs:
//   node tools/support/compat/github/update-tooling-test-timings.ts <run-id>...
// Legacy `PR tooling scripts (i/n)` and current `Tooling (cohort)` jobs log timing records.
// Finished failed/cancelled jobs retain their observations too. A run's
// event decides nothing: every record names its own tier. Files a run did not execute keep
// their previous value; files no longer in the suite are dropped.
import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { stripVTControlCharacters } from "node:util";
import {
  MAX_TOOLING_SHARDS,
  TOOLING_LEAD_STEPS,
  TOOLING_TIMINGS_LOG_MARKER,
  TOOLING_TIMINGS_PATH,
  validateToolingTimings,
  toolingTestFiles,
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

// `Affected tooling (cohort)` is the PR tier's worker since #9681.
const TOOLING_JOB =
  /(?:^| \/ )(?:(PR tooling scripts) \(([^()]*)\)|((?:Affected t|T)ooling) \(([^()]*)\))$/;
const TOOLING_FILE = /^tests\/tooling\/(?:[^/]+\/)?[^/]+\.test\.(?:ts|mjs)$/;
const CURRENT_LEAD_NAMES: Record<string, string> = {
  "native-js-setup": "Prepare plugin runtime and test native JS setup",
  "native-attribute-values-7502": "Capture original native attribute values and fixed fresh mounts",
  "historical-sfc-documents": "Join original historical Vue documents to pinned whole goldens",
};

function coordinates(shard: string) {
  const match = /^([1-9]\d*)\/([1-9]\d*)$/.exec(shard);
  const index = Number(match?.[1]);
  const total = Number(match?.[2]);
  if (
    !match ||
    !Number.isSafeInteger(index) ||
    !Number.isSafeInteger(total) ||
    index > total ||
    total > MAX_TOOLING_SHARDS
  ) {
    throw new Error("invalid tooling timing shard");
  }
  return { index, total };
}

// Display names identify an actual file cohort, while the emitted record retains
// the authoritative numeric coordinates. Nested test subprocesses can emit extra
// paths; retain those raw observations without mistaking them for cohort members.
function assertCohort(name: string, records: readonly TimingRecord[]) {
  const files = [...new Set(records.flatMap((record) => Object.keys(record.files)))].filter(
    (file) => TOOLING_FILE.test(file),
  );
  if (files.length === 0 && ["runtime captures", "no selected test files"].includes(name)) return;
  const match = /^(.*?)(?: \+ ([1-9]\d*) files)?$/.exec(name)!;
  const count = match[2] ? Number(match[2]) + 1 : 1;
  const primary = files.some(
    (file) => file.replace(/^tests\/tooling\//, "").replace(/\.test\.(?:ts|mjs)$/, "") === match[1],
  );
  if (!primary || count !== files.length) throw new Error(`Tooling (${name}): cohort mismatch`);
}

export function parseToolingTimingLog(log: string): TimingRecord[] {
  const records: TimingRecord[] = [];
  for (const line of stripVTControlCharacters(log).split(/\r?\n/)) {
    const at = line.indexOf(`${TOOLING_TIMINGS_LOG_MARKER} {`);
    if (at < 0) continue;
    const record = JSON.parse(line.slice(at + TOOLING_TIMINGS_LOG_MARKER.length + 1));
    if (
      !["pr", "merge"].includes(record?.tier) ||
      typeof record.shard !== "string" ||
      typeof record.files !== "object" ||
      record.files === null ||
      Array.isArray(record.files) ||
      !Object.values(record.files).every((ms) => Number.isSafeInteger(ms) && Number(ms) >= 0)
    ) {
      throw new Error("malformed tooling timing record");
    }
    coordinates(record.shard);
    records.push(record);
  }
  return records;
}

// Whole seconds from the Actions step API; a composite's post step counts toward it.
export function leadStepDurations(steps: readonly JobStep[]) {
  const durations: Record<string, number> = {};
  for (const { id, step } of TOOLING_LEAD_STEPS) {
    const names = [step, CURRENT_LEAD_NAMES[id]].filter(Boolean);
    const ran = steps.filter(
      (entry) =>
        names.some((name) => entry.name === name || entry.name === `Post ${name}`) &&
        ["success", "failure", "cancelled"].includes(entry.conclusion ?? "") &&
        entry.started_at &&
        entry.completed_at,
    );
    if (!ran.some((entry) => names.includes(entry.name))) continue;
    durations[id] = ran.reduce((sum, entry) => {
      const ms = Date.parse(entry.completed_at!) - Date.parse(entry.started_at!);
      if (!Number.isSafeInteger(ms) || ms < 0)
        throw new Error(`${entry.name}: invalid step duration`);
      return sum + ms;
    }, 0);
  }
  return durations;
}

// Composite leaf steps are absent from the Actions jobs API. The runner's raw
// action spans retain exact outcomes and durations, including a capture that
// fails before the per-file reporter can run. Never assign the whole composite
// duration to a native capture: it includes installation and unrelated tests.
export function parseToolingLeadLog(log: string) {
  const pending = new Map<string, { id: string; post: boolean }>();
  const durations: Record<string, number> = {};
  const ran = new Set<string>();
  for (const line of stripVTControlCharacters(log).split(/\r?\n/)) {
    const start = /##\[start-action display=(.*);id=([^;\]]+)\]/.exec(line);
    if (start) {
      for (const { id, step } of TOOLING_LEAD_STEPS) {
        const names = [step, CURRENT_LEAD_NAMES[id]].filter(Boolean);
        if (names.some((name) => start[1] === name || start[1] === `Post ${name}`)) {
          pending.set(start[2], { id, post: start[1].startsWith("Post ") });
        }
      }
    }
    const end =
      /##\[end-action id=([^;\]]+);outcome=([^;]+);conclusion=([^;]+);duration_ms=([^;\]]+)\]/.exec(
        line,
      );
    const capture = end && pending.get(end[1]);
    if (!end || !capture) continue;
    pending.delete(end[1]);
    if (end[2] === "skipped") continue;
    const ms = Number(end[4]);
    if (!/^\d+$/.test(end[4]) || !Number.isSafeInteger(ms))
      throw new Error("invalid tooling lead duration");
    durations[capture.id] = (durations[capture.id] ?? 0) + ms;
    if (!capture.post) ran.add(capture.id);
  }
  return Object.fromEntries(Object.entries(durations).filter(([id]) => ran.has(id)));
}

export function measureToolingJobs(jobs: readonly Job[], logs: (id: number) => string) {
  const measured: ToolingMeasurements = { tiers: { pr: {}, merge: {} }, leadSteps: {} };
  for (const job of jobs) {
    const identity = TOOLING_JOB.exec(job.name);
    if (!identity || job.conclusion === null || job.conclusion === "skipped") continue;
    const legacyShard = identity[1] ? identity[2] : undefined;
    if (legacyShard) coordinates(legacyShard);
    const log = logs(job.id);
    const records = parseToolingTimingLog(log);
    if (identity[3] && records.length) assertCohort(identity[4], records);
    const shard = records[0]?.shard ?? legacyShard;
    for (const record of records) {
      if (record.shard !== shard || (legacyShard && record.shard !== legacyShard))
        throw new Error(`${job.name}: shard mismatch`);
      if (record.tier !== records[0].tier) throw new Error(`${job.name}: tier mismatch`);
      Object.assign(measured.tiers[record.tier], record.files);
    }
    const lead = { ...leadStepDurations(job.steps ?? []), ...parseToolingLeadLog(log) };
    if (shard && coordinates(shard).index !== 1) {
      // Only runner 1 owns lead work; modern capture evidence must agree
      // with the numeric coordinates emitted by its actual worker.
      if (identity[3] && Object.keys(lead).length)
        throw new Error(`${job.name}: lead timing outside shard 1`);
    } else {
      Object.assign(measured.leadSteps, lead);
    }
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
      gh(["api", "--allow-escape-sequences", `repos/${repo}/actions/jobs/${id}/logs`]),
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
