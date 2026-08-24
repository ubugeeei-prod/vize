// Platform contract for exact Davinci allocation-byte peaks.
//
// Allocation call counts stay machine-independent. Peak requested bytes are
// exact only on one target OS, so reports identify their producer and the
// budget registry must explicitly cover every supported platform.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const comparePath = path.join(repoRoot, "tools", "davinci", "bench-compare.mjs");
const benchId = "ricalco_lower_vfor_three_aliases";

function budgetText(peaks: string): string {
  return (
    "[bench]\n" +
    `${benchId} = { wall_p50_ns = 0, allocs = 10, ` +
    "rss_peak_bytes = 0, wall_tolerance = 0.05 }\n" +
    `[allocation_peak]\n${benchId} = { ${peaks} }\n`
  );
}

function report(platform: string | undefined, peak: number): Record<string, unknown> {
  return {
    bench_id: benchId,
    fixture: "synthetic:v-for-three-aliases",
    ...(platform == null ? {} : { platform }),
    wall_ns: { p50: 800, p95: 900 },
    allocs: 10,
    alloc_bytes_peak: peak,
    rss_peak_bytes: 0,
    harness_version: "0.0.0-fixture",
  };
}

function runCase(budgets: string, current: Record<string, unknown>) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-platform-gate-"));
  const results = path.join(root, "results");
  fs.mkdirSync(results);
  fs.writeFileSync(path.join(root, "budgets.toml"), budgets);
  fs.writeFileSync(path.join(results, `${benchId}.json`), `${JSON.stringify(current)}\n`);
  const result = spawnSync(
    process.execPath,
    [
      comparePath,
      "--budgets",
      path.join(root, "budgets.toml"),
      "--baseline",
      path.join(root, "no-baseline"),
      "--results",
      results,
    ],
    { cwd: repoRoot, encoding: "utf8" },
  );
  fs.rmSync(root, { recursive: true, force: true });
  return result;
}

test("exact peak canaries reconcile on both registered platforms", () => {
  const budgets = budgetText("linux = 1254, macos = 1246");
  for (const [platform, peak] of [
    ["linux", 1254],
    ["macos", 1246],
  ] as const) {
    const result = runCase(budgets, report(platform, peak));
    assert.equal(result.stderr, "");
    assert.equal(result.status, 0, result.stdout);
    assert.match(result.stdout, new RegExp(`alloc_bytes_peak\\[${platform}\\] ${peak}B`));
  }
});

test("an unknown report platform fails closed", () => {
  const result = runCase(budgetText("linux = 1254, macos = 1246"), report("freebsd", 1254));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 1, result.stdout);
  assert.match(
    result.stdout,
    /FAIL ricalco_lower_vfor_three_aliases alloc_bytes_peak platform freebsd has no exact budget \(registered: linux, macos\)/u,
  );
});

test("a missing required platform budget is a configuration error", () => {
  const result = runCase(budgetText("linux = 1254"), report("linux", 1254));
  assert.equal(result.status, 2, result.stderr);
  assert.match(result.stderr, /must have exactly the platforms linux, macos \(found: linux\)/u);
});

test("a report without its platform is rejected before reconciliation", () => {
  const result = runCase(budgetText("linux = 1254, macos = 1246"), report(undefined, 1254));
  assert.equal(result.status, 2, result.stderr);
  assert.match(result.stderr, /has no valid platform/u);
});
