import assert from "node:assert/strict";
import { test } from "node:test";

import { scanConsumerMigrationSurfaces } from "../../legacy-tools/davinci/lib/consumer-migration-scan.mjs";

const expressionRows = [
  ["crates/vize_atelier_core/src/steps/expression.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/expression/collector.rs", "source", 2],
  ["crates/vize_atelier_core/src/steps/expression/inline_handler.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/expression/inline_handler.rs", "test", 1],
  ["crates/vize_atelier_core/src/steps/expression/nesting.rs", "source", 2],
  ["crates/vize_atelier_core/src/steps/expression/parse_checks.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/expression/prefix.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/expression/reparse.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/expression/retained_rewrite.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/expression/rewrite.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/expression/splice.rs", "source", 1],
  ["crates/vize_atelier_core/src/steps/expression/splice.rs", "test", 1],
  ["crates/vize_atelier_core/src/steps/expression/tests.rs", "test", 1],
  ["crates/vize_atelier_core/src/steps/expression/typescript.rs", "source", 1],
];

function compiler() {
  const scan = scanConsumerMigrationSurfaces();
  const consumer = scan.consumers.find((candidate) => candidate.id === "compiler");
  assert.ok(consumer);
  return consumer;
}

void test("Atelier core expression transform steps import S0 through the preferred name", () => {
  const rows = compiler().fileRows;

  for (const [relPath, mode, sites] of expressionRows) {
    const row = rows.find((candidate) => candidate.relPath === relPath && candidate.mode === mode);
    assert.ok(row, `${relPath} (${mode})`);
    assert.equal(row.surfaceCounts.s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_carton ?? 0, 0, relPath);
  }

  const compatRows = rows
    .filter((row) => row.relPath.startsWith("crates/vize_atelier_core/src/steps/expression"))
    .filter((row) => (row.surfaceNameCounts.s0.vize_carton ?? 0) > 0)
    .map((row) => `${row.relPath}:${row.mode}`);
  assert.deepEqual(compatRows, []);
});
