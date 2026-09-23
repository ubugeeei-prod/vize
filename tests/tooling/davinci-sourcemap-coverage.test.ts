// Davinci TS-31 source-map coverage gate (plan/phase-3.md P3-9).
//
// `cargo test -p vize_atelier_sfc --test davinci_ts31_sourcemap` measures the
// TS-31 fixture battery and keeps `docs/davinci/plan/ts31-sourcemap-coverage.json`
// equal to the measurement. This gate compares that report with the pinned
// `[sourcemap]` budgets: every row either meets its budget or is listed, with
// its exact measured counts, in the P3-9 record's tracked-shortfall ledger.
// A row that starts passing must leave the ledger; a new shortfall must enter
// it. Budgets are never adjusted here.

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { parseTomlLite } from "../../tools/support/compat/davinci/toml-lite.mjs";

type Budget = {
  backend: string;
  category: string;
  anchors_min: number;
  coverage_pct_min: number;
  span_accuracy_pct_min: number;
};

type ReportRow = {
  authored: number;
  covered: number;
  exact: number;
  anchors: Record<string, string>;
};

type Report = {
  suite: string;
  command: string;
  battery: { templates: number; sfcs: number };
  rows: Record<string, ReportRow>;
};

type Shortfall = { authored: number; covered: number; exact: number };

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const plan = path.join(repoRoot, "docs/davinci", "plan");
const budgets = (
  parseTomlLite(fs.readFileSync(path.join(plan, "budgets.toml"), "utf8")) as {
    sourcemap: Record<string, Budget>;
  }
).sourcemap;
const report = JSON.parse(
  fs.readFileSync(path.join(plan, "ts31-sourcemap-coverage.json"), "utf8"),
) as Report;
const record = fs.readFileSync(path.join(plan, "phase-3-records", "p3-9.md"), "utf8");

const STATUSES = new Set(["exact", "covered", "unmapped", "absent"]);
const SHORTFALL_HEADING = "### TS-31 tracked shortfalls";

function rowKey(budget: Budget): string {
  return `${budget.backend}/${budget.category}`;
}

function meetsBudget(row: ReportRow, budget: Budget): boolean {
  return (
    row.exact >= budget.anchors_min &&
    row.covered > 0 &&
    row.covered * 100 >= budget.coverage_pct_min * row.authored &&
    row.exact * 100 >= budget.span_accuracy_pct_min * row.covered
  );
}

function shortfallLedger(): Map<string, Shortfall> {
  const start = record.indexOf(SHORTFALL_HEADING);
  assert.ok(start >= 0, `P3-9 record must keep a "${SHORTFALL_HEADING}" ledger`);
  const lines = record.slice(start + SHORTFALL_HEADING.length).split("\n");
  const firstRow = lines.findIndex((line) => line.startsWith("|"));
  assert.ok(firstRow >= 0, "shortfall ledger must be a table");
  const tableEnd = lines.slice(firstRow).findIndex((line) => !line.startsWith("|"));
  const table = lines.slice(firstRow, tableEnd === -1 ? undefined : firstRow + tableEnd);
  const cells = (line: string) =>
    line
      .slice(1, -1)
      .split("|")
      .map((cell) => cell.trim());
  assert.deepEqual(cells(table[0]), ["Row", "Authored", "Covered", "Exact", "Closing slice"]);
  const ledger = new Map<string, Shortfall>();
  for (const line of table.slice(2)) {
    const [row, authored, covered, exact, closing] = cells(line);
    const key = /^`(?<key>[a-z-]+\/[a-z-]+)`$/u.exec(row)?.groups?.key;
    assert.ok(key, `ledger row must name a backticked report row: ${line}`);
    assert.ok(closing.length > 0, `${key} must name the slice that closes it`);
    assert.ok(!ledger.has(key), `duplicate ledger row ${key}`);
    ledger.set(key, {
      authored: Number(authored),
      covered: Number(covered),
      exact: Number(exact),
    });
  }
  return ledger;
}

test("TS-31 report carries exactly one row per pinned budget", () => {
  assert.equal(report.suite, "TS-31");
  assert.equal(report.command, "cargo test -p vize_atelier_sfc --test davinci_ts31_sourcemap");
  const budgetKeys = Object.values(budgets).map(rowKey).sort();
  assert.deepEqual(Object.keys(report.rows).sort(), budgetKeys);
});

test("TS-31 report counts agree with the per-anchor statuses", () => {
  for (const [key, row] of Object.entries(report.rows)) {
    const statuses = Object.values(row.anchors);
    for (const status of statuses) {
      assert.ok(STATUSES.has(status), `${key}: unknown anchor status ${status}`);
    }
    const exact = statuses.filter((status) => status === "exact").length;
    const covered = exact + statuses.filter((status) => status === "covered").length;
    const authored = statuses.filter((status) => status !== "absent").length;
    assert.deepEqual(
      { authored: row.authored, covered: row.covered, exact: row.exact },
      { authored, covered, exact },
      `${key}: summary counts must be derived from its anchors`,
    );
  }
});

test("every TS-31 row meets its budget or is a tracked shortfall", () => {
  const failing = new Map<string, Shortfall>();
  for (const budget of Object.values(budgets)) {
    const key = rowKey(budget);
    const row = report.rows[key];
    if (!meetsBudget(row, budget)) {
      failing.set(key, { authored: row.authored, covered: row.covered, exact: row.exact });
    }
  }
  const ledger = shortfallLedger();
  assert.deepEqual(
    [...ledger.entries()].sort(([a], [b]) => a.localeCompare(b)),
    [...failing.entries()].sort(([a], [b]) => a.localeCompare(b)),
    "the P3-9 shortfall ledger must list exactly the rows below budget, with measured counts",
  );
});

test("the legacy SFC recovery is deleted and its frozen row stays the floor", () => {
  const legacy = Object.values(budgets).filter((budget) => budget.backend === "legacy-sfc");
  assert.deepEqual(legacy.map(rowKey), ["legacy-sfc/text-matching-recovery"]);
  const legacyRecovery = path.join(repoRoot, "crates/vize_atelier_sfc/src/source_map.rs");
  assert.equal(
    fs.existsSync(legacyRecovery),
    false,
    "the text-matching recovery is deleted; the structured module map is the SFC source map",
  );
  assert.equal(meetsBudget(report.rows[rowKey(legacy[0])], legacy[0]), true);
});

// The legacy row is frozen: the recovery is deleted, so it is not remeasured.
// Anchor by anchor, the live structured status is never worse than that
// frozen row, and its budget is never looser.
test("the structured SFC map covers every anchor the legacy recovery does", () => {
  const rank = (status: string) => ["unmapped", "covered", "exact"].indexOf(status);
  const legacyBudget = budgets.legacy_sfc_text_matching_recovery;
  const structuredBudget = budgets.sfc_text_matching_recovery;
  assert.equal(rowKey(structuredBudget), "sfc/text-matching-recovery");
  const legacyRow = report.rows[rowKey(legacyBudget)];
  const structuredRow = report.rows[rowKey(structuredBudget)];
  assert.deepEqual(
    Object.keys(structuredRow.anchors).sort(),
    Object.keys(legacyRow.anchors).sort(),
  );
  for (const [anchor, status] of Object.entries(legacyRow.anchors)) {
    assert.ok(
      rank(structuredRow.anchors[anchor]) >= rank(status),
      `${anchor}: structured ${structuredRow.anchors[anchor]} is worse than legacy ${status}`,
    );
  }
  assert.ok(structuredBudget.anchors_min >= legacyRow.exact);
  assert.ok(structuredBudget.coverage_pct_min >= legacyBudget.coverage_pct_min);
  assert.ok(structuredBudget.span_accuracy_pct_min >= legacyBudget.span_accuracy_pct_min);
  assert.equal(meetsBudget(structuredRow, structuredBudget), true);
});
