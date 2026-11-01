/** Select permitted Linux CPUs without assuming that the cgroup starts at CPU0. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

export function coldScaleAffinity(status: string, limit = 8): string {
  assert.ok(Number.isSafeInteger(limit) && limit > 0);
  const match = /^Cpus_allowed_list:\s*([0-9,-]+)$/mu.exec(status);
  assert.ok(match, "Linux process CPU affinity is unavailable");
  const cpus = new Set<number>();
  for (const part of match[1].split(",")) {
    const range = /^(\d+)(?:-(\d+))?$/u.exec(part);
    assert.ok(range, "invalid permitted CPU range");
    const first = Number(range[1]),
      last = Number(range[2] ?? range[1]);
    assert.ok(Number.isSafeInteger(first) && first >= 0 && last >= first && last <= 1_048_575);
    for (let cpu = first; cpu <= last; cpu++) {
      assert.equal(cpus.has(cpu), false, "overlapping permitted CPU ranges");
      cpus.add(cpu);
    }
  }
  assert.ok(cpus.size > 0);
  return [...cpus]
    .sort((a, b) => a - b)
    .slice(0, limit)
    .join(",");
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href)
  console.log(coldScaleAffinity(readFileSync("/proc/self/status", "utf8")));
