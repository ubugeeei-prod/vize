import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

import type { AppConfig } from "../../_helpers/apps.ts";
import { repoRoot } from "../../_helpers/realworld-patch.ts";
import { runCrashFreeVizeCheck, type VizeCheckSummary } from "./realworld.ts";

export interface BatchCheckBudget {
  coldMs: number;
  warmMs: number;
}

const registryPath = "tests/_fixtures/vue-ecosystem-fixtures.json";

export function loadBatchCheckBudget(projectId: string): BatchCheckBudget {
  const registry = JSON.parse(fs.readFileSync(path.join(repoRoot, registryPath), "utf8")) as {
    projects: Array<{ id: string; batchCheckBudget?: BatchCheckBudget }>;
  };
  const owners = registry.projects.filter(
    (project) => project.id === projectId && project.batchCheckBudget != null,
  );
  assert.equal(
    owners.length,
    1,
    `${projectId} must have exactly one batchCheckBudget block in ${registryPath}`,
  );
  const budget = owners[0].batchCheckBudget!;
  for (const [lane, value] of Object.entries(budget)) {
    assert.ok(
      Number.isSafeInteger(value) && value > 0 && value <= 600_000,
      `${projectId} batchCheckBudget.${lane} must be a positive integer at most 600000`,
    );
  }
  assert.ok(budget.warmMs <= budget.coldMs, `${projectId} warm budget must not exceed cold`);
  return budget;
}

export function runBudgetedBatchVizeCheck(app: AppConfig): {
  cold: VizeCheckSummary;
  warm: VizeCheckSummary;
} {
  const budget = loadBatchCheckBudget(app.name);
  const cold = runCrashFreeVizeCheck(app, { timeoutMs: budget.coldMs });
  const warm = runCrashFreeVizeCheck(app, { timeoutMs: budget.warmMs });

  assert.deepEqual(
    warm.result,
    cold.result,
    `${app.name} warm batch check must reproduce the complete cold result`,
  );
  return { cold, warm };
}
