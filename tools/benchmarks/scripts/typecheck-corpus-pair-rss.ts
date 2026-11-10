/**
 * Process-tree memory of one host run, by sampling /proc (Linux only).
 *
 * GNU time's %M is the peak of the largest single process. This sampler runs
 * beside the driver (`node typecheck-corpus-pair-rss.ts <driver pid> <out>
 * <interval ms>`), follows every process tree the driver starts, and at each
 * tree's exit appends one JSON line: the sampled peak of the summed VmRSS of
 * the whole tree, and per process its kernel high-water mark (VmHWM, exact up
 * to the last sample) and parent. A tree is named by the $VIZE_PAIR_RSS_TAG
 * its root process inherited. `treeRss` splits a record into the host
 * process (the first process below GNU time) and the mapper processes below
 * it. The sampler's own CPU is reaped once, after the last timed run.
 */

import { spawn } from "node:child_process";
import { appendFileSync, existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

export type ProcStatus = { name: string; ppid: number; rssKb: number; hwmKb: number };
export type ProcPeak = { pid: number; ppid: number; name: string; hwmKb: number };
export const RSS_TAG = "VIZE_PAIR_RSS_TAG";
export type TreeRecord = {
  root: number;
  tag: string | null;
  samples: number;
  treePeakKb: number;
  procs: ProcPeak[];
};
export type TreeRss = {
  samples: number;
  treePeakKb: number;
  hostKb: number;
  hostName: string;
  mapperKb: number;
  mappers: number;
};

/** Name, parent and resident memory from /proc/<pid>/status; null for a zombie or a gone pid. */
export function parseProcStatus(text: string): ProcStatus | null {
  const field = (key: string) => new RegExp(`^${key}:\\s*(.*)$`, "mu").exec(text)?.[1]?.trim();
  const kb = (key: string) => {
    const value = field(key);
    return value == null ? null : Number(/^(\d+) kB$/u.exec(value)?.[1] ?? Number.NaN);
  };
  const [name, ppid, rssKb, hwmKb] = [
    field("Name"),
    Number(field("PPid")),
    kb("VmRSS"),
    kb("VmHWM"),
  ];
  if (name == null || !Number.isInteger(ppid) || rssKb == null || hwmKb == null) return null;
  if (!Number.isFinite(rssKb) || !Number.isFinite(hwmKb)) throw new Error(`bad status:\n${text}`);
  return { name, ppid, rssKb, hwmKb };
}

/** Host and mapper peaks of one tree: GNU time wrappers, then the host, then its descendants. */
export function treeRss(record: TreeRecord): TreeRss {
  const children = (pid: number) => record.procs.filter((proc) => proc.ppid === pid);
  let host = record.procs.find((proc) => proc.pid === record.root);
  while (host?.name === "time") host = children(host.pid)[0];
  const mappers: ProcPeak[] = [];
  const stack = host ? children(host.pid) : [];
  while (stack.length > 0) {
    const proc = stack.pop() as ProcPeak;
    mappers.push(proc);
    stack.push(...children(proc.pid));
  }
  return {
    samples: record.samples,
    treePeakKb: record.treePeakKb,
    hostKb: host?.hwmKb ?? 0,
    hostName: host?.name ?? "",
    mapperKb: mappers.reduce((total, proc) => total + proc.hwmKb, 0),
    mappers: mappers.length,
  };
}

function readText(path: string): string | null {
  try {
    return readFileSync(path, "utf8");
  } catch {
    return null;
  }
}

const hasChildrenFiles = existsSync(`/proc/${process.pid}/task/${process.pid}/children`);

/** Child pids of every thread of `pid` (CONFIG_PROC_CHILDREN), else by scanning every PPid. */
function childPids(pid: number): number[] {
  if (!hasChildrenFiles) {
    return readdirSync("/proc")
      .filter((entry) => /^\d+$/u.test(entry))
      .filter((entry) => parseProcStatus(readText(`/proc/${entry}/status`) ?? "")?.ppid === pid)
      .map(Number);
  }
  let tasks: string[];
  try {
    tasks = readdirSync(`/proc/${pid}/task`);
  } catch {
    return [];
  }
  return tasks.flatMap((tid) =>
    (readText(`/proc/${pid}/task/${tid}/children`) ?? "").split(" ").filter(Boolean).map(Number),
  );
}

/** $VIZE_PAIR_RSS_TAG from a NUL-separated /proc/<pid>/environ. */
export function environTag(environ: string): string | null {
  const entry = environ.split("\0").find((line) => line.startsWith(`${RSS_TAG}=`));
  return entry == null ? null : entry.slice(RSS_TAG.length + 1);
}

function sample(driver: number, out: string, intervalMs: number): void {
  type Tree = {
    tag: string | null;
    samples: number;
    treePeakKb: number;
    procs: Map<number, ProcPeak>;
  };
  const trees = new Map<number, Tree>();
  const tick = () => {
    // Orphaned: the driver is gone, so no tree of it can start.
    if (process.ppid !== driver) process.exit(0);
    const roots = new Set(childPids(driver).filter((pid) => pid !== process.pid));
    for (const root of roots) {
      const tree = trees.get(root) ?? { tag: null, samples: 0, treePeakKb: 0, procs: new Map() };
      // Read until set: between fork and exec the root still has the driver's environment.
      tree.tag ??= environTag(readText(`/proc/${root}/environ`) ?? "");
      trees.set(root, tree);
    }
    for (const [root, tree] of trees) {
      if (!roots.has(root)) {
        const { tag, samples, treePeakKb } = tree;
        const record: TreeRecord = {
          root,
          tag,
          samples,
          treePeakKb,
          procs: [...tree.procs.values()],
        };
        appendFileSync(out, `${JSON.stringify(record)}\n`);
        trees.delete(root);
        continue;
      }
      let sum = 0;
      for (const stack = [root]; stack.length > 0;) {
        const pid = stack.pop() as number;
        const status = parseProcStatus(readText(`/proc/${pid}/status`) ?? "");
        if (!status) continue;
        sum += status.rssKb;
        const seen = tree.procs.get(pid);
        tree.procs.set(pid, {
          pid,
          ppid: status.ppid,
          name: status.name,
          hwmKb: Math.max(seen?.hwmKb ?? 0, status.hwmKb, status.rssKb),
        });
        stack.push(...childPids(pid));
      }
      tree.samples++;
      tree.treePeakKb = Math.max(tree.treePeakKb, sum);
    }
  };
  setInterval(tick, intervalMs);
  writeFileSync(out, `${JSON.stringify({ ready: process.pid })}\n`);
}

const idle = new Int32Array(new SharedArrayBuffer(4));
const pause = (ms: number) => Atomics.wait(idle, 0, 0, ms);

/** Start the sampler; `take(tag)` blocks until the tree started with that tag is recorded. */
export function startRssSampler(
  out: string,
  intervalMs = 5,
): {
  take: (tag: string) => TreeRss | null;
  stop: () => void;
} {
  if (process.platform !== "linux") return { take: () => null, stop: () => {} };
  const child = spawn(
    process.execPath,
    [fileURLToPath(import.meta.url), String(process.pid), out, String(intervalMs)],
    {
      stdio: "ignore",
    },
  );
  child.unref();
  for (
    let waited = 0;
    !existsSync(out) || !readFileSync(out, "utf8").includes('"ready"');
    waited += 10
  ) {
    if (waited > 10_000) throw new Error("the RSS sampler did not start");
    pause(10);
  }
  const records = () =>
    readFileSync(out, "utf8")
      .split("\n")
      .filter((line) => line.startsWith('{"root"'))
      .map((line) => JSON.parse(line) as TreeRecord);
  return {
    take(tag) {
      for (let waited = 0; waited < 5_000; waited += 5) {
        const record = records().find((entry) => entry.tag === tag);
        if (record) return treeRss(record);
        pause(5);
      }
      return null;
    },
    stop: () => void child.kill(),
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const [driver, out, interval] = process.argv.slice(2);
  sample(Number(driver), out as string, Number(interval ?? 5));
}
