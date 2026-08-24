// The davinci bench budget registry loader (davinci-road/plan/budgets.toml
// `[bench]`), used by tools/davinci/bench-compare.mjs.
//
// Every entry must carry exactly the documented field set, so a typo or a
// half-added bench fails the gate instead of silently going ungated. The
// wall tolerance is additionally converted to whole basis points so the wall
// comparison can run in exact integer arithmetic.

import fs from "node:fs";

import { parseTomlLite, TomlLiteError } from "../toml-lite.mjs";
import { BENCH_ID, fail } from "./bench-config.mjs";

export const BUDGET_FIELDS = ["wall_p50_ns", "allocs", "rss_peak_bytes", "wall_tolerance"];

export function loadBudgets(budgetsPath) {
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
  const allocationPeak = root.allocation_peak ?? {};
  if (typeof allocationPeak !== "object" || Array.isArray(allocationPeak)) {
    fail(`${budgetsPath}: [allocation_peak] must be a table`);
  }
  for (const [id, value] of Object.entries(allocationPeak)) {
    const budget = budgets.get(id);
    if (budget == null) fail(`${budgetsPath}: [allocation_peak] has unknown bench ${id}`);
    if (!Number.isSafeInteger(value) || value < 0) {
      fail(`${budgetsPath}: [allocation_peak.${id}] must be a non-negative integer`);
    }
    budget.allocBytesPeak = value;
  }
  return budgets;
}
