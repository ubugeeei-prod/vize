#!/usr/bin/env node
// Davinci bench budget gate (plan/phase-0.md P0-4).
//
// Compares the current davinci bench reports against the committed baseline
// reports under the thresholds registered in davinci-road/plan/budgets.toml
// `[bench]`, and exits non-zero on any breach. One verdict row per bench on
// stdout, sorted by bench id, then a summary line.
//
// Inputs
//   --budgets  <path>  budgets file      (default davinci-road/plan/budgets.toml)
//   --baseline <dir>   baseline reports  (default bench/results/davinci/baseline)
//   --results  <dir>   current reports   (default bench/results/davinci)
//   --bench    <id>    select one bench  (repeatable; default is the full registry)
//   Reports are the flat *.json files the harness exports (one per bench,
//   named <bench_id>.json, shaped by davinci-bench.schema.json); the baseline
//   subdirectory inside the default results dir is not itself a result.
//
// The registry loader lives in lib/bench-budgets.mjs, the report loader in
// lib/bench-reports.mjs, and the per-bench gates (wall p50 vs the baseline
// report, exact alloc counts, report-only RSS, registry drift in both
// directions) in lib/bench-verdicts.mjs.
//
// Report-only states (never fail, always listed)
//   unbaselined — the bench has no committed baseline report yet.
//   unrecorded  — a baseline report exists but the budget entry still carries
//     the wall_p50_ns = 0 seed, i.e. the reference runner has not recorded
//     the blessed number; gating starts when the recorded budgets land.
//
// --update-baseline copies every current report over the baseline directory,
// and refuses (exit 2) unless DAVINCI_BASELINE_REFRESH=1 is set in the
// environment: the baseline is the reference every PR is gated against, so
// refreshing it must be deliberate. Even then the registry reconciliation
// must pass first — a drifted bench set cannot be blessed.
//
// Exit codes: 0 = no breach, 1 = breach, 2 = usage/config error.

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { loadBudgets } from "./lib/bench-budgets.mjs";
import { ConfigError, fail } from "./lib/bench-config.mjs";
import { loadReports } from "./lib/bench-reports.mjs";
import { byBenchId, compare, reconciliationOnly } from "./lib/bench-verdicts.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

function parseArgs(argv) {
  const options = {
    budgets: path.join(repoRoot, "davinci-road", "plan", "budgets.toml"),
    baseline: path.join(repoRoot, "bench", "results", "davinci", "baseline"),
    results: path.join(repoRoot, "bench", "results", "davinci"),
    benches: [],
    updateBaseline: false,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--update-baseline") {
      options.updateBaseline = true;
      continue;
    }
    if (arg === "--bench") {
      const value = argv[i + 1];
      if (value == null) fail("--bench requires a bench id");
      options.benches.push(value);
      i += 1;
      continue;
    }
    if (arg === "--budgets" || arg === "--baseline" || arg === "--results") {
      const value = argv[i + 1];
      if (value == null) fail(`${arg} requires a path argument`);
      options[arg.slice(2)] = value;
      i += 1;
      continue;
    }
    fail(
      `unknown argument ${JSON.stringify(arg)} ` +
        "(expected --budgets/--baseline/--results/--bench/--update-baseline)",
    );
  }
  return options;
}

function selectEntries(entries, ids) {
  if (ids.length === 0) return entries;
  return new Map(ids.filter((id) => entries.has(id)).map((id) => [id, entries.get(id)]));
}

function updateBaseline(options, budgets, currentReports) {
  const driftRows = reconciliationOnly(budgets, currentReports);
  if (driftRows.length > 0) {
    for (const row of driftRows) process.stdout.write(`${row}\n`);
    process.stdout.write(
      `bench-compare: refusing baseline update: reconciliation failed (breaches=${driftRows.length})\n`,
    );
    return 1;
  }
  fs.mkdirSync(options.baseline, { recursive: true });
  for (const id of [...currentReports.keys()].sort(byBenchId)) {
    fs.copyFileSync(currentReports.get(id).file, path.join(options.baseline, `${id}.json`));
    process.stdout.write(`updated baseline ${id}\n`);
  }
  process.stdout.write(
    `bench-compare: baseline updated (${currentReports.size} benches) under ${options.baseline}\n`,
  );
  return 0;
}

function main() {
  const options = parseArgs(process.argv.slice(2));

  if (options.updateBaseline && process.env.DAVINCI_BASELINE_REFRESH !== "1") {
    process.stderr.write(
      "bench-compare: refusing --update-baseline without DAVINCI_BASELINE_REFRESH=1.\n" +
        "The committed baseline is the reference every PR is gated against; refreshing it\n" +
        "must be a deliberate act on the reference runner, not a side effect. Re-run with\n" +
        "DAVINCI_BASELINE_REFRESH=1 in the environment to proceed.\n",
    );
    return 2;
  }

  const allBudgets = loadBudgets(options.budgets);
  const selected = [...new Set(options.benches)];
  for (const id of selected) {
    if (!allBudgets.has(id)) fail(`--bench ${id} has no budgets.toml entry`);
  }
  const budgets = selectEntries(allBudgets, selected);
  const currentReports = selectEntries(loadReports(options.results, "current"), selected);
  if (currentReports.size === 0) {
    fail(
      `no current bench reports (*.json) under ${options.results} — ` +
        `run the davinci benches first (vp bench:davinci)`,
    );
  }

  process.stdout.write(
    `bench-compare: budgets=${options.budgets} baseline=${options.baseline} results=${options.results}\n`,
  );

  if (options.updateBaseline) {
    return updateBaseline(options, budgets, currentReports);
  }

  const baselineReports = selectEntries(loadReports(options.baseline, "baseline"), selected);
  const { rows, breaches, gatedOk, allocGated } = compare(budgets, baselineReports, currentReports);
  for (const row of rows) process.stdout.write(`${row}\n`);
  process.stdout.write(
    `bench-compare: breaches=${breaches} gated_ok=${gatedOk} ` +
      `alloc_gated=${allocGated} registered=${budgets.size}\n`,
  );
  return breaches > 0 ? 1 : 0;
}

try {
  process.exitCode = main();
} catch (error) {
  if (error instanceof ConfigError) {
    process.stderr.write(`bench-compare: ${error.message}\n`);
    process.exitCode = 2;
  } else {
    throw error;
  }
}
