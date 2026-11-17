import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

import {
  isScenario,
  parseConfig,
  parseTimeOutput,
} from "../../tools/support/github/rust-frontend-threads-bench.ts";
import {
  percentile,
  summarize,
  type Report,
} from "../../tools/support/github/rust-frontend-threads-summary.ts";

void test("frontend thread configurations select the pinned stable or the nightly frontend", () => {
  assert.deepEqual(parseConfig("stable"), { id: "stable", toolchain: "1.99.1" });
  assert.deepEqual(parseConfig("j8"), {
    id: "nightly-j8",
    toolchain: "nightly-2026-11-16",
    jobsFrontend: 8,
  });
  assert.throws(() => parseConfig("nightly"), /unknown configuration/);
});

void test("GNU and BSD time reports yield CPU seconds and peak RSS in MiB", () => {
  const gnu = [
    "\tUser time (seconds): 12.50",
    "\tSystem time (seconds): 1.50",
    "\tMaximum resident set size (kbytes): 2097152",
  ].join("\n");
  assert.deepEqual(parseTimeOutput(gnu, true), { cpuSeconds: 14, maxRssMiB: 2048 });
  const bsd =
    "        3.00 real         2.25 user         0.75 sys\n  1073741824  maximum resident set size\n";
  assert.deepEqual(parseTimeOutput(bsd, false), { cpuSeconds: 3, maxRssMiB: 1024 });
});

void test("summaries report nearest-rank percentiles against stable and one frontend thread", () => {
  assert.equal(percentile([3, 1, 2], 0.5), 2);
  assert.equal(percentile([3, 1, 2], 0.95), 3);
  const sample = (config: string, wallSeconds: number, status = 0) => ({
    scenario: "crate:vize_canon",
    measure: "vize_canon check",
    config,
    round: 0,
    wallSeconds,
    cpuSeconds: wallSeconds,
    maxRssMiB: 1000,
    status,
  });
  const report: Report = {
    host: { cpus: 32, memoryGiB: 113, platform: "linux-x64", loadAverage: [1, 1, 1] },
    samples: [
      sample("nightly-j8", 2),
      sample("stable", 4),
      sample("nightly-j1", 5),
      sample("nightly-j8", 100, 101),
    ],
  };
  const rows = summarize([report])
    .split("\n")
    .filter((line) => line.startsWith("| vize_canon check"));
  assert.deepEqual(rows, [
    "| vize_canon check | stable | 1 | 4.0 | 4.0 |  |  | 4.0 | 1000 |",
    "| vize_canon check | nightly-j1 | 1 | 5.0 | 5.0 | 25.0% |  | 5.0 | 1000 |",
    "| vize_canon check | nightly-j8 | 1 | 2.0 | 2.0 | -50.0% | -60.0% | 2.0 | 1000 |",
  ]);
});

void test("the frontend benchmark is manual evidence and never gates or releases", () => {
  const workflow = parse(
    readFileSync(
      new URL("../../.github/workflows/rust-frontend-threads-bench.yml", import.meta.url),
      "utf8",
    ),
  );
  assert.deepEqual(Object.keys(workflow.on), ["workflow_dispatch"]);
  assert.deepEqual(workflow.permissions, { contents: "read" });
  const include = workflow.jobs.measure.strategy.matrix.include as Array<{
    scenario: string;
    configs: string;
  }>;
  for (const entry of include) {
    assert.equal(isScenario(entry.scenario), true, entry.scenario);
    for (const config of entry.configs.split(",")) assert.ok(parseConfig(config));
  }
  const determinism = include.find((entry) => entry.scenario === "determinism");
  assert.deepEqual(determinism?.configs.split(","), ["j8", "j8", "j1"]);
});
