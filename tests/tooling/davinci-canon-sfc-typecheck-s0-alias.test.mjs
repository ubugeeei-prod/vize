import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { scanConsumerMigrationSurfaces } from "../../legacy-tools/davinci/lib/consumer-migration-scan.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const sfcTypecheckRows = [
  ["crates/vize_canon/src/sfc_typecheck/analysis.rs", 1],
  ["crates/vize_canon/src/sfc_typecheck/checks.rs", 1],
  ["crates/vize_canon/src/sfc_typecheck/runner.rs", 2],
  ["crates/vize_canon/src/sfc_typecheck/virtual_ts.rs", 1],
];

function typechecker() {
  const scan = scanConsumerMigrationSurfaces();
  const consumer = scan.consumers.find((candidate) => candidate.id === "typechecker");
  assert.ok(consumer);
  return consumer;
}

void test("Canon SFC type checking imports S0 through the preferred name", () => {
  const rows = typechecker().fileRows;

  for (const [relPath, sites] of sfcTypecheckRows) {
    const row = rows.find(
      (candidate) => candidate.relPath === relPath && candidate.mode === "source",
    );
    assert.ok(row, relPath);
    assert.equal(row.surfaceCounts.s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_carton ?? 0, 0, relPath);

    const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
    assert.doesNotMatch(source, /\bvize_carton\b/u, relPath);
  }
});
