import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { availableParallelism, cpus, hostname } from "node:os";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { assertCampaignEnvironment } from "./template-alias-paired-11822-process.ts";

export function cpuControllerEvidence(readText = (path: string) => readFileSync(path, "utf8")) {
  const paths = [
    "/sys/fs/cgroup/cpu.max",
    "/sys/fs/cgroup/cpu/cpu.cfs_quota_us",
    "/sys/fs/cgroup/cpu/cpu.cfs_period_us",
    "/sys/fs/cgroup/cpu,cpuacct/cpu.cfs_quota_us",
    "/sys/fs/cgroup/cpu,cpuacct/cpu.cfs_period_us",
    "/sys/fs/cgroup/cpuset.cpus.effective",
    "/proc/self/cgroup",
    "/sys/fs/cgroup/cgroup.controllers",
  ];
  const files = paths.map((path) => {
    try {
      return { path, text: readText(path), unavailable: null };
    } catch (error) {
      const code = (error as NodeJS.ErrnoException).code;
      if (!["ENOENT", "EACCES", "EPERM"].includes(code ?? "")) throw error;
      return { path, text: null, unavailable: { code, message: String(error) } };
    }
  });
  const present = (path: string) => files.some((file) => file.path === path && file.text !== null);
  const membership = files.find((file) => file.path === "/proc/self/cgroup")?.text;
  return {
    meaning:
      "raw evidence; observed hierarchy or exposed interface does not prove active quota; missing or inaccessible values mean unknown, never zero or unlimited",
    observedHierarchy: {
      v2: membership === null || membership === undefined ? null : /^0::/mu.test(membership),
      v1Cpu:
        membership === null || membership === undefined
          ? null
          : membership.split("\n").some((line) => line.split(":")[1]?.split(",").includes("cpu")),
    },
    exposedCpuFiles: {
      v2: present(paths[0]),
      v1: ["cpu", "cpu,cpuacct"].some(
        (directory) =>
          present(`/sys/fs/cgroup/${directory}/cpu.cfs_quota_us`) &&
          present(`/sys/fs/cgroup/${directory}/cpu.cfs_period_us`),
      ),
    },
    files,
  };
}

export function runtimeCpuEvidence() {
  let affinity: { value: string | null; unavailable: string | null };
  try {
    affinity = {
      value:
        /^Cpus_allowed_list:\s*(.+)$/mu.exec(readFileSync("/proc/self/status", "utf8"))?.[1] ??
        null,
      unavailable: null,
    };
  } catch (error) {
    affinity = { value: null, unavailable: String(error) };
  }
  return {
    runnerName: process.env.RUNNER_NAME ?? null,
    hostname: hostname(),
    availableParallelism: availableParallelism(),
    cpus: cpus(),
    affinity,
    widthEnvironment: {
      GOMAXPROCS: process.env.GOMAXPROCS ?? null,
      RAYON_NUM_THREADS: process.env.RAYON_NUM_THREADS ?? null,
    },
    controller: cpuControllerEvidence(),
  };
}

export function sameCpuConditions(
  before: ReturnType<typeof runtimeCpuEvidence>,
  after: ReturnType<typeof runtimeCpuEvidence>,
) {
  const stable = (value: ReturnType<typeof runtimeCpuEvidence>) => [
    value.runnerName,
    value.hostname,
    value.availableParallelism,
    value.affinity,
    value.widthEnvironment,
    value.cpus.map(({ model }) => model),
  ];
  const knownQuotaChanged = before.controller.files.some((file) => {
    if (
      !/(?:cpu\.max|cpu\.cfs_quota_us|cpu\.cfs_period_us|cpuset\.cpus\.effective)$/u.test(
        file.path,
      ) ||
      file.text === null
    )
      return false;
    const current = after.controller.files.find((value) => value.path === file.path);
    return current?.text !== null && current?.text !== undefined && current.text !== file.text;
  });
  return !knownQuotaChanged && JSON.stringify(stable(before)) === JSON.stringify(stable(after));
}

export function retainCpuAfter(
  observe: () => void,
  recordFailure: (error: unknown) => void,
  hasPrimaryFailure: boolean,
) {
  try {
    observe();
  } catch (error) {
    try {
      recordFailure(error);
    } catch (recordError) {
      if (!hasPrimaryFailure) throw recordError;
    }
    if (!hasPrimaryFailure) throw error;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assertCampaignEnvironment();
  assert.equal(process.version, "v26.12.1");
  assert.equal(process.argv.length, 3);
  writeFileSync(process.argv[2], JSON.stringify(runtimeCpuEvidence(), null, 2) + "\n", {
    flag: "wx",
  });
}
