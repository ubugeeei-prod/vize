/**
 * Type-check scaling lane: generated Vue projects of increasing size, and a
 * project-references monorepo, checked by vize check, tsc/tsrs with the Vize
 * mapper, vue-tsc and tsc with Vue's mapper.
 *
 *   node typecheck-scaling.ts --target <built checkout> --sha <sha> \
 *     --sizes 1000,5000,20000 --runs 3 --monorepo-sfcs 5000 \
 *     --work <dir> --out <dir>
 *
 * Every run must report exactly the planted errors (typecheck-scaling-gate.ts)
 * or the tool's timing for that corpus is refused and the lane exits non-zero.
 * Timing needs Linux (/proc/self/stat for CPU, GNU time for peak RSS);
 * elsewhere the lane only gates, as a smoke test.
 */

import { spawnSync } from "node:child_process";
import {
  cpSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { availableParallelism, totalmem } from "node:os";
import { join, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { reapedChildCpuMs } from "./typecheck-command.mjs";
import {
  type Corpus,
  generateMonorepo,
  generateProject,
  monorepoMapperConfigs,
  monorepoOutDir,
  writeCorpus,
} from "./typecheck-scaling-corpus.ts";
import {
  STAT_KEYS,
  type ProcessOutput,
  gate,
  parseRun,
  refusalText,
  statNumber,
} from "./typecheck-scaling-gate.ts";
import {
  type Cell,
  type Sample,
  type ScalingResults,
  renderSummary,
} from "./typecheck-scaling-report.ts";
import { renderScalingSvg } from "./typecheck-scaling-svg.ts";
import {
  type ScalingEnv,
  type ScalingTool,
  TOOLS,
  collectProvenance,
  invocation,
  prepareProjectPackages,
  resolveScalingEnv,
} from "./typecheck-scaling-tools.ts";
import { CONTENT_MAPPERS, mapperTsconfigName } from "./typecheck-mapper-hosts.ts";

const GNU_TIME = "/usr/bin/time";
const GNU_TIME_FORMAT = "vize-time wall=%e user=%U sys=%S maxrss_kb=%M status=%x";
const GNU_TIME_LINE =
  /^vize-time wall=(\d+\.\d+) user=(\d+\.\d+) sys=(\d+\.\d+) maxrss_kb=(\d+) status=(\d+)$/gmu;
/** vue-tsc on the largest corpora needs more than V8's default old space. */
export const NODE_OPTIONS = "--max-old-space-size=98304";

export function parseGnuTime(text: string): { maxRssKb: number; status: number } {
  const matches = [...text.matchAll(GNU_TIME_LINE)];
  if (matches.length !== 1) throw new Error(`unexpected GNU time output:\n${text}`);
  const match = matches[0] as RegExpMatchArray;
  return { maxRssKb: Number(match[4]), status: Number(match[5]) };
}

function gnuTimeVersion(): string {
  const probe = spawnSync(GNU_TIME, ["--version"], { encoding: "utf8" });
  const version = `${probe.stdout ?? ""}${probe.stderr ?? ""}`.trim().split("\n")[0] ?? "";
  if (probe.status !== 0 || !/GNU time/iu.test(version)) {
    throw new Error(`peak RSS needs GNU time at ${GNU_TIME}; found: ${version || probe.error}`);
  }
  return version;
}

/** Even runs keep the tool order, odd runs reverse it, so no tool always runs first. */
export const runOrder = <T>(items: readonly T[], run: number): T[] =>
  run % 2 === 0 ? [...items] : [...items].reverse();

type Measured = {
  output: ProcessOutput;
  wallMs: number;
  cpuMs: number | null;
  maxRssKb: number | null;
};

function measure(
  bin: string,
  args: string[],
  cwd: string,
  cacheDir: string,
  timeFile: string,
  timeoutMs: number,
  timed: boolean,
  extraEnv: Record<string, string> = {},
): Measured {
  const [command, argv] = timed
    ? [GNU_TIME, ["-q", "-f", GNU_TIME_FORMAT, "-o", timeFile, bin, ...args]]
    : [bin, args];
  const before = timed ? (reapedChildCpuMs() as number | null) : null;
  const start = performance.now();
  const result = spawnSync(command, argv, {
    cwd,
    env: {
      ...process.env,
      NO_COLOR: "1",
      FORCE_COLOR: "0",
      XDG_CACHE_HOME: cacheDir,
      NODE_OPTIONS,
      ...extraEnv,
    },
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 1024,
    timeout: timeoutMs,
    killSignal: "SIGKILL",
  });
  const wallMs = performance.now() - start;
  const after = timed ? (reapedChildCpuMs() as number | null) : null;
  if (result.error) {
    // GNU time dies with the timeout; its child may not. Nothing else runs under these paths.
    for (const path of [cwd, cacheDir]) spawnSync("pkill", ["-KILL", "-f", path]);
    throw new Error(`${bin} did not finish: ${result.error.message}`);
  }
  const output = { status: result.status, stdout: result.stdout, stderr: result.stderr };
  if (!timed) return { output, wallMs, cpuMs: null, maxRssKb: null };
  const time = parseGnuTime(readFileSync(timeFile, "utf8"));
  if (time.status !== result.status) {
    throw new Error(`GNU time saw status ${time.status}, the runner saw ${result.status}`);
  }
  return {
    output,
    wallMs,
    cpuMs: before == null || after == null ? null : after - before,
    maxRssKb: time.maxRssKb,
  };
}

type Lane = { cell: Cell; tool: ScalingTool; firstFingerprint: string | null };

function runOnce(
  lane: Lane,
  corpus: Corpus,
  dir: string,
  env: ScalingEnv,
  options: { work: string; out: string; timeoutMs: number; timed: boolean },
  phase: string,
): Sample {
  for (const name of corpus.packages) {
    for (const entry of readdirSync(join(dir, name))) {
      if (entry.startsWith(monorepoOutDir(""))) rmSync(join(dir, name, entry), { recursive: true });
    }
  }
  const { bin, args, stats } = invocation(lane.tool, env, dir, corpus.kind);
  const cacheDir = mkdtempSync(join(options.work, "cache-"));
  const tag = `${lane.tool.id}-${corpus.kind}-${corpus.sfcs}`;
  try {
    const run = measure(
      bin,
      args,
      dir,
      cacheDir,
      join(options.work, `${tag}.time`),
      options.timeoutMs,
      options.timed,
    );
    let parsed: ReturnType<typeof parseRun>;
    try {
      parsed = parseRun(run.output, dir, lane.tool.format);
    } catch (error) {
      throw new Error(
        `${(error as Error).message.slice(0, 4000)}\nstderr:\n${run.output.stderr.slice(-2000)}`,
      );
    }
    if (phase === "cold") {
      writeFileSync(
        join(options.out, "diagnostics", `${tag}.json`),
        `${JSON.stringify(parsed.diagnostics, null, 1)}\n`,
      );
    }
    const verdict = gate(lane.tool.id, parsed.diagnostics, corpus.plants);
    if (!verdict.ok) throw new Error(refusalText(lane.tool.id, verdict));
    if (lane.firstFingerprint != null && lane.firstFingerprint !== parsed.fingerprint) {
      throw new Error(`${lane.tool.id}: diagnostics changed between runs (${phase})`);
    }
    lane.firstFingerprint = parsed.fingerprint;
    return {
      wallMs: run.wallMs,
      cpuMs: run.cpuMs,
      maxRssKb: run.maxRssKb,
      errors: verdict.errors,
      warnings: verdict.warnings,
      fingerprint: parsed.fingerprint,
      types: stats ? statNumber(parsed.stats, STAT_KEYS.types) : null,
      instantiations: stats ? statNumber(parsed.stats, STAT_KEYS.instantiations) : null,
      checkTimeS: stats ? statNumber(parsed.stats, STAT_KEYS.checkTimeS) : null,
    };
  } finally {
    rmSync(cacheDir, { recursive: true, force: true });
  }
}

function main(): void {
  const { values } = parseArgs({
    options: {
      target: { type: "string" },
      sha: { type: "string", default: "" },
      sizes: { type: "string", default: "1000,5000,20000" },
      runs: { type: "string", default: "3" },
      "monorepo-sfcs": { type: "string", default: "5000" },
      "monorepo-packages": { type: "string", default: "6" },
      tools: { type: "string", default: TOOLS.map((tool) => tool.id).join(",") },
      "timeout-min": { type: "string", default: "45" },
      runner: { type: "string", default: process.env.VIZE_SCALING_RUNNER_LABEL ?? "local" },
      "run-url": { type: "string" },
      work: { type: "string" },
      out: { type: "string" },
      keep: { type: "boolean", default: false },
    },
  });
  if (!values.target || !values.work || !values.out) {
    throw new Error("--target, --work and --out are required");
  }
  const runs = Number(values.runs);
  if (!Number.isInteger(runs) || runs < 1) throw new Error("--runs must be a positive integer");
  const sizes = values.sizes
    .split(",")
    .filter((size) => size.trim())
    .map(Number);
  const monorepoSfcs = Number(values["monorepo-sfcs"]);
  const tools = values.tools.split(",").map((id) => {
    const tool = TOOLS.find((candidate) => candidate.id === id);
    if (!tool) throw new Error(`unknown tool ${id}; known: ${TOOLS.map((t) => t.id).join(", ")}`);
    return tool;
  });
  const timed = process.platform === "linux";
  const gnuTime = timed ? gnuTimeVersion() : null;
  const env = resolveScalingEnv(resolve(values.target));
  const work = resolve(values.work);
  const out = resolve(values.out);
  mkdirSync(join(out, "diagnostics"), { recursive: true });
  mkdirSync(join(out, "plants"), { recursive: true });
  mkdirSync(work, { recursive: true });
  const options = { work, out, timeoutMs: Number(values["timeout-min"]) * 60_000, timed };
  const results: ScalingResults = {
    schemaVersion: 1,
    sha: values.sha,
    runner: values.runner,
    machine: `${availableParallelism()} vCPUs, ${(totalmem() / 2 ** 30).toFixed(0)} GiB`,
    measured: timed,
    runs,
    generatedAt: new Date().toISOString(),
    runUrl: values["run-url"] ?? null,
    nodeOptions: NODE_OPTIONS,
    gnuTime,
    provenance: collectProvenance(env),
    cells: [],
  };
  const save = () => {
    writeFileSync(join(out, "results.json"), `${JSON.stringify(results, null, 1)}\n`);
    writeFileSync(join(out, "summary.md"), renderSummary(results));
    writeFileSync(join(out, "scaling.svg"), renderScalingSvg(results));
  };
  const corpora: (() => Corpus)[] = [
    ...sizes.map((sfcs) => () => generateProject(sfcs)),
    ...(monorepoSfcs > 0
      ? [() => generateMonorepo(monorepoSfcs, Number(values["monorepo-packages"]))]
      : []),
  ];
  for (const make of corpora) {
    const corpus = make();
    const dir = join(work, `${corpus.kind}-${corpus.sfcs}`);
    writeCorpus(dir, corpus.files);
    prepareProjectPackages(dir, env);
    if (corpus.kind === "monorepo") {
      for (const mapper of CONTENT_MAPPERS) {
        writeCorpus(dir, monorepoMapperConfigs(corpus, mapper, mapperTsconfigName(mapper)), false);
      }
    }
    writeFileSync(
      join(out, "plants", `${corpus.kind}-${corpus.sfcs}.json`),
      `${JSON.stringify(corpus.plants, null, 1)}\n`,
    );
    const lanes: Lane[] = tools.map((tool) => {
      const cell: Cell = {
        tool: tool.id,
        label: tool.label,
        vize: tool.vize,
        kind: corpus.kind,
        sfcs: corpus.sfcs,
        plants: corpus.plants.length,
        mode: corpus.kind === "project" ? "-p" : tool.id === "vize-check" ? "solution root" : "-b",
        cold: null,
        runs: [],
        refused: null,
      };
      results.cells.push(cell);
      return { cell, tool, firstFingerprint: null };
    });
    const label = `${corpus.kind} ${corpus.sfcs} SFCs (${corpus.features} features, ${corpus.plants.length} plants)`;
    console.log(`== ${label}`);
    for (const [index, phase] of [
      "cold",
      ...Array.from({ length: runs }, (_, run) => `run ${run + 1}`),
    ].entries()) {
      for (const lane of runOrder(lanes, index)) {
        if (lane.cell.refused) continue;
        try {
          const sample = runOnce(lane, corpus, dir, env, options, phase);
          if (phase === "cold") lane.cell.cold = sample;
          else lane.cell.runs.push(sample);
          console.log(
            `${lane.tool.id} ${phase}: wall ${(sample.wallMs / 1000).toFixed(2)}s cpu ${sample.cpuMs == null ? "n/a" : (sample.cpuMs / 1000).toFixed(2)}s rss ${sample.maxRssKb ?? "n/a"}K, ${sample.errors} errors`,
          );
        } catch (error) {
          lane.cell.refused = (error as Error).message;
          console.error(
            `::error::${lane.tool.id} on ${label} refused (${phase}):\n${lane.cell.refused}`,
          );
        }
        save();
      }
      if (!timed) break;
    }
    if (process.env.VIZE_SCALING_PROFILE === "1" && corpus.kind === "project") {
      // One untimed profiled vize check, in a fresh cache, with the phases and artifacts kept.
      const profileDir = join(out, "profile", `${corpus.sfcs}`);
      mkdirSync(profileDir, { recursive: true });
      const vize = tools.find((tool) => tool.id === "vize-check");
      if (vize) {
        const base = invocation(vize, env, dir, corpus.kind);
        const args = [
          ...base.args.filter((arg) => arg !== "--quiet" && arg !== "--format" && arg !== "json"),
          "--profile",
        ];
        const started = performance.now();
        const result = spawnSync(base.bin, args, {
          cwd: dir,
          env: {
            ...process.env,
            NO_COLOR: "1",
            FORCE_COLOR: "0",
            XDG_CACHE_HOME: mkdtempSync(join(work, "cache-")),
            NODE_OPTIONS,
          },
          encoding: "utf8",
          maxBuffer: 1024 * 1024 * 1024,
        });
        writeFileSync(
          join(profileDir, "vize-check.txt"),
          `wall ${((performance.now() - started) / 1000).toFixed(2)}s status ${result.status}\n--- stderr\n${result.stderr}\n--- stdout (tail)\n${(result.stdout ?? "").slice(-20000)}`,
        );
        const artifacts = join(dir, "node_modules", ".vize", "check-profile");
        if (existsSync(artifacts))
          cpSync(artifacts, join(profileDir, "check-profile"), { recursive: true });
        // Checker-width sweep: wall, CPU, RSS and diagnostic identity per Corsa width.
        const sweep: Record<string, unknown>[] = [];
        let reference: string | null = null;
        for (const width of (process.env.VIZE_SCALING_WIDTHS ?? "1,2,4,8,16,32").split(",")) {
          const run = measure(
            base.bin,
            base.args,
            dir,
            mkdtempSync(join(work, "cache-")),
            join(work, `width-${width}.time`),
            options.timeoutMs,
            true,
            { VIZE_CHECKERS: width },
          );
          const parsed = parseRun(run.output, dir, vize.format);
          reference ??= parsed.fingerprint;
          sweep.push({
            width: Number(width),
            wallMs: Math.round(run.wallMs),
            cpuMs: run.cpuMs,
            maxRssKb: run.maxRssKb,
            diagnostics: parsed.diagnostics.length,
            identical: parsed.fingerprint === reference,
          });
          console.log(`width ${width} @ ${corpus.sfcs}: ${JSON.stringify(sweep.at(-1))}`);
        }
        writeFileSync(join(profileDir, "width-sweep.json"), `${JSON.stringify(sweep, null, 1)}\n`);
      }
    }
    if (!values.keep) rmSync(dir, { recursive: true, force: true });
  }
  save();
  const refused = results.cells.filter((cell) => cell.refused);
  if (refused.length > 0) {
    console.error(`::error::${refused.length} cell(s) refused; see summary.md`);
    process.exitCode = 1;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
