import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registry = JSON.parse(
  fs.readFileSync(path.join(root, "tests/_fixtures/vue-ecosystem-fixtures.json"), "utf8"),
) as {
  projects: Array<{
    id: string;
    batchIncrementalBudget?: {
      coldMs: number;
      warmMs: number;
      maxRequestedFiles: number;
      maxChangedFiles: number;
    };
  }>;
};

test("Vben owns the Tier-L batch incremental work budget", () => {
  const owners = registry.projects.filter((project) => project.batchIncrementalBudget != null);
  assert.deepEqual(
    owners.map((project) => project.id),
    ["vue-vben-admin"],
  );
  assert.deepEqual(owners[0].batchIncrementalBudget, {
    coldMs: 15_000,
    warmMs: 10_000,
    maxRequestedFiles: 500,
    maxChangedFiles: 1,
  });
});

test("the Tier-L oracle gates deterministic incremental work and emits evidence", () => {
  const source = [
    "crates/vize_canon/tests/tier_l_incremental.rs",
    "crates/vize_canon/tests/support/tier_l_incremental_artifact.rs",
  ]
    .map((file) => fs.readFileSync(path.join(root, file), "utf8"))
    .join("\n");
  assert.match(source, /check_incremental/);
  assert.match(source, /session_to_cli_fallbacks, 0/);
  assert.match(source, /last_changed_files, budget\.max_changed_files/);
  assert.match(source, /last_requested_files <= budget\.max_requested_files/);
  assert.match(source, /VIZE_TIER_L_BUDGET_SCALE/);
  assert.match(source, /budget_scale/);
  assert.match(source, /metrics\.json/);
  assert.match(source, /summary\.md/);
  assert.doesNotMatch(source, /broken_ms\s*[<>]=?\s*cold_ms/);
  assert.doesNotMatch(source, /repaired_ms\s*[<>]=?\s*cold_ms/);
});
