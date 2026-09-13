// Davinci optimization budgets (plan/phase-3.md P3-10).
//
// P3-10's try-measure-commit pass is allowed to choose an extraction only
// after the metric rule is pinned. This test keeps the `-O` tiers, candidate
// budgets, epsilons and tie policy reviewable before the pass implementation
// lands.

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { parseTomlLite } from "../../legacy-tools/davinci/toml-lite.mjs";

type OptimizationBudget = {
  tier: string;
  candidate_budget: number;
  emitted_size_epsilon_pct: number;
  reactive_edge_epsilon_pct: number;
  update_path_epsilon_pct: number;
  required_improvements_min: number;
  tie_policy: string;
};

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const budgetsPath = path.join(repoRoot, "davinci-road", "plan", "budgets.toml");
const budgetsText = fs.readFileSync(budgetsPath, "utf8");
const budgets = parseTomlLite(budgetsText) as {
  optimization: Record<string, OptimizationBudget>;
  target: Record<string, Record<string, unknown>>;
};

const REQUIRED_TIERS = [
  ["o0", "O0", 0, 0],
  ["o1", "O1", 8, 1],
  ["o2", "O2", 32, 1],
  ["o3", "O3", 128, 1],
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
    `[optimization.${id}] ${field} must be a 0..100 integer, got ${String(value)}`,
  );
}

test("optimization budgets pin every P3-10 -O tier", () => {
  assert.deepEqual(
    Object.keys(budgets.optimization).sort(),
    REQUIRED_TIERS.map(([id]) => id),
  );
  for (const [id, tier, candidateBudget, requiredImprovements] of REQUIRED_TIERS) {
    const entry = budgets.optimization[id];
    assert.deepEqual(
      Object.keys(entry).sort(),
      [
        "candidate_budget",
        "emitted_size_epsilon_pct",
        "reactive_edge_epsilon_pct",
        "required_improvements_min",
        "tie_policy",
        "tier",
        "update_path_epsilon_pct",
      ],
      `[optimization.${id}] must keep the documented field set`,
    );
    assert.equal(entry.tier, tier);
    assert.equal(entry.candidate_budget, candidateBudget);
    assert.equal(entry.required_improvements_min, requiredImprovements);
    assert.equal(entry.tie_policy, "reject");
    assertPercent(entry.emitted_size_epsilon_pct, "emitted_size_epsilon_pct", id);
    assertPercent(entry.reactive_edge_epsilon_pct, "reactive_edge_epsilon_pct", id);
    assertPercent(entry.update_path_epsilon_pct, "update_path_epsilon_pct", id);
  }
});

test("optimization budgets are monotone and reject ties above O0", () => {
  let previousBudget = -1;
  for (const [id] of REQUIRED_TIERS) {
    const entry = budgets.optimization[id];
    assert.ok(
      entry.candidate_budget > previousBudget,
      `[optimization.${id}] candidate_budget must increase with optimization level`,
    );
    previousBudget = entry.candidate_budget;
    assert.equal(
      entry.reactive_edge_epsilon_pct,
      0,
      `[optimization.${id}] reactive-edge regressions are not allowed at task start`,
    );
    assert.equal(
      entry.update_path_epsilon_pct,
      0,
      `[optimization.${id}] update-path regressions are not allowed at task start`,
    );
    assert.equal(entry.emitted_size_epsilon_pct, 0);
    assert.equal(entry.required_improvements_min, id === "o0" ? 0 : 1);
  }
});

test("optimization entries are one-line inline tables", () => {
  const entryLines = inlineTableLines("optimization");
  assert.equal(entryLines.length, REQUIRED_TIERS.length);
  for (const line of entryLines) {
    assert.match(
      line,
      /^[A-Za-z0-9._-]+ = \{ tier = "O[0-3]", candidate_budget = \d+, emitted_size_epsilon_pct = \d+, reactive_edge_epsilon_pct = \d+, update_path_epsilon_pct = \d+, required_improvements_min = \d+, tie_policy = "reject" \}$/,
      `optimization entries must keep the canonical field order and spacing:\n${line}`,
    );
  }
});

test("P3-10 target records the metric roles and task-start revision", () => {
  const target = budgets.target?.["p3-10"];
  assert.ok(target, "budgets.toml must carry a [target.p3-10] table");
  assert.ok(
    typeof target.task_start_rev === "string" && /^[0-9a-f]{40}$/.test(target.task_start_rev),
    `[target.p3-10] task_start_rev must be a full 40-hex commit sha`,
  );
  assert.match(
    String(target.task_start_date),
    /^\d{4}-\d{2}-\d{2}$/,
    "[target.p3-10] task_start_date must be an ISO date string",
  );
  assert.equal(target.objective_metric, "emitted-size");
  assert.deepEqual(target.constraint_metrics, ["reactive-edge", "update-path"]);
  assert.equal(target.tie_policy, "reject");
});
