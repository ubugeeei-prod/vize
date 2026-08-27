import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { scanConsumerMigrationSurfaces } from "../../tools/davinci/lib/consumer-migration-scan.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const internalTestRows = [
  ["crates/vize_atelier_core/src/codegen/tests.rs", "test", 20],
  ["crates/vize_atelier_core/src/lane/tests.rs", "test", 1],
  ["crates/vize_atelier_core/src/lane/structural/tests.rs", "test", 1],
  ["crates/vize_atelier_core/src/retained/tests.rs", "test", 1],
];

function compiler() {
  const scan = scanConsumerMigrationSurfaces();
  const consumer = scan.consumers.find((candidate) => candidate.id === "compiler");
  assert.ok(consumer);
  return consumer;
}

void test("Atelier core internal tests import S0 through the preferred name", () => {
  const rows = compiler().fileRows;

  for (const [relPath, mode, sites] of internalTestRows) {
    const row = rows.find((candidate) => candidate.relPath === relPath && candidate.mode === mode);
    assert.ok(row, `${relPath} (${mode})`);
    assert.equal(row.surfaceCounts.s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_carton ?? 0, 0, relPath);

    const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
    assert.doesNotMatch(source, /\bvize_carton\b/u, relPath);
  }

  const migratedPaths = new Set(internalTestRows.map(([relPath]) => relPath));
  const compatRows = rows
    .filter((row) => migratedPaths.has(row.relPath))
    .filter((row) => (row.surfaceNameCounts.s0.vize_carton ?? 0) > 0)
    .map((row) => `${row.relPath}:${row.mode}`);
  assert.deepEqual(compatRows, []);
});
