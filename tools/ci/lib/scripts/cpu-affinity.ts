import assert from "node:assert/strict";
import { textFile } from "./files.ts";

export function linuxCpuAffinity(): number[] {
  // /proc reports the same allowed CPU set as sched_getaffinity on Linux, including gaps.
  const list = /^Cpus_allowed_list:\s*([0-9,-]+)$/mu.exec(textFile("/proc/self/status"));
  assert.ok(list, "the runner must expose its CPU affinity");
  return list[1].split(",").flatMap((span) => {
    const [first, last = first] = span.split("-").map(Number);
    return Array.from({ length: last - first + 1 }, (_, index) => first + index);
  });
}
