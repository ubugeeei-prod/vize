#!/usr/bin/env node
// Davinci bench budget gate (plan/phase-0.md P0-4).
//
// Compares the current davinci bench reports against the committed baseline
// reports under the thresholds registered in davinci-road/plan/budgets.toml
// `[bench.<bench_id>]`, and exits non-zero on any breach. One verdict row per
// bench on stdout, sorted by bench id, then a summary line.
//
// Inputs
//   --budgets  <path>  budgets file      (default davinci-road/plan/budgets.toml)
//   --baseline <dir>   baseline reports  (default bench/results/davinci/baseline)
//   --results  <dir>   current reports   (default bench/results/davinci)
//   Reports are the flat *.json files the harness exports (one per bench,
//   named <bench_id>.json, shaped by davinci-bench.schema.json); the baseline
//   subdirectory inside the default results dir is not itself a result.
//
// Gates, per registered bench
//   wall p50 — gated against the BASELINE report (never against the budget
//     number): current p50 may exceed baseline p50 by at most the entry's
//     wall_tolerance. Evaluated in exact integer arithmetic (BigInt, the
//     tolerance scaled to basis points).
//   allocs   — deterministic, so gated exactly: any change from the baseline
//     fails, including a change to or from null.
//   rss      — report-only for now (see the [bench] field docs in budgets.toml).
//
// Report-only states (never fail, always listed)
//   unbaselined — the bench has no committed baseline report yet.
//   unrecorded  — a baseline report exists but the budget entry still carries
//     the wall_p50_ns = 0 seed, i.e. the reference runner has not recorded
//     the blessed number; gating starts when the recorded budgets land.
//
// Registry drift is a failure in both directions — silent drift is the enemy:
//   a budgets entry with no current result       -> FAIL "bench disappeared"
//   a result (current or baseline) with no entry -> FAIL "unregistered bench"
//
// --update-baseline copies every current report over the baseline directory,
// and refuses (exit 2) unless DAVINCI_BASELINE_REFRESH=1 is set in the
// environment: the baseline is the reference every PR is gated against, so
// refreshing it must be deliberate. Even then the registry reconciliation
// above must pass first — a drifted bench set cannot be blessed.
//
// Exit codes: 0 = no breach, 1 = breach, 2 = usage/config error.

import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

import { parseTomlLite, TomlLiteError } from "./toml-lite.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

const BENCH_ID = /^[A-Za-z0-9._-]+$/;
const BUDGET_FIELDS = ["wall_p50_ns", "allocs", "rss_peak_bytes", "wall_tolerance"];

class ConfigError extends Error {}

function fail(message) {
  throw new ConfigError(message);
}

function parseArgs(argv) {
  const options = {
    budgets: path.join(repoRoot, "davinci-road", "plan", "budgets.toml"),
    baseline: path.join(repoRoot, "bench", "results", "davinci", "baseline"),
    results: path.join(repoRoot, "bench", "results", "davinci"),
    updateBaseline: false,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--update-baseline") {
      options.updateBaseline = true;
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
      `unknown argument ${JSON.stringify(arg)} (expected --budgets/--baseline/--results/--update-baseline)`,
    );
  }
  return options;
}

function loadBudgets(budgetsPath) {
  let text;
  try {
    text = fs.readFileSync(budgetsPath, "utf8");
  } catch {
    fail(`cannot read budgets file ${budgetsPath}`);
  }
  let root;
  try {
    root = parseTomlLite(text);
  } catch (error) {
    if (error instanceof TomlLiteError) fail(`${budgetsPath}: ${error.message}`);
    throw error;
  }
  const bench = root.bench;
  if (bench == null || typeof bench !== "object" || Array.isArray(bench)) {
    fail(`${budgetsPath}: missing [bench] section`);
  }
  const budgets = new Map();
  for (const [id, entry] of Object.entries(bench)) {
    if (!BENCH_ID.test(id)) fail(`${budgetsPath}: [bench.${id}] is not a valid bench id`);
    if (entry == null || typeof entry !== "object" || Array.isArray(entry)) {
      fail(`${budgetsPath}: [bench.${id}] is not a table`);
    }
    const keys = Object.keys(entry).sort();
    if (keys.join(",") !== [...BUDGET_FIELDS].sort().join(",")) {
      fail(
        `${budgetsPath}: [bench.${id}] must have exactly the fields ` +
          `${BUDGET_FIELDS.join(", ")} (found: ${keys.join(", ")})`,
      );
    }
    for (const field of ["wall_p50_ns", "allocs", "rss_peak_bytes"]) {
      const value = entry[field];
      if (!Number.isSafeInteger(value) || value < 0) {
        fail(`${budgetsPath}: [bench.${id}] ${field} must be a non-negative integer`);
      }
    }
    const tolerance = entry.wall_tolerance;
    if (typeof tolerance !== "number" || !(tolerance > 0) || !(tolerance < 1)) {
      fail(`${budgetsPath}: [bench.${id}] wall_tolerance must be a number in (0, 1)`);
    }
    const toleranceBp = Math.round(tolerance * 10000);
    if (Math.abs(tolerance * 10000 - toleranceBp) > 1e-6) {
      fail(`${budgetsPath}: [bench.${id}] wall_tolerance must be a whole number of basis points`);
    }
    budgets.set(id, { ...entry, toleranceBp });
  }
  return budgets;
}

function integerOrNull(value) {
  return value === null || (Number.isSafeInteger(value) && value >= 0);
}

function loadReports(dir, label) {
  const reports = new Map();
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    return reports; // a missing directory is an empty report set
  }
  for (const entry of entries) {
    if (!entry.isFile() || !entry.name.endsWith(".json")) continue;
    const file = path.join(dir, entry.name);
    let report;
    try {
      report = JSON.parse(fs.readFileSync(file, "utf8"));
    } catch {
      fail(`${label} report ${file} is not valid JSON`);
    }
    const stem = entry.name.slice(0, -".json".length);
    if (report == null || typeof report !== "object" || Array.isArray(report)) {
      fail(`${label} report ${file} is not an object`);
    }
    if (report.bench_id !== stem) {
      fail(
        `${label} report ${file} has bench_id ${JSON.stringify(report.bench_id)} (must match the file name)`,
      );
    }
    if (!BENCH_ID.test(stem)) fail(`${label} report ${file} has an invalid bench id`);
    const wall = report.wall_ns;
    if (
      wall == null ||
      typeof wall !== "object" ||
      !Number.isSafeInteger(wall.p50) ||
      wall.p50 < 0 ||
      !Number.isSafeInteger(wall.p95) ||
      wall.p95 < 0
    ) {
      fail(`${label} report ${file} has no integer wall_ns.p50/p95`);
    }
    if (!integerOrNull(report.allocs)) {
      fail(`${label} report ${file} has a non-integer, non-null allocs`);
    }
    if (!integerOrNull(report.rss_peak_bytes)) {
      fail(`${label} report ${file} has a non-integer, non-null rss_peak_bytes`);
    }
    reports.set(stem, { file, ...report });
  }
  return reports;
}

function byBenchId(a, b) {
  return a < b ? -1 : a > b ? 1 : 0;
}

function formatCount(value) {
  return value === null ? "n/a" : String(value);
}

function formatRss(value) {
  return value === null ? "n/a" : `${value}B`;
}

function wallLimit(baselineP50, toleranceBp) {
  return (BigInt(baselineP50) * (10000n + BigInt(toleranceBp))) / 10000n;
}

function compare(budgets, baselineReports, currentReports) {
  const rows = [];
  let breaches = 0;
  let gatedOk = 0;
  let reportOnly = 0;
  const ids = [
    ...new Set([...budgets.keys(), ...currentReports.keys(), ...baselineReports.keys()]),
  ].sort(byBenchId);
  for (const id of ids) {
    const budget = budgets.get(id);
    const current = currentReports.get(id);
    const baseline = baselineReports.get(id);
    if (budget == null) {
      const where = current != null ? "current" : "baseline";
      rows.push(
        `FAIL ${id} unregistered bench (${where} result has no budgets.toml [bench] entry)`,
      );
      breaches += 1;
      continue;
    }
    if (current == null) {
      rows.push(`FAIL ${id} bench disappeared (budgets.toml entry has no current result)`);
      breaches += 1;
      continue;
    }
    const stats =
      `wall_p50 ${current.wall_ns.p50}ns allocs ${formatCount(current.allocs)} ` +
      `rss ${formatRss(current.rss_peak_bytes)}`;
    if (baseline == null) {
      rows.push(`unbaselined ${id} ${stats} (report-only)`);
      reportOnly += 1;
      continue;
    }
    if (budget.wall_p50_ns === 0) {
      rows.push(`unrecorded ${id} ${stats} (report-only: budgets.toml baseline not yet recorded)`);
      reportOnly += 1;
      continue;
    }
    const limit = wallLimit(baseline.wall_ns.p50, budget.toleranceBp);
    const benchRows = [];
    if (BigInt(current.wall_ns.p50) > limit) {
      benchRows.push(
        `FAIL ${id} wall_p50 ${current.wall_ns.p50}ns > limit ${limit}ns ` +
          `(baseline ${baseline.wall_ns.p50}ns + ${budget.toleranceBp / 100}% tolerance)`,
      );
    }
    if (current.allocs !== baseline.allocs) {
      benchRows.push(
        `FAIL ${id} allocs ${formatCount(baseline.allocs)} -> ${formatCount(current.allocs)} ` +
          `(exact gate: allocs are deterministic)`,
      );
    }
    if (benchRows.length > 0) {
      rows.push(...benchRows);
      breaches += benchRows.length;
    } else {
      rows.push(
        `ok ${id} wall_p50 ${current.wall_ns.p50}ns ` +
          `(baseline ${baseline.wall_ns.p50}ns limit ${limit}ns) ` +
          `allocs ${formatCount(current.allocs)} rss ${formatRss(current.rss_peak_bytes)}`,
      );
      gatedOk += 1;
    }
  }
  return { rows, breaches, gatedOk, reportOnly };
}

function reconciliationOnly(budgets, currentReports) {
  const rows = [];
  const ids = [...new Set([...budgets.keys(), ...currentReports.keys()])].sort(byBenchId);
  for (const id of ids) {
    if (!budgets.has(id)) {
      rows.push(`FAIL ${id} unregistered bench (current result has no budgets.toml [bench] entry)`);
    } else if (!currentReports.has(id)) {
      rows.push(`FAIL ${id} bench disappeared (budgets.toml entry has no current result)`);
    }
  }
  return rows;
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

  const budgets = loadBudgets(options.budgets);
  const currentReports = loadReports(options.results, "current");
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

  const baselineReports = loadReports(options.baseline, "baseline");
  const { rows, breaches, gatedOk, reportOnly } = compare(budgets, baselineReports, currentReports);
  for (const row of rows) process.stdout.write(`${row}\n`);
  process.stdout.write(
    `bench-compare: breaches=${breaches} gated_ok=${gatedOk} ` +
      `report_only=${reportOnly} registered=${budgets.size}\n`,
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
