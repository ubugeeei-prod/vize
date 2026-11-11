/**
 * Cold wall of the corpus pair: every timed run is a fresh host process with
 * no tsbuildinfo (`incremental: false`), XDG_CACHE_HOME and TMPDIR in a new
 * empty directory, Node's compile cache and Vize's config cache off. Each
 * run's wall splits into what the host's `--extendedDiagnostics` attributes
 * (config, parse with the mapper wait inside it, bind, check) and what lies
 * outside its Total (launcher, process start, report, teardown). Memory comes
 * from the /proc sampler (typecheck-corpus-pair-rss.ts).
 */

import { mkdirSync, readdirSync, rmSync } from "node:fs";
import { join } from "node:path";

import { runTypecheckCommand } from "./typecheck-command.mjs";
import {
  type HostRun,
  PAIR_HOSTS,
  type PairHost,
  type PairHostBins,
  hostRun,
} from "./typecheck-corpus-pair-host.ts";
import { type GnuTime, type StatEntry, median } from "./typecheck-corpus-pair-parse.ts";
import { RSS_TAG, type TreeRss, startRssSampler } from "./typecheck-corpus-pair-rss.ts";

export const TARGET_MS = 500;
export type ColdRun = HostRun & { rss: TreeRss | null; cacheFiles: number };
export type ColdRunRecord = Omit<ColdRun, "diagnostics">;
export type ColdLane = {
  corpus: string;
  host: PairHost["id"];
  variant: string;
  runs: ColdRunRecord[];
};
/** Generated reply shape of a lane, from its probe run. */
export type ReplyCounts = { replyBytes: number; spans: number; directives: number };
export type ColdResults = {
  runs: number;
  runner: string;
  corpora: Array<{ id: string; label: string; sfcs: number }>;
  lanes: ColdLane[];
  startup: Array<{ host: PairHost["id"]; wallMs: number[] }>;
  replies: Record<string, ReplyCounts>;
  notes: string[];
};

const countFiles = (dir: string): number =>
  readdirSync(dir, { withFileTypes: true }).reduce(
    (total, entry) =>
      total + (entry.isDirectory() ? countFiles(join(entry.parentPath, entry.name)) : 1),
    0,
  );

/** A hostRun with fresh cache directories and a sampled process tree. */
export function coldRunner(work: string): {
  run: (
    command: string[],
    cwd: string,
    tsconfig: string,
    timeFile: string,
    env?: Record<string, string>,
  ) => ColdRun;
  stop: () => void;
} {
  mkdirSync(join(work, "cold"), { recursive: true });
  const sampler = startRssSampler(join(work, "cold", "rss.jsonl"));
  let count = 0;
  return {
    run(command, cwd, tsconfig, timeFile, env = {}) {
      const tag = `cold-${count++}`;
      const fresh = join(work, "cold", tag);
      for (const dir of ["cache", "tmp"]) mkdirSync(join(fresh, dir), { recursive: true });
      const result = hostRun(command, cwd, tsconfig, timeFile, {
        ...env,
        XDG_CACHE_HOME: join(fresh, "cache"),
        TMPDIR: join(fresh, "tmp"),
        NODE_DISABLE_COMPILE_CACHE: "1",
        VIZE_CONFIG_CACHE: "0",
        [RSS_TAG]: tag,
      });
      const cacheFiles = countFiles(fresh);
      rmSync(fresh, { recursive: true, force: true });
      return { ...result, rss: sampler.take(tag), cacheFiles };
    },
    stop: sampler.stop,
  };
}

/** `<host> --version` walls: the launcher and process start no check can go below. */
export function startupRuns(
  bins: PairHostBins,
  hosts: readonly PairHost[],
  runs: number,
  cwd: string,
): ColdResults["startup"] {
  return hosts.map((host) => ({
    host: host.id,
    wallMs: Array.from({ length: runs }, () => {
      const result = runTypecheckCommand(bins[host.bin], [...host.args, "--version"], { cwd }) as {
        ms: number;
        status: number | null;
      };
      if (result.status !== 0) throw new Error(`${host.label} --version exited ${result.status}`);
      return result.ms;
    }),
  }));
}

/** Milliseconds of one `--extendedDiagnostics` time; null when the host does not report it. */
export function statMs(stats: StatEntry[], key: string): number | null {
  const value = stats.find(([name]) => name === key)?.[1];
  const match = value == null ? null : /^(\d+(?:\.\d+)?)s$/u.exec(value);
  return match ? Number(match[1]) * 1000 : null;
}

export type ColdSummary = {
  corpus: string;
  host: PairHost["id"];
  variant: string;
  wallMs: number;
  outsideTotalMs: number | null;
  configMs: number | null;
  parseMs: number | null;
  waitMs: number | null;
  bindMs: number | null;
  checkMs: number | null;
  totalMs: number | null;
  cpuS: number;
  treeMb: number | null;
  hostMb: number | null;
  mapperMb: number | null;
  mappers: number | null;
  largestMb: number;
  cacheFiles: number;
};

const STATS = {
  configMs: "Config time",
  parseMs: "Parse time",
  waitMs: "Content mapper request wait time",
  bindMs: "Bind time",
  checkMs: "Check time",
  totalMs: "Total time",
} as const;

export function summarizeCold(lane: ColdLane): ColdSummary {
  const of = (pick: (run: ColdRunRecord) => number | null | undefined): number | null => {
    const values = lane.runs.map(pick).filter((value): value is number => value != null);
    return values.length === lane.runs.length && values.length > 0 ? median(values) : null;
  };
  const stats = Object.fromEntries(
    Object.entries(STATS).map(([field, key]) => [field, of((run) => statMs(run.stats, key))]),
  ) as Record<keyof typeof STATS, number | null>;
  const mb = (kb: number | null) => (kb == null ? null : kb / 1024);
  return {
    corpus: lane.corpus,
    host: lane.host,
    variant: lane.variant,
    wallMs: of((run) => run.wallMs) as number,
    outsideTotalMs: of((run) => {
      const total = statMs(run.stats, STATS.totalMs);
      return total == null ? null : run.wallMs - total;
    }),
    ...stats,
    cpuS: (of((run) => run.cpuMs) as number) / 1000,
    treeMb: mb(of((run) => run.rss?.treePeakKb)),
    hostMb: mb(of((run) => run.rss?.hostKb)),
    mapperMb: mb(of((run) => run.rss?.mapperKb)),
    mappers: of((run) => run.rss?.mappers),
    largestMb: (of((run) => (run.time as GnuTime).maxRssKb) as number) / 1024,
    cacheFiles: Math.max(...lane.runs.map((run) => run.cacheFiles)),
  };
}

/** The largest named part of one lane's wall; parse is counted without the mapper wait inside it. */
export function dominantPart(lane: ColdSummary): { part: string; ms: number } | null {
  const parts: Array<[string, number | null]> = [
    ["outside Total (startup + teardown)", lane.outsideTotalMs],
    ["mapper wait", lane.waitMs],
    [
      "parse without mapper wait",
      lane.parseMs == null ? null : Math.max(0, lane.parseMs - (lane.waitMs ?? 0)),
    ],
    ["bind", lane.bindMs],
    ["check", lane.checkMs],
    ["config", lane.configMs],
  ];
  const known = parts.filter((entry): entry is [string, number] => entry[1] != null);
  if (known.length === 0) return null;
  const [part, ms] = known.reduce((best, entry) => (entry[1] > best[1] ? entry : best));
  return { part, ms };
}

export const hostLabel = (id: string) => PAIR_HOSTS.find((host) => host.id === id)?.label ?? id;
