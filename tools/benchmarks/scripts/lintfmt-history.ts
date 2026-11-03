#!/usr/bin/env node
/**
 * Lint and format CLI benchmark across vize builds (published or source-built).
 *
 * Measures the README commands (`vize lint . --quiet`, `vize fmt --write *.vue`)
 * per build, 1T (RAYON_NUM_THREADS=1) and max apart: wall, CPU-seconds of the
 * whole process tree (user+sys from wait4, so a Node config child counts), perf
 * task-clock when --perf is given, and peak RSS of the largest process. Each
 * build also gets one identity run: lint diagnostics text and formatted bytes
 * are hashed so two builds can be shown to produce identical output.
 *
 * Config setups: `vite` is the generated corpus (vite.config.mjs); `json` swaps
 * it for the equivalent vize.config.json that builds before the vite.config
 * migration read. A setup a build rejects (non-zero identity exit other than
 * lint's 1) is reported and skipped.
 *
 *   node lintfmt-history.ts --corpus DIR --work DIR --bin label=/path/vize ... \
 *     [--setups vite,json] [--tasks lint,fmt] [--threads 1t,max] [--runs 3] \
 *     [--warmups 1] [--rounds 2] [--perf /path/perf] [--valgrind] \
 *     [--out summary.md] [--json results.json]
 */

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { join, resolve } from "node:path";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";

import { isRecord } from "./script-types.ts";

type Task = "lint" | "fmt";
type Threads = "1t" | "max";
type Setup = "vite" | "json";
type Bin = { label: string; path: string };
type Spec = { bin: Bin; setup: Setup; task: Task; threads: Threads };
type Sample = { wallMs: number; cpuMs: number; taskClockMs: number; maxRssKb: number };
type Identity = {
  exit: number;
  summary: string;
  outputBytes: number;
  outputSha256: string;
  configChildWallMs: number;
  instructions: number;
};

function parseArgs(argv: string[]): Map<string, string[]> {
  const args = new Map<string, string[]>();
  for (let i = 0; i < argv.length; i++) {
    const key = argv[i];
    if (!key?.startsWith("--")) continue;
    const next = argv[i + 1];
    const value = next === undefined || next.startsWith("--") ? "true" : (i++, next);
    args.set(key.slice(2), [...(args.get(key.slice(2)) ?? []), value]);
  }
  return args;
}

function median(values: number[]): number {
  const sorted = values.filter(Number.isFinite).sort((a, b) => a - b);
  if (sorted.length === 0) return NaN;
  const mid = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1
    ? (sorted[mid] ?? NaN)
    : ((sorted[mid - 1] ?? NaN) + (sorted[mid] ?? NaN)) / 2;
}

function copyDir(from: string, to: string, keep: (name: string) => boolean): void {
  rmSync(to, { recursive: true, force: true });
  mkdirSync(to, { recursive: true });
  for (const name of readdirSync(from)) {
    if (keep(name)) copyFileSync(join(from, name), join(to, name));
  }
}

/** The vize.config.json the corpus generator wrote before the vite.config migration. */
async function jsonConfigFor(corpus: string): Promise<string> {
  const loaded: unknown = await import(pathToFileURL(join(corpus, "vite.config.mjs")).href);
  const vue = isRecord(loaded) && isRecord(loaded.default) ? loaded.default.vue : undefined;
  if (!isRecord(vue)) throw new Error("vite.config.mjs has no vue settings");
  const typeChecker = isRecord(vue.typeChecker) ? { ...vue.typeChecker } : {};
  delete typeChecker.jsxTypecheck;
  return `${JSON.stringify({ linter: vue.linter, typeChecker }, null, 2)}\n`;
}

class Bench {
  readonly corpus: string;
  readonly work: string;
  readonly perf: string | undefined;
  readonly lintDirs: Record<Setup, string>;
  readonly vueFiles: string[];
  readonly jsonConfig: string;
  private fmtRun = 0;

  constructor(corpus: string, work: string, perf: string | undefined, jsonConfig: string) {
    this.corpus = corpus;
    this.work = work;
    this.perf = perf;
    this.jsonConfig = jsonConfig;
    this.vueFiles = readdirSync(corpus).filter((name) => name.endsWith(".vue"));
    const jsonDir = `${corpus.replace(/\/$/, "")}-json`;
    copyDir(corpus, jsonDir, (name) => name !== "vite.config.mjs");
    writeFileSync(join(jsonDir, "vize.config.json"), jsonConfig);
    this.lintDirs = { vite: corpus, json: jsonDir };
  }

  /** A fresh unformatted copy, as the README's prepareFormatDir makes. */
  fmtDir(setup: Setup): string {
    const dir = join(this.work, `fmt-${setup}-${String(++this.fmtRun).padStart(4, "0")}`);
    copyDir(this.corpus, dir, (name) => name.endsWith(".vue") || (setup === "vite" && name === "vite.config.mjs"));
    if (setup === "json") writeFileSync(join(dir, "vize.config.json"), this.jsonConfig);
    return dir;
  }

  command(spec: Pick<Spec, "task" | "setup">, quiet = true): { args: string[]; cwd: string } {
    if (spec.task === "lint") {
      return { args: quiet ? ["lint", ".", "--quiet"] : ["lint", "."], cwd: this.lintDirs[spec.setup] };
    }
    return { args: ["fmt", "--write", "*.vue"], cwd: this.fmtDir(spec.setup) };
  }

  env(threads: Threads, extra: NodeJS.ProcessEnv = {}): NodeJS.ProcessEnv {
    const env = { ...process.env, ...extra };
    if (threads === "1t") env.RAYON_NUM_THREADS = "1";
    else delete env.RAYON_NUM_THREADS;
    return env;
  }

  /** One timed process: wall here, rusage from GNU time, task-clock from perf. */
  run(spec: Spec): Sample & { exit: number; stdout: string; stderr: string; cwd: string } {
    const { args, cwd } = this.command(spec);
    const timeFile = join(this.work, "time.txt");
    const statFile = join(this.work, "stat.csv");
    const wrapped = this.perf
      ? [this.perf, "stat", "-x,", "-o", statFile, "-e", "task-clock", "--", spec.bin.path, ...args]
      : [spec.bin.path, ...args];
    const start = performance.now();
    const child = spawnSync("/usr/bin/time", ["-f", "%U %S %M", "-o", timeFile, ...wrapped], {
      cwd,
      env: this.env(spec.threads),
      encoding: "utf8",
      maxBuffer: 256 * 1024 * 1024,
    });
    const wallMs = performance.now() - start;
    if (child.error) throw child.error;
    const [user, sys, rss] = readFileSync(timeFile, "utf8").trim().split("\n").at(-1)?.split(" ").map(Number) ?? [];
    let taskClockMs = NaN;
    if (this.perf) {
      const line = readFileSync(statFile, "utf8").split("\n").find((row) => row.includes("task-clock"));
      taskClockMs = Number(line?.split(",")[0]);
    }
    return {
      wallMs,
      cpuMs: ((user ?? NaN) + (sys ?? NaN)) * 1000,
      taskClockMs,
      maxRssKb: rss ?? NaN,
      exit: child.status ?? -1,
      stdout: child.stdout,
      stderr: child.stderr,
      cwd,
    };
  }

  /** Output identity, config-child wall and (optionally) instruction counts for one build. */
  identity(bin: Bin, setup: Setup, task: Task, valgrind: boolean): Identity {
    const hash = createHash("sha256");
    let summary = "";
    let outputBytes = 0;
    let exit: number;
    let stderr: string;
    const { args, cwd } = this.command({ task, setup }, false);
    if (task === "lint") {
      const child = spawnSync(bin.path, args, { cwd, env: this.env("max"), encoding: "utf8", maxBuffer: 512 * 1024 * 1024 });
      exit = child.status ?? -1;
      stderr = child.stderr;
      const stdout = child.stdout
        .split("\n")
        .filter((line) => !/^Linted \d+ files in /.test(line))
        .join("\n");
      hash.update(stdout);
      outputBytes = Buffer.byteLength(stdout);
      const lines = child.stdout.trim().split("\n");
      summary = lines.slice(-2).join(" / ").replace(/ in [\d.]+[µm]?s$/, "");
    } else {
      const child = spawnSync(bin.path, args, { cwd, env: this.env("max"), encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
      exit = child.status ?? -1;
      stderr = child.stderr;
      for (const name of [...this.vueFiles].sort()) {
        const bytes = readFileSync(join(cwd, name));
        hash.update(name);
        hash.update("\0");
        hash.update(bytes);
        outputBytes += bytes.length;
      }
      summary = `${child.stdout}${child.stderr}`.trim().split("\n").filter((line) => /file\(s\)/.test(line)).join(" / ");
    }
    if (exit !== 0 && !(task === "lint" && exit === 1)) {
      summary = `rejected (exit ${exit}): ${stderr.trim().split("\n").slice(-3).join(" / ").slice(0, 300)}`;
    }
    if (task === "fmt") rmSync(cwd, { recursive: true, force: true });
    const traced = this.traceConfig(bin, setup, task);
    const instructions = valgrind && !summary.startsWith("rejected") ? this.instructions(bin, setup, task) : NaN;
    return { exit, summary, outputBytes, outputSha256: hash.digest("hex"), configChildWallMs: traced, instructions };
  }

  /** Wall of the Node config evaluation child, when the build traces it. */
  traceConfig(bin: Bin, setup: Setup, task: Task): number {
    const { args, cwd } = this.command({ task, setup });
    const child = spawnSync(bin.path, args, {
      cwd,
      env: this.env("1t", { VIZE_TRACE_EDITOR_PREPARATION: "1" }),
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
    });
    if (task === "fmt") rmSync(cwd, { recursive: true, force: true });
    const walls = [...child.stderr.matchAll(/phase=child_wall elapsed_ms=([\d.]+)/g)].map((m) => Number(m[1]));
    return walls.length > 0 ? walls.reduce((a, b) => a + b, 0) : NaN;
  }

  /** Instructions executed by the 1T vize process (Node config children run untraced). */
  instructions(bin: Bin, setup: Setup, task: Task): number {
    const { args, cwd } = this.command({ task, setup });
    const child = spawnSync(
      "valgrind",
      ["--tool=cachegrind", "--cache-sim=no", "--cachegrind-out-file=/dev/null", bin.path, ...args],
      { cwd, env: this.env("1t"), encoding: "utf8", maxBuffer: 512 * 1024 * 1024 },
    );
    if (task === "fmt") rmSync(cwd, { recursive: true, force: true });
    const match = child.stderr.match(/I\s+refs:\s+([\d,]+)/);
    return match ? Number(match[1]?.replaceAll(",", "")) : NaN;
  }
}

const fmt = (value: number, digits = 1) => (Number.isFinite(value) ? value.toFixed(digits) : "");

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  const one = (key: string, fallback?: string) => args.get(key)?.at(-1) ?? fallback;
  const list = (key: string, fallback: string) => (one(key, fallback) ?? "").split(",").filter(Boolean);
  const corpus = resolve(one("corpus") ?? "");
  const work = resolve(one("work") ?? "");
  if (!existsSync(join(corpus, "vite.config.mjs"))) throw new Error("--corpus must be a generated corpus");
  mkdirSync(work, { recursive: true });
  const runs = Number(one("runs", "3"));
  const warmups = Number(one("warmups", "1"));
  const rounds = Number(one("rounds", "2"));
  const setups = list("setups", "vite,json").filter((s): s is Setup => s === "vite" || s === "json");
  const tasks = list("tasks", "lint,fmt").filter((t): t is Task => t === "lint" || t === "fmt");
  const threads = list("threads", "1t,max").filter((t): t is Threads => t === "1t" || t === "max");
  const valgrind = one("valgrind") === "true";
  const bins: Bin[] = (args.get("bin") ?? []).map((entry) => {
    const at = entry.indexOf("=");
    return { label: entry.slice(0, at), path: resolve(entry.slice(at + 1)) };
  });
  const bench = new Bench(corpus, work, one("perf"), await jsonConfigFor(corpus));

  const identities = new Map<string, Identity>();
  const key = (bin: Bin, setup: Setup, task: Task) => `${bin.label}\0${setup}\0${task}`;
  const specs: Spec[] = [];
  for (const bin of bins) {
    for (const setup of setups) {
      for (const task of tasks) {
        const identity = bench.identity(bin, setup, task, valgrind);
        identities.set(key(bin, setup, task), identity);
        console.error(`${bin.label} ${setup} ${task}: ${identity.summary} ${identity.outputSha256.slice(0, 12)}`);
        if (identity.summary.startsWith("rejected")) continue;
        for (const thread of threads) specs.push({ bin, setup, task, threads: thread });
      }
    }
  }

  const samples = new Map<Spec, Sample[]>(specs.map((spec) => [spec, []]));
  for (let round = 0; round < rounds; round++) {
    const ordered = round % 2 === 0 ? specs : [...specs].reverse();
    for (const spec of ordered) {
      for (let i = 0; i < warmups + runs; i++) {
        const sample = bench.run(spec);
        if (spec.task === "fmt") rmSync(sample.cwd, { recursive: true, force: true });
        if (i >= warmups) samples.get(spec)?.push(sample);
      }
      const last = samples.get(spec)?.at(-1);
      console.error(
        `${spec.bin.label} ${spec.setup} ${spec.task} ${spec.threads}: ${fmt(last?.wallMs ?? NaN)} ms wall, ${fmt(last?.cpuMs ?? NaN)} ms cpu`,
      );
    }
  }

  const rows = specs.map((spec) => {
    const list = samples.get(spec) ?? [];
    const identity = identities.get(key(spec.bin, spec.setup, spec.task));
    return {
      label: spec.bin.label,
      path: spec.bin.path,
      setup: spec.setup,
      task: spec.task,
      threads: spec.threads,
      wallMs: median(list.map((s) => s.wallMs)),
      cpuMs: median(list.map((s) => s.cpuMs)),
      taskClockMs: median(list.map((s) => s.taskClockMs)),
      maxRssMb: Math.max(...list.map((s) => s.maxRssKb)) / 1024,
      identity,
      samples: list,
    };
  });
  const rejected = [...identities.entries()].filter(([, identity]) => identity.summary.startsWith("rejected"));
  const lines = [
    "## Lint and format benchmark",
    "",
    `Runner: ${os.cpus()[0]?.model ?? "unknown"} x${os.availableParallelism()}, ${process.version}. ` +
      `Corpus: ${bench.vueFiles.length} .vue files. ${rounds} rounds x ${runs} runs (+${warmups} warmups per round), one process per run.`,
    "",
    "Wall and CPU are medians. CPU = user+sys of the process tree (GNU time); task clock = perf. Config child = wall of the Node config evaluation the build traces (VIZE_TRACE_EDITOR_PREPARATION). Instructions: valgrind cachegrind, vize process only.",
    "",
    "| Build | Setup | Task | Threads | Wall | CPU | Task clock | CPU/Wall | Peak RSS | Config child | Instructions 1T | Summary | Output SHA-256 |",
    "| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- | --- |",
    ...rows.map((row) =>
      [
        row.label,
        row.setup,
        row.task,
        row.threads,
        `${fmt(row.wallMs)} ms`,
        `${fmt(row.cpuMs, 0)} ms`,
        Number.isFinite(row.taskClockMs) ? `${fmt(row.taskClockMs)} ms` : "",
        fmt(row.cpuMs / row.wallMs, 2),
        `${fmt(row.maxRssMb, 0)} MB`,
        Number.isFinite(row.identity?.configChildWallMs ?? NaN) ? `${fmt(row.identity?.configChildWallMs ?? NaN)} ms` : "",
        Number.isFinite(row.identity?.instructions ?? NaN) ? `${fmt((row.identity?.instructions ?? NaN) / 1e9, 3)}G` : "",
        (row.identity?.summary ?? "").replaceAll("|", "\\|"),
        `\`${(row.identity?.outputSha256 ?? "").slice(0, 16)}\``,
      ]
        .join(" | ")
        .replace(/^/, "| ")
        .concat(" |"),
    ),
    "",
    ...(rejected.length > 0
      ? ["Rejected setups:", "", ...rejected.map(([k, identity]) => `- ${k.replaceAll("\0", " ")}: ${identity.summary}`), ""]
      : []),
  ];
  const out = one("out");
  if (out) writeFileSync(out, `${lines.join("\n")}\n`);
  const json = one("json");
  if (json) {
    writeFileSync(
      json,
      `${JSON.stringify({ corpus, runs, warmups, rounds, node: process.version, cpus: os.availableParallelism(), rows }, null, 2)}\n`,
    );
  }
  console.log(lines.join("\n"));
}

await main();
