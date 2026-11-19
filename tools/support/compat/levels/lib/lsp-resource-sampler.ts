import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";

export const mib = (bytes: number): number => Math.round((bytes / 1024 / 1024) * 10) / 10;

export interface ProcessEntry {
  pid: number;
  parent: number;
  birth_ticks: number;
  rss_bytes: number;
  cpu_seconds: number;
  command: string;
  executable: string | null;
}

export interface TreeSample {
  rss_bytes: number;
  cpu_seconds: number;
  processes: ProcessEntry[];
}

export interface ObservedProcess {
  pid: number;
  birth_ticks: number;
  first_sample: number;
  last_sample: number;
  samples: number;
  parent_pids: number[];
  commands: string[];
  executables: string[];
  peak_rss_bytes: number;
  last_rss_bytes: number;
  cpu_seconds: number;
}

export interface ProcessHistory {
  observe(sample: number, processes: readonly ProcessEntry[]): void;
  entries(): ObservedProcess[];
}

export interface Sampler {
  sample: () => TreeSample;
  observedProcesses(): ObservedProcess[];
  stop(): { peak: TreeSample; samples: number; observed_processes: ObservedProcess[] };
}

const processIdentity = (entry: Pick<ProcessEntry, "pid" | "birth_ticks">): string =>
  `${entry.pid}:${entry.birth_ticks}`;

const appendUnique = <T>(values: T[], value: T): void => {
  if (!values.includes(value)) values.push(value);
};

/** Run-wide evidence. Retired children remain distinct from reused PIDs. */
export function createProcessHistory(): ProcessHistory {
  const observed = new Map<string, ObservedProcess>();
  let lastSample = -1;
  return {
    observe(sample, processes) {
      assert.ok(Number.isSafeInteger(sample) && sample > lastSample, "sample index must increase");
      lastSample = sample;
      for (const process of processes) {
        const identity = processIdentity(process);
        const previous = observed.get(identity);
        if (!previous) {
          observed.set(identity, {
            pid: process.pid,
            birth_ticks: process.birth_ticks,
            first_sample: sample,
            last_sample: sample,
            samples: 1,
            parent_pids: [process.parent],
            commands: [process.command],
            executables: process.executable ? [process.executable] : [],
            peak_rss_bytes: process.rss_bytes,
            last_rss_bytes: process.rss_bytes,
            cpu_seconds: process.cpu_seconds,
          });
          continue;
        }
        previous.last_sample = sample;
        previous.samples++;
        appendUnique(previous.parent_pids, process.parent);
        appendUnique(previous.commands, process.command);
        if (process.executable) appendUnique(previous.executables, process.executable);
        previous.peak_rss_bytes = Math.max(previous.peak_rss_bytes, process.rss_bytes);
        previous.last_rss_bytes = process.rss_bytes;
        previous.cpu_seconds = Math.max(previous.cpu_seconds, process.cpu_seconds);
      }
    },
    entries: () =>
      [...observed.values()]
        .map((entry) => ({
          ...entry,
          parent_pids: [...entry.parent_pids],
          commands: [...entry.commands],
          executables: [...entry.executables],
        }))
        .sort((left, right) => left.birth_ticks - right.birth_ticks || left.pid - right.pid),
  };
}

/** Select only the current root identity and its live descendants. */
export function ownedProcessTree(
  processes: ReadonlyMap<number, ProcessEntry>,
  root: Pick<ProcessEntry, "pid" | "birth_ticks">,
): ProcessEntry[] {
  const currentRoot = processes.get(root.pid);
  assert.ok(currentRoot, "the measured Maestro server exited");
  assert.equal(currentRoot.birth_ticks, root.birth_ticks, "the measured Maestro PID was reused");
  return [...processes.values()].filter((entry) => {
    let pid: number | undefined = entry.pid;
    const visited = new Set<number>();
    while (pid && !visited.has(pid)) {
      if (pid === root.pid) return true;
      visited.add(pid);
      pid = processes.get(pid)?.parent;
    }
    return false;
  });
}

// The harness is outside the server tree. Count Maestro and every descendant,
// including Corsa. RSS is summed per process, matching the TS-44 methodology.
export function createSampler(root: number): Sampler {
  assert.equal(process.platform, "linux", "resource acceptance requires Linux /proc");
  const ticks = Number(execFileSync("getconf", ["CLK_TCK"], { encoding: "utf8" }));
  const page = Number(execFileSync("getconf", ["PAGESIZE"], { encoding: "utf8" }));
  assert.ok(ticks > 0 && page > 0);
  // Keep the last observed own CPU time after a child exits. Subtracting only
  // the two live-tree totals would lose retired Corsa workers' accumulated CPU.
  const observedCpu = new Map<string, number>();
  const history = createProcessHistory();
  let rootIdentity: Pick<ProcessEntry, "pid" | "birth_ticks"> | undefined;
  let sampleIndex = 0;
  const sample = (): TreeSample => {
    const processes = new Map<number, ProcessEntry>();
    for (const pid of fs.readdirSync("/proc").filter((entry) => /^\d+$/.test(entry))) {
      try {
        const stat = fs.readFileSync(`/proc/${pid}/stat`, "utf8");
        const fields = stat
          .slice(stat.lastIndexOf(") ") + 2)
          .trim()
          .split(/\s+/);
        processes.set(Number(pid), {
          pid: Number(pid),
          parent: Number(fields[1]),
          birth_ticks: Number(fields[19]),
          rss_bytes: Math.max(0, Number(fields[21])) * page,
          // Live children's own times are counted separately; adding cutime /
          // cstime here would double count already reaped descendants' CPU.
          cpu_seconds: (Number(fields[11]) + Number(fields[12])) / ticks,
          command: stat.slice(stat.indexOf("(") + 1, stat.lastIndexOf(") ")),
          executable: null,
        });
      } catch {
        // A process can exit between readdir and stat.
      }
    }
    const currentRoot = processes.get(root);
    assert.ok(currentRoot, "the measured Maestro server exited");
    rootIdentity ??= { pid: root, birth_ticks: currentRoot.birth_ticks };
    const tree = ownedProcessTree(processes, rootIdentity).map((entry) => {
      try {
        return { ...entry, executable: fs.realpathSync(`/proc/${entry.pid}/exe`) };
      } catch {
        // The stat row remains valid measurement evidence when a worker exits
        // between the table scan and executable identity lookup.
        return entry;
      }
    });
    for (const entry of tree) {
      observedCpu.set(processIdentity(entry), entry.cpu_seconds);
    }
    history.observe(sampleIndex++, tree);
    return {
      rss_bytes: tree.reduce((total, entry) => total + entry.rss_bytes, 0),
      cpu_seconds: [...observedCpu.values()].reduce((total, value) => total + value, 0),
      processes: tree,
    };
  };
  let peak = sample();
  let count = 1;
  let samplingError: unknown;
  const interval = setInterval(() => {
    try {
      const current = sample();
      count++;
      if (current.rss_bytes > peak.rss_bytes) peak = current;
    } catch (error) {
      samplingError = error;
    }
  }, 50);
  return {
    sample,
    observedProcesses: () => history.entries(),
    stop() {
      clearInterval(interval);
      if (samplingError) throw samplingError;
      return { peak, samples: count, observed_processes: history.entries() };
    },
  };
}
