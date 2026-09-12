// Davinci source-map budgets (plan/phase-3.md P3-9).
//
// P3-9 replaces string appends with structured, span-carrying emission across
// DOM, Vapor and SSR. This test pins the numeric coverage contract before the
// emitter work lands, and keeps the legacy SFC text-matching recovery visible
// as the deletion floor.

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { parseTomlLite } from "../../legacy-tools/davinci/toml-lite.mjs";

type SourceMapBudget = {
  backend: string;
  category: string;
  anchors_min: number;
  coverage_pct_min: number;
  span_accuracy_pct_min: number;
};

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const budgetsPath = path.join(repoRoot, "davinci-road", "plan", "budgets.toml");
const budgetsText = fs.readFileSync(budgetsPath, "utf8");
const budgets = parseTomlLite(budgetsText) as {
  sourcemap: Record<string, SourceMapBudget>;
};

const REQUIRED_BACKENDS = ["dom", "vapor", "ssr"] as const;
const REQUIRED_CATEGORIES = [
  "static-template",
  "rewritten-identifier",
  "section-boundary",
] as const;

function inlineTableLines(section: string): string[] {
  const lines = budgetsText.split("\n");
  const start = lines.indexOf(`[${section}]`);
  assert.ok(start >= 0, `budgets.toml must declare a [${section}] section`);
  const rest = lines.slice(start + 1);
  const end = rest.findIndex((line) => line.startsWith("["));
  const body = end === -1 ? rest : rest.slice(0, end);
  return body.filter((line) => /^[A-Za-z0-9._-]+ = \{/.test(line));
}

function assertPercent(value: unknown, field: string, id: string): asserts value is number {
  assert.ok(
    Number.isSafeInteger(value) && value >= 0 && value <= 100,
    `[sourcemap.${id}] ${field} must be a 0..100 integer, got ${String(value)}`,
  );
}

test("source-map budgets cover every P3-9 backend/category cell", () => {
  const entries = Object.entries(budgets.sourcemap);
  assert.ok(entries.length > 0, "budgets.toml [sourcemap] must not be empty for P3-9");

  const requiredCells = new Set(
    REQUIRED_BACKENDS.flatMap((backend) =>
      REQUIRED_CATEGORIES.map((category) => `${backend}/${category}`),
    ),
  );
  const seenCells = new Set<string>();
  const legacyRows: string[] = [];

  for (const [id, entry] of entries) {
    assert.deepEqual(
      Object.keys(entry).sort(),
      ["anchors_min", "backend", "category", "coverage_pct_min", "span_accuracy_pct_min"],
      `[sourcemap.${id}] must keep the documented field set`,
    );
    assert.ok(Number.isSafeInteger(entry.anchors_min) && entry.anchors_min > 0);
    assertPercent(entry.coverage_pct_min, "coverage_pct_min", id);
    assertPercent(entry.span_accuracy_pct_min, "span_accuracy_pct_min", id);

    if (entry.backend === "legacy-sfc") {
      legacyRows.push(`${id}/${entry.category}`);
      continue;
    }

    assert.ok(
      (REQUIRED_BACKENDS as readonly string[]).includes(entry.backend),
      `[sourcemap.${id}] backend ${entry.backend} is not a P3-9 backend`,
    );
    assert.ok(
      (REQUIRED_CATEGORIES as readonly string[]).includes(entry.category),
      `[sourcemap.${id}] category ${entry.category} is not a required P3-9 category`,
    );
    const cell = `${entry.backend}/${entry.category}`;
    assert.ok(!seenCells.has(cell), `duplicate source-map budget cell ${cell}`);
    seenCells.add(cell);
  }

  const missingCells = [...requiredCells].filter((cell) => !seenCells.has(cell)).sort();
  assert.deepEqual(missingCells, [], "P3-9 must pin every backend/category cell");
  assert.deepEqual(legacyRows, ["legacy_sfc_text_matching_recovery/text-matching-recovery"]);
});

test("source-map budget entries are one-line inline tables with real thresholds", () => {
  const entryLines = inlineTableLines("sourcemap");
  assert.equal(entryLines.length, Object.keys(budgets.sourcemap).length);
  for (const line of entryLines) {
    assert.match(
      line,
      /^[A-Za-z0-9._-]+ = \{ backend = "[a-z-]+", category = "[a-z-]+", anchors_min = [1-9]\d*, coverage_pct_min = \d+, span_accuracy_pct_min = \d+ \}$/,
      `sourcemap entries must keep the canonical field order and spacing:\n${line}`,
    );
  }
});

test("rewritten identifiers and legacy recovery keep exact span accuracy floors", () => {
  for (const [id, entry] of Object.entries(budgets.sourcemap)) {
    if (entry.category === "rewritten-identifier" || entry.backend === "legacy-sfc") {
      assert.equal(
        entry.coverage_pct_min,
        100,
        `[sourcemap.${id}] must keep full coverage until P3-9 records a reviewed floor`,
      );
      assert.equal(entry.span_accuracy_pct_min, 100);
    }
  }
});
