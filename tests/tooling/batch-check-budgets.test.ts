import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { loadBatchCheckBudget } from "../snapshots/_helpers/batch-check-performance.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registry = JSON.parse(
  fs.readFileSync(path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
) as { projects: Array<{ id: string; batchCheckBudget?: { coldMs: number; warmMs: number } }> };

test("Vben and Misskey own complete batch cold and warm budgets", () => {
  const owners = registry.projects.filter((project) => project.batchCheckBudget != null);
  assert.deepEqual(owners.map((project) => project.id).sort(), ["misskey", "vue-vben-admin"]);
  assert.deepEqual(loadBatchCheckBudget("vue-vben-admin"), {
    coldMs: 300_000,
    warmMs: 300_000,
  });
  // Raised with the deterministic single-checker pin (#3905): the one-program
  // misskey check measured 61s cold on an M-series darwin, and CI runners
  // need headroom above that.
  assert.deepEqual(loadBatchCheckBudget("misskey"), {
    coldMs: 180_000,
    warmMs: 150_000,
  });
});

test("batch budget consumers run both projects through the fail-closed helper", () => {
  for (const fixture of ["vue-vben-admin.ts", "misskey.ts"]) {
    const source = fs.readFileSync(path.join(root, "tests/snapshots/check", fixture), "utf8");
    assert.match(source, /runBudgetedBatchVizeCheck\(app\)/);
  }
  const helper = fs.readFileSync(
    path.join(root, "tests/snapshots/_helpers/batch-check-performance.ts"),
    "utf8",
  );
  assert.match(helper, /const cold = runCrashFreeVizeCheck/);
  assert.match(helper, /const warm = runCrashFreeVizeCheck/);
  assert.match(helper, /warm\.result,[\s\S]*cold\.result/);
});
