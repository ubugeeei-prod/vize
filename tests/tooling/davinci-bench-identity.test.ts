// Davinci bench artifacts are only comparable when they describe the same
// fixture, platform, and harness version.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const comparePath = path.join(repoRoot, "tools", "davinci", "bench-compare.mjs");
const fixtureDir = path.join(repoRoot, "tests", "_fixtures", "davinci-bench-compare");

function runCompare(root: string, budgets: string, baseline: string, current: string) {
  const env = { ...process.env };
  delete env.DAVINCI_BASELINE_REFRESH;
  return spawnSync(
    process.execPath,
    [comparePath, "--budgets", budgets, "--baseline", baseline, "--results", current],
    { cwd: root, encoding: "utf8", env },
  );
}

function writeBudget(pathname: string) {
  fs.writeFileSync(
    pathname,
    "[bench]\nfixture_parse_small = { wall_p50_ns = 100000, allocs = 42, " +
      "rss_peak_bytes = 8192, wall_tolerance = 0.05 }\n",
  );
}

function runIdentityMismatch(field: string, value: string) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-bench-identity-"));
  const baseline = path.join(root, "baseline");
  const current = path.join(root, "current");
  const budgets = path.join(root, "budgets.toml");
  fs.mkdirSync(baseline);
  fs.mkdirSync(current);
  writeBudget(budgets);
  const report = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, "baseline", "fixture_parse_small.json"), "utf8"),
  );
  fs.writeFileSync(path.join(baseline, "fixture_parse_small.json"), `${JSON.stringify(report)}\n`);
  const changed = structuredClone(report);
  changed[field] = value;
  fs.writeFileSync(path.join(current, "fixture_parse_small.json"), `${JSON.stringify(changed)}\n`);
  const result = runCompare(root, budgets, baseline, current);
  fs.rmSync(root, { recursive: true, force: true });
  return { budgets, baseline, current, result };
}

test("bench-compare rejects mismatched baseline/current artifact identity", () => {
  for (const { field, value, original } of [
    {
      field: "fixture",
      original: "synthetic:bench-compare-fixture",
      value: "synthetic:other-fixture",
    },
    { field: "platform", original: "linux", value: "macos" },
    { field: "harness_version", original: "0.0.0-fixture", value: "0.0.1-fixture" },
  ]) {
    const { budgets, baseline, current, result } = runIdentityMismatch(field, value);
    assert.equal(result.status, 2, result.stderr);
    assert.equal(
      result.stdout,
      `bench-compare: budgets=${budgets} baseline=${baseline} results=${current}\n`,
    );
    assert.equal(
      result.stderr,
      `bench-compare: fixture_parse_small baseline/current ${field} mismatch: ${original} vs ${value}\n`,
    );
  }
});
