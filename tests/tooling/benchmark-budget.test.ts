import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { createBenchmarkBudget, makeTasks, renderMarkdown } from "../../bench/compare-pr.mjs";
import {
  DEFAULT_SKIP_OVERRIDE_LABEL,
  enforceBenchmarkBudget,
} from "../../bench/enforce-pr-budget.mjs";

const stableResult = {
  id: "compile",
  label: "Compile SFC",
  baseMs: 100,
  headMs: 102,
  rate: 1.02,
  changePercent: 2,
  status: "stable",
  baseRuns: [100, 101, 99],
  headRuns: [102, 101, 103],
};

const regressionResult = {
  id: "check",
  label: "Type check",
  baseMs: 100,
  headMs: 115,
  rate: 1.15,
  changePercent: 15,
  status: "regression",
  baseRuns: [99, 100, 101],
  headRuns: [114, 115, 116],
};

test("benchmark budget marks regression tasks as a failed gate", () => {
  const budget = createBenchmarkBudget([stableResult, regressionResult]);

  assert.equal(budget.status, "failed");
  assert.equal(budget.regressionCount, 1);
  assert.deepEqual(
    budget.regressions.map((regression) => regression.id),
    ["check"],
  );

  const enforcement = enforceBenchmarkBudget({
    results: [stableResult, regressionResult],
    budget,
  });

  assert.equal(enforcement.ok, false);
  assert.match(enforcement.message, /Type check/);
  assert.match(enforcement.message, /1\.150x/);
  assert.match(enforcement.message, /\+15\.00%/);
});

test("benchmark markdown reports the active regression budget", () => {
  const markdown = renderMarkdown({
    baseLabel: "base",
    headLabel: "head",
    fileCount: 300,
    runs: 5,
    warmups: 1,
    thresholdPercent: 5,
    results: [stableResult, regressionResult],
  });

  assert.match(markdown, /Budget: failed \(1 regression\)\./);
  assert.match(markdown, /Regression budget failures:/);
  assert.match(markdown, /Type check: 1\.150x \(\+15\.00%\)/);
});

test("benchmark tasks gate the formatter without mutating the corpus", () => {
  const tasks = makeTasks("/nonexistent-input-dir", "");
  const ids = tasks.map((task) => task.id);

  assert.ok(ids.includes("fmt"), "default task set must include the fmt lane");

  const fmt = tasks.find((task) => task.id === "fmt");
  assert.ok(fmt);
  assert.ok(fmt.args.includes("--check"), "fmt lane must run in non-destructive check mode");
  assert.ok(!fmt.args.includes("--write"), "fmt lane must never rewrite the shared corpus");
  assert.equal(
    fmt.allowNonZeroExit,
    true,
    "unformatted corpus exits non-zero in check mode; the lane only measures time",
  );

  // Filtering still works for the new lane.
  const onlyFmt = makeTasks("/nonexistent-input-dir", "fmt");
  assert.deepEqual(
    onlyFmt.map((task) => task.id),
    ["fmt"],
  );
});

test("benchmark tasks gate both single-server and sharded typecheck paths", () => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-tasks-"));
  try {
    const tsconfig = path.join(tempDir, "tsconfig.json");
    fs.writeFileSync(tsconfig, "{}\n");

    const tasks = makeTasks(tempDir, "");
    const checkTasks = tasks.filter((task) => task.id.startsWith("check"));

    assert.deepEqual(
      checkTasks.map((task) => task.id),
      ["check", "check-max"],
    );

    const singleServer = checkTasks.find((task) => task.id === "check");
    assert.ok(singleServer);
    assert.deepEqual(singleServer.args, [
      "check",
      ".",
      "--quiet",
      "--servers",
      "1",
      "--tsconfig",
      tsconfig,
    ]);

    const sharded = checkTasks.find((task) => task.id === "check-max");
    assert.ok(sharded);
    assert.deepEqual(sharded.args, ["check", ".", "--quiet", "--tsconfig", tsconfig]);
    assert.equal(
      sharded.allowNonZeroExit,
      true,
      "type-check diagnostics are benchmarked even when the corpus reports errors",
    );

    const onlySharded = makeTasks(tempDir, "check-max");
    assert.deepEqual(
      onlySharded.map((task) => task.id),
      ["check-max"],
    );
  } finally {
    fs.rmSync(tempDir, { force: true, recursive: true });
  }
});

test("benchmark budget blocks skipped benchmark runs without an override label", () => {
  const enforcement = enforceBenchmarkBudget({
    skipped: true,
    reason: "base_metadata_invalid",
  });

  assert.equal(enforcement.ok, false);
  assert.match(enforcement.message, /base_metadata_invalid/);
  assert.match(enforcement.message, new RegExp(DEFAULT_SKIP_OVERRIDE_LABEL));
});

test("benchmark budget allows skipped benchmark runs with an override label", () => {
  const enforcement = enforceBenchmarkBudget(
    {
      skipped: true,
      reason: "base_metadata_invalid",
    },
    {
      labels: [DEFAULT_SKIP_OVERRIDE_LABEL],
    },
  );

  assert.equal(enforcement.ok, true);
  assert.match(enforcement.message, /base_metadata_invalid/);
  assert.match(enforcement.message, new RegExp(DEFAULT_SKIP_OVERRIDE_LABEL));
});
