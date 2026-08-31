import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { laneEnvironment, makeTasks } from "../../tools/benchmarks/scripts/compare-pr-lanes.mjs";
import { createBenchmarkBudget, renderMarkdown } from "../../tools/benchmarks/scripts/compare-pr.mjs";
import {
  DEFAULT_SKIP_OVERRIDE_LABEL,
  enforceBenchmarkBudget,
} from "../../tools/benchmarks/scripts/enforce-pr-budget.mjs";
import { readRepoFile, workflowJobBody } from "./support/github-workflows.ts";

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
  assert.match(markdown, /Any threshold breach receives 5 fresh confirmation pairs/);
  assert.match(markdown, /\| Task \| Base median \| Head median \| Paired rate \| Result \|/);
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

// #3460: the published lint and fmt medians that regressed 20-27% were the
// `max` variants. Both PR-gate lanes ran pinned to one Rayon worker, where the
// same window measured lint at +2.6% and fmt at -35% — so the gate would have
// reported the window as neutral-to-better. Every parallel-path regression on
// these two surfaces was invisible to CI until these lanes existed.
test("benchmark tasks gate lint and fmt at max parallelism as well as single-threaded", () => {
  const tasks = makeTasks("/nonexistent-input-dir", "");

  const lintAndFmt = tasks.filter((task) => /^(lint|fmt)(-max)?$/.test(task.id));

  assert.deepEqual(
    lintAndFmt.map((task) => [task.id, task.label, task.args, task.threads ?? 1]),
    [
      ["lint", "Lint (1T)", ["lint", ".", "--quiet"], 1],
      ["lint-max", "Lint (max)", ["lint", ".", "--quiet"], "max"],
      ["fmt", "Format (1T)", ["fmt", "*.vue", "--check"], 1],
      ["fmt-max", "Format (max)", ["fmt", "*.vue", "--check"], "max"],
    ],
  );

  // The parallel lanes must stay non-destructive for the same reason the
  // single-threaded fmt lane does: base and head alternate over one corpus.
  // Asserting the whole argv is what pins that -- `fmt-max` reformats in
  // memory under `--check` and never gains a `--write`.
  assert.deepEqual(
    lintAndFmt
      .filter((task) => task.threads === "max")
      .map((task) => [task.id, task.args, task.allowNonZeroExit]),
    [
      ["lint-max", ["lint", ".", "--quiet"], true],
      ["fmt-max", ["fmt", "*.vue", "--check"], true],
    ],
  );

  assert.deepEqual(
    makeTasks("/nonexistent-input-dir", "lint-max,fmt-max").map((task) => task.id),
    ["lint-max", "fmt-max"],
  );
});

// #3586 pinned the weekly Benchmark run to a fixed historical base SHA, so the
// lanes `makeTasks` returns are also the lanes that gate long-term drift. That
// only reaches the parallel lint and fmt paths while the workflow keeps passing
// no `--tasks` filter; a filter added later would silently drop them again.
test("the benchmark workflow gates every lane on both the PR and the fixed-baseline run", () => {
  const workflow = readRepoFile(".github", "workflows", "benchmark.yml");
  const job = workflowJobBody(workflow, "pr-benchmark");
  const step = job.slice(job.indexOf("\n      - name: Compare base and head\n"));
  const compare = step.slice(0, step.indexOf("\n      - name: ", 1));

  assert.equal(
    /--tasks\b/.test(compare),
    false,
    "a --tasks filter would drop the parallel lint and fmt lanes from the gate",
  );

  // The budget gate reads the corpus and the JSON this step writes, so those
  // options are part of the contract even though the rest of the command line
  // is incidental.
  for (const option of ["--input", "--out", "--json"]) {
    assert.ok(compare.includes(option), `compare step must pass ${option}`);
  }

  // The weekly run reuses this job, so the fixed historical base is compared
  // over the same full lane set rather than a subset.
  assert.deepEqual(
    [
      /BENCHMARK_BASE_SHA:[^\n]*github\.event_name == 'schedule' && '[0-9a-f]{40}'/.test(workflow),
      /BENCHMARK_HEAD_SHA:[^\n]*github\.event_name == 'schedule' && github\.sha/.test(workflow),
    ],
    [true, true],
  );
});

test("lane environment pins Rayon per lane and unpins it for the max lanes", () => {
  const base = { PATH: "/usr/bin", RAYON_NUM_THREADS: "leaked-from-parent" };

  assert.deepEqual(laneEnvironment(1, base), {
    PATH: "/usr/bin",
    RAYON_NUM_THREADS: "1",
  });

  // `max` deletes the variable rather than writing a core count, so Rayon
  // sizes its own pool exactly as it does under `tools/benchmarks/scripts/compare-tools.mjs`.
  assert.deepEqual(laneEnvironment("max", base), { PATH: "/usr/bin" });

  // The caller's environment is never mutated in place.
  assert.deepEqual(base, { PATH: "/usr/bin", RAYON_NUM_THREADS: "leaked-from-parent" });
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
