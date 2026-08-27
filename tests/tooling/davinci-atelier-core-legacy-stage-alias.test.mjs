import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { scanConsumerMigrationSurfaces } from "../../tools/davinci/lib/consumer-migration-scan.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const legacyRows = [
  ["crates/vize_atelier_core/src/steps/legacy.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/legacy_filters.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/legacy/tests.rs", "test", 8],
];

function compiler() {
  const scan = scanConsumerMigrationSurfaces();
  const consumer = scan.consumers.find((candidate) => candidate.id === "compiler");
  assert.ok(consumer);
  return consumer;
}

void test("Atelier core legacy steps import S0 through the preferred name", () => {
  const rows = compiler().fileRows;

  for (const [relPath, mode, sites] of legacyRows) {
    const row = rows.find((candidate) => candidate.relPath === relPath && candidate.mode === mode);
    assert.ok(row, `${relPath} (${mode})`);
    assert.equal(row.surfaceCounts.s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_carton ?? 0, 0, relPath);

    const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
    assert.doesNotMatch(source, /\bvize_carton\b/u, relPath);
  }

  const compatRows = rows
    .filter((row) => row.relPath.startsWith("crates/vize_atelier_core/src/steps/legacy"))
    .filter((row) => (row.surfaceNameCounts.s0.vize_carton ?? 0) > 0)
    .map((row) => `${row.relPath}:${row.mode}`);
  assert.deepEqual(compatRows, []);
});
