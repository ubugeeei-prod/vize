import assert from "node:assert/strict";
import { test } from "node:test";
import {
  pairedColdStatistics,
  linuxCommandCpu,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-paired-stats.ts";

test("existing GNU time CPU records are exact, allow zero system, and require positive total", () => {
  const record = (user: string, system: string) =>
    `\tUser time (seconds): ${user}\n\tSystem time (seconds): ${system}\n`;
  assert.deepEqual(linuxCommandCpu(record("14.53", "1.35")), {
    userCpuMs: 14530,
    systemCpuMs: 1350,
    totalCpuMs: 15880,
    userSecondsRaw: "14.53",
    systemSecondsRaw: "1.35",
  });
  assert.deepEqual(linuxCommandCpu(record("1.00", "0.00")), {
    userCpuMs: 1000,
    systemCpuMs: 0,
    totalCpuMs: 1000,
    userSecondsRaw: "1.00",
    systemSecondsRaw: "0.00",
  });
  assert.equal(linuxCommandCpu(record("0.00", "0.01")).totalCpuMs, 10);
  for (const invalid of [
    "",
    "User time (seconds): 1.00\n",
    "System time (seconds): 1.00\n",
    record("1.00", "0.00") + "User time (seconds): 1.00\n",
    record("1.00", "0.00") + "System time (seconds): malformed\n",
    record("-1.00", "1.00"),
    record("NaN", "1.00"),
    record("1.0", "1.00"),
    record("1e2", "1.00"),
    record("1.00 trailing", "1.00"),
    record("9007199254740992.00", "0.00"),
    record("0.00", "0.00"),
  ])
    assert.throws(() => linuxCommandCpu(invalid), invalid);
});

test("block resampling keeps both sides of every complete pair and retains all samples", () => {
  const pairs = Array.from({ length: 40 }, (_, index) => ({
    base: index + 1,
    head: (index + 1) / 2,
  }));
  const original = structuredClone(pairs);
  const result = pairedColdStatistics(pairs);
  assert.equal(result.p50Ratio, 0.5);
  assert.equal(result.p95Ratio, 0.5);
  assert.deepEqual(result.p50Ratio95CI, [0.5, 0.5]);
  assert.deepEqual(result.p95Ratio95CI, [0.5, 0.5]);
  assert.deepEqual(pairs, original);
  assert.deepEqual(pairedColdStatistics(pairs), result);
  assert.throws(() => pairedColdStatistics(pairs.slice(1)));
  assert.throws(() => pairedColdStatistics([...pairs.slice(1), { base: 0, head: 1 }]));
});
