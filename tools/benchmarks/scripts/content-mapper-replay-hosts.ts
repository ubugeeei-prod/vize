/**
 * Real host runs for content-mapper-replay: `tsc` and `tsrs --fast` check
 * each corpus through each mapper, alternating mappers per run, and report
 * what the host waited for and how many transforms it kept in flight.
 *
 *   node tools/benchmarks/scripts/content-mapper-replay-hosts.ts \
 *     --work /tmp/replay --corpora misskey,n8n --hosts tsc,tsrs \
 *     --tsc <bin> --tsrs <bin> --mapper base=/tmp/vize-base --mapper head=/tmp/vize-head \
 *     --ready --runs 5 --out /tmp/replay-out
 *
 * `--work` holds the projects content-mapper-replay-record.ts prepared. Each
 * run points the project's `vize` package at content-mapper-replay-proxy.ts
 * in front of the mapper, so every frame is timed where the host sees it.
 * Reported per corpus, host and mapper (median of runs): host wall, its
 * `--extendedDiagnostics` parse time, content-mapper request wait, summed
 * transform time and mapper initialization, and from the proxy timeline the
 * mean and peak transforms in flight, the share of the transform window at
 * the worker count, the first transform's latency, and how often the host
 * let every transform finish before sending the next (the mapper drops its
 * shared imported-type snapshot when idle) with the latency of the next one.
 */

import { spawnSync } from "node:child_process";
import { availableParallelism } from "node:os";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

import { median } from "./content-mapper-pair-report.ts";
import { parseMapper } from "./content-mapper-replay.ts";
import type { ReplayMapper } from "./content-mapper-replay-report.ts";

const PROXY = join(dirname(fileURLToPath(import.meta.url)), "content-mapper-replay-proxy.ts");

export type TimelineEvent = { t: number; dir: "req" | "rep"; id: string; method?: string };

export type Concurrency = {
  transforms: number;
  /** Mean transforms in flight over the window from first request to last reply. */
  meanInFlight: number;
  peakInFlight: number;
  /** Share of the window with at least `workers` transforms in flight. */
  atWorkersShare: number;
  /** Latency of the first transform the host sent. */
  firstLatencyMs: number;
  p50LatencyMs: number;
  windowMs: number;
  /** Times every sent transform had been answered before the host sent another. */
  idleGaps: number;
  /** Median latency of the transforms sent after such a gap. */
  postIdleLatencyMs: number;
};

/** In-flight transforms over a proxy timeline. */
export function concurrency(events: readonly TimelineEvent[], workers: number): Concurrency {
  const sent = new Map<string, number>();
  const steps: [number, number][] = [];
  const latencies: number[] = [];
  const postIdle = new Set<string>();
  const postIdleLatencies: number[] = [];
  let firstId: string | undefined;
  let firstLatencyMs = NaN;
  let idleGaps = 0;
  for (const event of events) {
    if (event.dir === "req" && event.method === "transform") {
      firstId ??= event.id;
      if (sent.size === 0 && latencies.length > 0) {
        idleGaps++;
        postIdle.add(event.id);
      }
      sent.set(event.id, event.t);
      steps.push([event.t, 1]);
    } else if (event.dir === "rep" && sent.has(event.id)) {
      const latency = event.t - (sent.get(event.id) ?? event.t);
      if (event.id === firstId) firstLatencyMs = latency;
      if (postIdle.has(event.id)) postIdleLatencies.push(latency);
      latencies.push(latency);
      sent.delete(event.id);
      steps.push([event.t, -1]);
    }
  }
  steps.sort((a, b) => a[0] - b[0] || b[1] - a[1]);
  let level = 0;
  let peak = 0;
  let area = 0;
  let atWorkers = 0;
  const first = steps[0]?.[0] ?? 0;
  let previous = first;
  for (const [t, delta] of steps) {
    area += level * (t - previous);
    if (level >= workers) atWorkers += t - previous;
    level += delta;
    peak = Math.max(peak, level);
    previous = t;
  }
  const windowMs = previous - first;
  return {
    transforms: latencies.length,
    meanInFlight: windowMs > 0 ? area / windowMs : 0,
    peakInFlight: peak,
    atWorkersShare: windowMs > 0 ? atWorkers / windowMs : 0,
    firstLatencyMs,
    p50LatencyMs: median(latencies),
    windowMs,
    idleGaps,
    postIdleLatencyMs: median(postIdleLatencies),
  };
}

/** Seconds from a host's `--extendedDiagnostics` report. */
export function hostStats(output: string): Record<string, number> {
  const seconds = (pattern: RegExp) => {
    const value = pattern.exec(output)?.[1];
    return value === undefined ? NaN : Number(value);
  };
  return {
    parse: seconds(/^Parse time:\s+([\d.]+)s/m),
    mapperWait: seconds(/^Content mapper request wait time:\s+([\d.]+)s/m),
    transformSum: seconds(/^vize@[^ ]+ transform time:\s+([\d.]+)s/m),
    initialization: seconds(/^vize@[^ ]+ initialization time:\s+([\d.]+)s/m),
    total: seconds(/^Total time:\s+([\d.]+)s/m),
  };
}

type Sample = Record<string, number>;

function pointMapper(project: string, mapper: ReplayMapper, timeline: string): void {
  const manifest = join(project, "node_modules", "vize", "package.json");
  const json = JSON.parse(readFileSync(manifest, "utf8")) as {
    typescript: { contentMapper: { exec: string[] } };
  };
  json.typescript.contentMapper.exec = [
    process.execPath,
    PROXY,
    "-",
    timeline,
    "--",
    mapper.command,
    ...mapper.args,
  ];
  writeFileSync(manifest, `${JSON.stringify(json)}\n`);
}

function readTimeline(path: string): TimelineEvent[] {
  return readFileSync(path, "utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line) as TimelineEvent);
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      work: { type: "string" },
      corpora: { type: "string" },
      hosts: { type: "string", default: "tsc,tsrs" },
      tsc: { type: "string" },
      tsrs: { type: "string" },
      mapper: { type: "string", multiple: true },
      ready: { type: "boolean", default: false },
      runs: { type: "string", default: "5" },
      out: { type: "string" },
    },
  });
  if (!values.work || !values.corpora || !values.out || !values.mapper?.length) {
    throw new Error("usage: --work <dir> --corpora a,b --mapper id=path ... --out <dir>");
  }
  const work = resolve(values.work);
  const out = resolve(values.out);
  mkdirSync(out, { recursive: true });
  const mappers = values.mapper.map((spec) => parseMapper(spec, values.ready));
  const hosts = values.hosts.split(",").map((host) => {
    const bin = host === "tsc" ? values.tsc : values.tsrs;
    if (!bin) throw new Error(`--${host} wants the host binary`);
    const args = [
      "-p",
      "tsconfig.vize.json",
      "--noEmit",
      "--runExternalCode",
      "--extendedDiagnostics",
      "--pretty",
      "false",
    ];
    return { id: host, bin: resolve(bin), args: host === "tsrs" ? [...args, "--fast"] : args };
  });
  const workers = availableParallelism();
  const rows: string[] = [];
  const json: unknown[] = [];
  for (const corpus of values.corpora.split(",")) {
    const project = join(work, corpus);
    const manifest = join(project, "node_modules", "vize", "package.json");
    const original = readFileSync(manifest, "utf8");
    const samples = new Map<string, Sample[]>();
    try {
      for (let run = 0; run < Number(values.runs); run++) {
        const order = run % 2 === 0 ? mappers : [...mappers].reverse();
        for (const host of hosts) {
          for (const mapper of order) {
            const timeline = join(work, `${corpus}-${host.id}-${mapper.id}.jsonl`);
            rmSync(timeline, { force: true });
            pointMapper(project, mapper, timeline);
            const start = performance.now();
            const result = spawnSync(host.bin, host.args, {
              cwd: project,
              encoding: "utf8",
              maxBuffer: 512 * 1024 * 1024,
            });
            const wallMs = performance.now() - start;
            const stats = hostStats(result.stdout);
            if (!Number.isFinite(stats.total)) {
              throw new Error(`${host.id} on ${corpus} printed no statistics: ${result.stderr}`);
            }
            const flight = concurrency(readTimeline(timeline), workers);
            const key = `${host.id}\t${mapper.id}`;
            const list = samples.get(key) ?? [];
            list.push({ wallMs, ...stats, ...flight });
            samples.set(key, list);
          }
        }
      }
    } finally {
      writeFileSync(manifest, original);
    }
    for (const [key, list] of samples) {
      const [host, mapper] = key.split("\t");
      const pick = (name: string) => median(list.map((sample) => sample[name] ?? NaN));
      const row = {
        corpus,
        host,
        mapper,
        runs: list.length,
        wallMs: pick("wallMs"),
        parse: pick("parse"),
        mapperWait: pick("mapperWait"),
        transformSum: pick("transformSum"),
        initialization: pick("initialization"),
        total: pick("total"),
        meanInFlight: pick("meanInFlight"),
        peakInFlight: pick("peakInFlight"),
        atWorkersShare: pick("atWorkersShare"),
        firstLatencyMs: pick("firstLatencyMs"),
        p50LatencyMs: pick("p50LatencyMs"),
        windowMs: pick("windowMs"),
        idleGaps: pick("idleGaps"),
        postIdleLatencyMs: pick("postIdleLatencyMs"),
      };
      json.push(row);
      const f = (value: number, digits = 3) =>
        Number.isFinite(value) ? value.toFixed(digits) : "-";
      rows.push(
        `| ${corpus} | ${host} | ${mapper} | ${f(row.wallMs, 0)} | ${f(row.parse)} | ${f(row.mapperWait)} | ${f(row.transformSum)} | ${f(row.initialization)} | ${f(row.total)} | ${f(row.meanInFlight, 1)} / ${f(row.peakInFlight, 0)} | ${f(100 * row.atWorkersShare, 0)}% | ${f(row.firstLatencyMs, 1)} | ${f(row.p50LatencyMs, 2)} | ${f(row.windowMs, 0)} | ${f(row.idleGaps, 0)} / ${f(row.postIdleLatencyMs, 2)} |`,
      );
    }
    console.error(`${corpus}: host runs done`);
  }
  const markdown = [
    `### Host runs (median of ${values.runs}, ${workers} worker threads)`,
    "",
    "Wall is the host process; the next five columns are its `--extendedDiagnostics` seconds (transform time sums every request's latency). In flight, the window and latencies come from the proxy timeline; the window runs from the first transform request to the last reply.",
    "",
    "| corpus | host | mapper | wall ms | parse s | mapper wait s | transform sum s | mapper init s | total s | in flight mean / peak | at workers | first transform ms | p50 transform ms | window ms | idle gaps / p50 ms after one |",
    "|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|",
    ...rows,
    "",
  ].join("\n");
  writeFileSync(join(out, "hosts.json"), JSON.stringify(json, null, 2));
  writeFileSync(join(out, "hosts.md"), markdown);
  process.stdout.write(markdown);
}

if (import.meta.main) await main();
