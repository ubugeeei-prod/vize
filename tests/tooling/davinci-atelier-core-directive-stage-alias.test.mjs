import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { scanConsumerMigrationSurfaces } from "../../legacy-tools/davinci/lib/consumer-migration-scan.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const directiveRows = [
  ["crates/vize_atelier_core/src/steps/text.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/v_bind.rs", "source", 2],
  ["crates/vize_atelier_core/src/steps/v_for.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/v_for.rs", "test", 1],
  ["crates/vize_atelier_core/src/steps/v_if.rs", "test", 1],
  ["crates/vize_atelier_core/src/steps/v_memo.rs", "source", 2],
  ["crates/vize_atelier_core/src/steps/v_memo.rs", "test", 1],
  ["crates/vize_atelier_core/src/steps/v_model.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/v_model.rs", "test", 1],
  ["crates/vize_atelier_core/src/steps/v_on.rs", "source", 2],
  ["crates/vize_atelier_core/src/steps/v_once.rs", "source", 2],
  ["crates/vize_atelier_core/src/steps/v_once.rs", "test", 1],
  ["crates/vize_atelier_core/src/steps/v_slot.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/v_slot/params.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/v_slot/tests.rs", "test", 1],
  ["crates/vize_atelier_core/src/steps/v_slot/validate.rs", "source", 1],
];

function compiler() {
  const scan = scanConsumerMigrationSurfaces();
  const consumer = scan.consumers.find((candidate) => candidate.id === "compiler");
  assert.ok(consumer);
  return consumer;
}

void test("Atelier core directive steps import S0 through the preferred name", () => {
  const rows = compiler().fileRows;

  for (const [relPath, mode, sites] of directiveRows) {
    const row = rows.find((candidate) => candidate.relPath === relPath && candidate.mode === mode);
    assert.ok(row, `${relPath} (${mode})`);
    assert.equal(row.surfaceCounts.s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_carton ?? 0, 0, relPath);

    const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
    assert.doesNotMatch(source, /\bvize_carton\b/u, relPath);
  }

  const migratedPaths = new Set(directiveRows.map(([relPath]) => relPath));
  const compatRows = rows
    .filter((row) => migratedPaths.has(row.relPath))
    .filter((row) => (row.surfaceNameCounts.s0.vize_carton ?? 0) > 0)
    .map((row) => `${row.relPath}:${row.mode}`);
  assert.deepEqual(compatRows, []);
});
