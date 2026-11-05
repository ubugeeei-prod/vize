import assert from "node:assert/strict";
import { test } from "node:test";
import {
  cpuControllerEvidence,
  sameCpuConditions,
  retainCpuAfter,
  type runtimeCpuEvidence,
} from "./template-alias-paired-11822-cpu.ts";

const absent = () => {
  throw Object.assign(new Error("not exposed"), { code: "ENOENT" });
};

void test("a runner without a v2 CPU file preserves v1 evidence and unknown fields", () => {
  const result = cpuControllerEvidence((path) => {
    if (path === "/sys/fs/cgroup/cpu/cpu.cfs_quota_us") return "3200000\n";
    if (path === "/sys/fs/cgroup/cpu/cpu.cfs_period_us") return "100000\n";
    return absent();
  });
  assert.equal(result.files[0].text, null);
  assert.equal(result.files[0].unavailable?.code, "ENOENT");
  assert.equal(result.files[1].text, "3200000\n");
  assert.equal(result.files[2].text, "100000\n");
  assert.deepEqual(result.exposedCpuFiles, { v2: false, v1: true });
});

void test("v2 hierarchy can be observed even when its root does not expose cpu.max", () => {
  const result = cpuControllerEvidence((path) =>
    path === "/proc/self/cgroup" ? "0::/\n" : absent(),
  );
  assert.deepEqual(result.observedHierarchy, { v2: true, v1Cpu: false });
  assert.deepEqual(result.exposedCpuFiles, { v2: false, v1: false });
  const limited = cpuControllerEvidence((path) =>
    path === "/sys/fs/cgroup/cpu.max" ? "3200000 100000\n" : absent(),
  );
  assert.equal(limited.files[0].text, "3200000 100000\n");
  assert.equal(limited.exposedCpuFiles.v2, true);
});

void test("entirely hidden controllers stay explicitly unknown and real I/O errors still fail", () => {
  const result = cpuControllerEvidence(absent);
  assert.ok(
    result.files.every((file) => file.text === null && file.unavailable?.code === "ENOENT"),
  );
  assert.deepEqual(result.observedHierarchy, { v2: null, v1Cpu: null });
  assert.deepEqual(result.exposedCpuFiles, { v2: false, v1: false });
  const denied = cpuControllerEvidence(() => {
    throw Object.assign(new Error("denied"), { code: "EACCES" });
  });
  assert.ok(
    denied.files.every((file) => file.text === null && file.unavailable?.code === "EACCES"),
  );
  assert.throws(
    () =>
      cpuControllerEvidence(() => {
        throw Object.assign(new Error("I/O failure"), { code: "EIO" });
      }),
    /I\/O failure/,
  );
});

void test("stable CPU conditions compare runner and affinity while dynamic CPU speed/counters remain evidence", () => {
  const before = {
    runnerName: "runner",
    hostname: "host",
    availableParallelism: 32,
    affinity: { value: "0-31", unavailable: null },
    widthEnvironment: { GOMAXPROCS: null, RAYON_NUM_THREADS: null },
    cpus: [{ model: "CPU", speed: 1000, times: { user: 1 } }],
    controller: cpuControllerEvidence(absent),
  } as ReturnType<typeof runtimeCpuEvidence>;
  const after = structuredClone(before);
  after.cpus[0].speed = 2000;
  after.cpus[0].times.user = 2;
  assert.equal(sameCpuConditions(before, after), true);
  after.affinity.value = "0-15";
  assert.equal(sameCpuConditions(before, after), false);
  after.affinity.value = "0-31";
  after.hostname = "other";
  assert.equal(sameCpuConditions(before, after), false);
  after.hostname = "host";
  before.controller = cpuControllerEvidence((path) =>
    path === "/sys/fs/cgroup/cpu.max" ? "3200000 100000\n" : absent(),
  );
  after.controller = cpuControllerEvidence((path) =>
    path === "/sys/fs/cgroup/cpu.max" ? "100000 100000\n" : absent(),
  );
  assert.equal(sameCpuConditions(before, after), false);
  after.controller = cpuControllerEvidence(absent);
  assert.equal(
    sameCpuConditions(before, after),
    true,
    "unavailable quota alone cannot reject the observation",
  );
});

void test("a late CPU read failure is recorded without replacing the original measurement failure", () => {
  const primary = new Error("original semantic failure"),
    telemetry = new Error("CPU EIO");
  const recorded: unknown[] = [];
  let result: unknown;
  try {
    try {
      throw primary;
    } finally {
      retainCpuAfter(
        () => {
          throw telemetry;
        },
        (error) => {
          recorded.push(error);
        },
        true,
      );
    }
  } catch (error) {
    result = error;
  }
  assert.equal(result, primary);
  assert.deepEqual(recorded, [telemetry]);
  assert.throws(
    () =>
      retainCpuAfter(
        () => {
          throw telemetry;
        },
        () => {},
        false,
      ),
    (error) => error === telemetry,
  );
});
