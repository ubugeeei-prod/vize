/**
 * Wall time of `vize check` against `vue-tsc --noEmit` on the same tsconfig.
 *
 * Runs every tool alternately (rotating the order each round so load drift
 * hits all of them), after warmups, and optionally cold: a fresh `HOME` /
 * `XDG_CACHE_HOME` so vize rebuilds its virtual project from scratch (the OS
 * page cache stays warm). Records wall, user+sys CPU, vize's own stage split
 * and the diagnostic set, so a faster row that lost findings is visible.
 *
 * Usage:
 *   node tools/benchmarks/scripts/typecheck-vs-vue-tsc.ts \
 *     --project synthetic-workspace --project misskey \
 *     --vize released=/path/node_modules/.bin/vize --vize main=target/ci-opt/vize \
 *     --vue-tsc /path/node_modules/.bin/vue-tsc --runs 5 --warmups 1 --cold-runs 1 \
 *     --out typecheck-vs-vue-tsc.md --json typecheck-vs-vue-tsc.json
 *   --vize-env main:VIZE_CHECKERS=4 adds environment to one vize row.
 */

import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import { dirname, isAbsolute, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs, stripVTControlCharacters } from "node:util";

import {
  type BenchProject,
  SYNTHETIC_ID,
  prepareRealProject,
  prepareSyntheticWorkspace,
} from "./typecheck-vs-vue-tsc-projects.ts";

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const STAGES = ["collect", "imports", "gen", "corsa"] as const;
type Stage = (typeof STAGES)[number];

interface Tool {
  label: string;
  kind: "vize" | "vue-tsc";
  bin: string;
  env: Record<string, string>;
}

interface Sample {
  cold: boolean;
  wall: number;
  cpu: number;
  stages: Partial<Record<Stage, number>>;
  diagnostics: string[];
}

function seconds(text: string): number {
  const match = /^([\d.]+)(ns|µs|us|ms|s)$/u.exec(text.trim());
  if (!match) return Number.NaN;
  return Number(match[1]) * { ns: 1e-9, µs: 1e-6, us: 1e-6, ms: 1e-3, s: 1 }[match[2] as "s"];
}

function relativeTo(cwd: string, file: string) {
  return (isAbsolute(file) ? relative(cwd, file) : file).split("\\").join("/");
}

/** `file:line:code` keys; columns differ legitimately in template mappings. */
function vueTscDiagnostics(output: string, cwd: string): string[] {
  const keys: string[] = [];
  for (const line of output.split(/\r?\n/u)) {
    const match = /^(.+?)\((\d+),\d+\): error (TS\d+):/u.exec(line);
    if (match) keys.push(`${relativeTo(cwd, match[1])}:${match[2]}:${match[3]}`);
  }
  return keys.sort();
}

function vizeDiagnostics(output: string, cwd: string): string[] {
  const keys: string[] = [];
  let file = "";
  for (const line of output.split(/\r?\n/u)) {
    const diagnostic = /^\s+error:(\d+):\d+ \[(TS\d+)\]/u.exec(line);
    if (diagnostic) keys.push(`${file}:${diagnostic[1]}:${diagnostic[2]}`);
    else if (/^\S.*\.(vue|[cm]?[jt]sx?)$/u.test(line)) file = relativeTo(cwd, line.trim());
  }
  return keys.sort();
}

function vizeStages(output: string): Partial<Record<Stage, number>> {
  const line = /Type checked \d+ files in [\d.]+\S+ \(([^)]*)\)/u.exec(output);
  const stages: Partial<Record<Stage, number>> = {};
  for (const part of line?.[1].split(",") ?? []) {
    const [name, value] = part.split(":").map((piece) => piece.trim());
    if ((STAGES as readonly string[]).includes(name)) stages[name as Stage] = seconds(value);
  }
  return stages;
}

function shellQuote(value: string) {
  return `'${value.replaceAll("'", "'\\''")}'`;
}

function measure(tool: Tool, project: BenchProject, cold: boolean): Sample {
  const args =
    tool.kind === "vize"
      ? ["check", "--no-config", "--tsconfig", project.tsconfig]
      : ["--noEmit", "--pretty", "false", "-p", project.tsconfig];
  const home = cold ? mkdtempSync(join(os.tmpdir(), "tcbench-home-")) : undefined;
  const env = {
    ...process.env,
    NO_COLOR: "1",
    ...tool.env,
    ...(home ? { HOME: home, XDG_CACHE_HOME: join(home, ".cache") } : {}),
  };
  // bash's `time` reports the children's CPU, which node's spawn does not.
  const script = `TIMEFORMAT='__tcbench %R %U %S'; time ${[tool.bin, ...args].map(shellQuote).join(" ")}`;
  const result = spawnSync("bash", ["-c", script], {
    cwd: project.cwd,
    env,
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
    timeout: 1_800_000,
  });
  if (home) rmSync(home, { recursive: true, force: true });
  const output = stripVTControlCharacters(`${result.stdout}\n${result.stderr}`);
  const timing = /__tcbench ([\d.]+) ([\d.]+) ([\d.]+)/u.exec(output);
  if (!timing || ![0, 1, 2].includes(result.status ?? -1)) {
    throw new Error(
      `${tool.label} failed on ${project.id} (${result.status}):\n${output.slice(-4000)}`,
    );
  }
  const diagnostics =
    tool.kind === "vize"
      ? vizeDiagnostics(output, project.cwd)
      : vueTscDiagnostics(output, project.cwd);
  if (tool.kind === "vize" && !/Type checked \d+ files/u.test(output)) {
    throw new Error(`${tool.label} printed no summary on ${project.id}:\n${output.slice(-4000)}`);
  }
  return {
    cold,
    wall: Number(timing[1]),
    cpu: Number(timing[2]) + Number(timing[3]),
    stages: tool.kind === "vize" ? vizeStages(output) : {},
    diagnostics,
  };
}

const median = (values: number[]) => {
  if (values.length === 0) return Number.NaN;
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};

function multisetDiff(left: string[], right: string[]) {
  const counts = new Map<string, number>();
  for (const key of right) counts.set(key, (counts.get(key) ?? 0) + 1);
  let missing = 0;
  for (const key of left) {
    const count = counts.get(key) ?? 0;
    if (count > 0) counts.set(key, count - 1);
    else missing++;
  }
  return { missing, extra: [...counts.values()].reduce((sum, count) => sum + count, 0) };
}

interface ProjectResult {
  project: string;
  tools: {
    label: string;
    kind: Tool["kind"];
    warm: { wall: number; minWall: number; cpu: number; stages: Partial<Record<Stage, number>> };
    cold?: { wall: number; cpu: number };
    samples: Omit<Sample, "diagnostics">[];
    errors: number;
    /** Against the vue-tsc row: findings it reports that this row does not, and the reverse. */
    vsVueTsc?: { missing: number; extra: number };
    stable: boolean;
  }[];
}

function summarize(
  project: BenchProject,
  tools: Tool[],
  samples: Map<string, Sample[]>,
): ProjectResult {
  const baseline = tools.find((tool) => tool.kind === "vue-tsc");
  const baselineKeys = baseline ? samples.get(baseline.label)?.[0]?.diagnostics : undefined;
  return {
    project: project.id,
    tools: tools.map((tool) => {
      const rows = samples.get(tool.label) ?? [];
      const warm = rows.filter((row) => !row.cold);
      const cold = rows.filter((row) => row.cold);
      const keys = rows[0]?.diagnostics ?? [];
      return {
        label: tool.label,
        kind: tool.kind,
        warm: {
          wall: median(warm.map((row) => row.wall)),
          minWall: Math.min(...warm.map((row) => row.wall)),
          cpu: median(warm.map((row) => row.cpu)),
          stages: Object.fromEntries(
            STAGES.map((stage) => [
              stage,
              median(warm.map((row) => row.stages[stage] ?? Number.NaN)),
            ]),
          ),
        },
        cold: cold.length
          ? { wall: median(cold.map((row) => row.wall)), cpu: median(cold.map((row) => row.cpu)) }
          : undefined,
        samples: rows.map(({ diagnostics: _, ...row }) => row),
        errors: keys.length,
        vsVueTsc: baselineKeys && tool !== baseline ? multisetDiff(baselineKeys, keys) : undefined,
        stable: rows.every((row) => row.diagnostics.join("\n") === keys.join("\n")),
      };
    }),
  };
}

const fmt = (value: number | undefined) =>
  value === undefined || Number.isNaN(value) ? "–" : `${value.toFixed(2)}s`;

function markdown(results: ProjectResult[], meta: Record<string, string>) {
  const lines = [
    "## vize check vs vue-tsc",
    "",
    ...Object.entries(meta).map(([key, value]) => `- ${key}: ${value}`),
    "",
    "| project | tool | warm wall (median / min) | cold wall | CPU | collect / imports / gen / corsa | errors | vs vue-tsc (missing / extra) |",
    "| --- | --- | ---: | ---: | ---: | --- | ---: | ---: |",
  ];
  for (const result of results) {
    for (const tool of result.tools) {
      const stages =
        tool.kind === "vize" ? STAGES.map((stage) => fmt(tool.warm.stages[stage])).join(" / ") : "";
      const parity = tool.vsVueTsc ? `${tool.vsVueTsc.missing} / ${tool.vsVueTsc.extra}` : "";
      lines.push(
        `| ${result.project} | ${tool.label} | ${fmt(tool.warm.wall)} / ${fmt(tool.warm.minWall)} | ${fmt(tool.cold?.wall)} | ${fmt(tool.warm.cpu)} | ${stages} | ${tool.errors}${tool.stable ? "" : " (unstable)"} | ${parity} |`,
      );
    }
  }
  return `${lines.join("\n")}\n`;
}

function parseTools(values: {
  vize?: string[];
  "vize-env"?: string[];
  "vue-tsc"?: string;
}): Tool[] {
  const tools: Tool[] = [];
  if (values["vue-tsc"]) {
    tools.push({ label: "vue-tsc", kind: "vue-tsc", bin: resolve(values["vue-tsc"]), env: {} });
  }
  for (const spec of values.vize ?? []) {
    const [label, bin] = spec.includes("=") ? spec.split(/=(.*)/su) : ["vize", spec];
    tools.push({ label, kind: "vize", bin: resolve(bin), env: {} });
  }
  for (const spec of values["vize-env"] ?? []) {
    const match = /^([^:]+):([^=]+)=(.*)$/su.exec(spec);
    const tool = match && tools.find((candidate) => candidate.label === match[1]);
    if (!match || !tool) throw new Error(`--vize-env ${spec} names no --vize label`);
    tool.env[match[2]] = match[3];
  }
  if (!tools.some((tool) => tool.kind === "vize")) throw new Error("pass --vize label=path");
  return tools;
}

function main() {
  const { values } = parseArgs({
    options: {
      project: { type: "string", multiple: true },
      vize: { type: "string", multiple: true },
      "vize-env": { type: "string", multiple: true },
      "vue-tsc": { type: "string" },
      runs: { type: "string", default: "5" },
      warmups: { type: "string", default: "1" },
      "cold-runs": { type: "string", default: "0" },
      "fixtures-dir": { type: "string", default: join(repoRoot, "target", "typecheck-vs-vue-tsc") },
      vue: { type: "string" },
      typescript: { type: "string" },
      out: { type: "string" },
      json: { type: "string" },
      meta: { type: "string", multiple: true },
    },
  });
  const tools = parseTools(values);
  const fixturesDir = resolve(values["fixtures-dir"]);
  const projects = (values.project ?? [SYNTHETIC_ID]).map((id) =>
    id === SYNTHETIC_ID
      ? prepareSyntheticWorkspace(fixturesDir, { vue: values.vue, typescript: values.typescript })
      : prepareRealProject(repoRoot, id, fixturesDir),
  );
  const runs = Number(values.runs);
  const warmups = Number(values.warmups);
  const coldRuns = Number(values["cold-runs"]);
  const results: ProjectResult[] = [];
  for (const project of projects) {
    const samples = new Map(tools.map((tool) => [tool.label, [] as Sample[]]));
    const rotate = (round: number) =>
      tools.map((_, index) => tools[(index + round) % tools.length]);
    for (let round = 0; round < coldRuns; round++) {
      for (const tool of rotate(round)) samples.get(tool.label)?.push(measure(tool, project, true));
    }
    for (let round = 0; round < warmups; round++) {
      for (const tool of tools) measure(tool, project, false);
    }
    for (let round = 0; round < runs; round++) {
      for (const tool of rotate(round)) {
        const sample = measure(tool, project, false);
        samples.get(tool.label)?.push(sample);
        console.error(
          `${project.id} ${tool.label} ${sample.cold ? "cold" : "warm"} ${sample.wall.toFixed(2)}s`,
        );
      }
    }
    results.push(summarize(project, tools, samples));
  }
  const meta = Object.fromEntries(
    [
      `host=${os.platform()}-${os.arch()}, ${os.availableParallelism()} usable cores`,
      `runs=${runs} warm after ${warmups} warmup, ${coldRuns} cold`,
      ...(values.meta ?? []),
    ].map((entry) => entry.split(/=(.*)/su) as [string, string]),
  );
  const report = markdown(results, meta);
  process.stdout.write(report);
  if (values.out) writeFileSync(values.out, report);
  if (values.json) writeFileSync(values.json, `${JSON.stringify({ meta, results }, null, 2)}\n`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();
