/**
 * perf around the lanes: the stat events this kernel counts, the CSV they
 * produce, `perf script` streamed through the folder, and FlameGraph output.
 */

import { spawn, spawnSync } from "node:child_process";
import { createWriteStream, existsSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { createInterface } from "node:readline";
import { Readable } from "node:stream";
import { pipeline } from "node:stream/promises";
import { createGzip } from "node:zlib";

import { ProfileFolder } from "./typecheck-profile-fold.ts";
import type { RunResult } from "./typecheck-profile-lanes.ts";
import type { Counter } from "./typecheck-profile-report.ts";
import type { LaneShape } from "./typecheck-profile-rules.ts";

const STAT_EVENTS = [
  "task-clock",
  "duration_time",
  "user_time",
  "system_time",
  "context-switches",
  "cpu-migrations",
  "page-faults",
  "major-faults",
  "cycles",
  "instructions",
  "branch-misses",
  "cache-references",
  "cache-misses",
  "L1-dcache-load-misses",
  "dTLB-load-misses",
];

/** perf's own stderr lines removed, so the host output parses as it does without perf. */
export const withoutPerfNoise = (result: RunResult): RunResult => ({
  ...result,
  stderr: result.stderr
    .split("\n")
    .filter(
      (line) =>
        !/^\[ perf (?:record|stat)|^(?:Warning|WARNING|Lowering|Couldn't)|perf-\d+\.map/u.test(
          line,
        ),
    )
    .join("\n"),
});

/** tsgo's `--pprofDir` closing lines, which follow the `--extendedDiagnostics` block. */
export const withoutPprofLines = (result: RunResult): RunResult => ({
  ...result,
  stdout: result.stdout
    .split("\n")
    .filter((line) => !/^(?:CPU|Memory) profile: /u.test(line))
    .join("\n"),
});

/** Stat events this perf and kernel accept; unsupported hardware events still read `<not supported>`. */
export function statEvents(perf: string): string[] {
  return STAT_EVENTS.filter(
    (event) => spawnSync(perf, ["stat", "-x,", "-e", event, "--", "true"]).status === 0,
  );
}

/** `perf stat -x,` rows (value, unit, event, ...); unsupported or uncounted values are null. */
export function parseStatCsv(text: string): Record<string, Counter> {
  const out: Record<string, Counter> = {};
  for (const line of text.split("\n")) {
    if (!line.trim() || line.startsWith("#")) continue;
    const [value = "", unit = "", event = ""] = line.split(",");
    const name = event.replace(/:[a-zA-Z]+$/u, "").replace(/^cpu(?:_core|_atom)?\/(.+)\/$/u, "$1");
    if (!name) continue;
    const number = Number(value);
    out[name] = { value: value.startsWith("<") || !Number.isFinite(number) ? null : number, unit };
  }
  return out;
}

/** Stream `perf script` through the folder. */
export async function foldPerfData(
  perf: string,
  data: string,
  shape: LaneShape,
): Promise<{ folder: ProfileFolder; stderr: string }> {
  const folder = new ProfileFolder(shape);
  const child = spawn(
    perf,
    [
      "script",
      "-i",
      data,
      "-F",
      "comm,pid,tid,period,event,ip,sym,dso",
      "--show-task-events",
      "--show-mmap-events",
      "--no-inline",
      "--max-stack",
      "1024",
    ],
    { stdio: ["ignore", "pipe", "pipe"] },
  );
  let stderr = "";
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk: string) => {
    if (stderr.length < 64 * 1024) stderr += chunk;
  });
  const exit = new Promise<number | null>((done, fail) => {
    child.on("error", fail);
    child.on("close", done);
  });
  for await (const line of createInterface({ input: child.stdout, crlfDelay: Infinity })) {
    folder.line(line);
  }
  const status = await exit;
  if (status !== 0) throw new Error(`perf script exited with ${status}\n${stderr}`);
  return { folder, stderr };
}

export async function writeGzip(lines: Iterable<string>, path: string): Promise<void> {
  const text = (function* () {
    for (const line of lines) yield `${line}\n`;
  })();
  await pipeline(Readable.from(text), createGzip(), createWriteStream(path));
}

/** FlameGraph's flamegraph.pl (and difffolded.pl against `diffBase`) to an SVG. */
export function flamegraph(
  scripts: string,
  input: string,
  svg: string,
  title: string,
  diffBase?: string,
): void {
  const folded = diffBase
    ? spawnSync("perl", [join(scripts, "difffolded.pl"), diffBase, input], {
        encoding: "utf8",
        maxBuffer: 1 << 30,
      }).stdout
    : readFileSync(input, "utf8");
  const result = spawnSync(
    "perl",
    [join(scripts, "flamegraph.pl"), "--title", title, "--width", "1800", "--minwidth", "0.2"],
    { input: folded, encoding: "utf8", maxBuffer: 1 << 30 },
  );
  if (result.status === 0) writeFileSync(svg, result.stdout);
  else console.warn(`flamegraph ${svg}: ${result.stderr.slice(0, 2000)}`);
}

/** V8 writes JIT symbols here for perf (`--perf-basic-prof`); the path is fixed, not TMPDIR. */
export const PERF_MAP_DIR = "/tmp";
export const perfMaps = () =>
  new Set(
    existsSync(PERF_MAP_DIR)
      ? readdirSync(PERF_MAP_DIR).filter((file) => /^perf-\d+\.map$/u.test(file))
      : [],
  );
