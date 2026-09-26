import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";

export const mib = (bytes) => Math.round((bytes / 1024 / 1024) * 10) / 10;

// The harness is outside the server tree. Count Maestro and every descendant,
// including Corsa. RSS is summed per process, matching the TS-44 methodology.
export function createSampler(root) {
  assert.equal(process.platform, "linux", "resource acceptance requires Linux /proc");
  const ticks = Number(execFileSync("getconf", ["CLK_TCK"], { encoding: "utf8" }));
  const page = Number(execFileSync("getconf", ["PAGESIZE"], { encoding: "utf8" }));
  assert.ok(ticks > 0 && page > 0);
  // Keep the last observed own CPU time after a child exits. Subtracting only
  // the two live-tree totals would lose retired Corsa workers' accumulated CPU.
  const observedCpu = new Map();
  const sample = () => {
    const processes = new Map();
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
        });
      } catch {
        // A process can exit between readdir and stat.
      }
    }
    assert.ok(processes.has(root), "the measured Maestro server exited");
    const tree = [...processes.values()].filter(({ pid }) => {
      const visited = new Set();
      while (pid && !visited.has(pid)) {
        if (pid === root) return true;
        visited.add(pid);
        pid = processes.get(pid)?.parent;
      }
      return false;
    });
    for (const entry of tree) {
      observedCpu.set(`${entry.pid}:${entry.birth_ticks}`, entry.cpu_seconds);
    }
    return {
      rss_bytes: tree.reduce((total, entry) => total + entry.rss_bytes, 0),
      cpu_seconds: [...observedCpu.values()].reduce((total, value) => total + value, 0),
      processes: tree,
    };
  };
  let peak = sample();
  let count = 1;
  let samplingError;
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
    stop() {
      clearInterval(interval);
      if (samplingError) throw samplingError;
      return { peak, samples: count };
    },
  };
}
