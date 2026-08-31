import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { scanConsumerMigrationSurfaces } from "../../tools/davinci/lib/consumer-migration-scan.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const templateRefRows = [
  ["crates/vize_canon/src/virtual_ts/generator/template_refs.rs", "source", 2],
  ["crates/vize_canon/src/virtual_ts/generator/template_refs/auto_imports.rs", "source", 1],
  ["crates/vize_canon/src/virtual_ts/generator/template_refs/auto_imports.rs", "test", 2],
  ["crates/vize_canon/src/virtual_ts/generator/template_refs/deferred_bindings.rs", "source", 1],
];

function typechecker() {
  const scan = scanConsumerMigrationSurfaces();
  const consumer = scan.consumers.find((candidate) => candidate.id === "typechecker");
  assert.ok(consumer);
  return consumer;
}

void test("Canon template ref generation imports S0 through the preferred name", () => {
  const rows = typechecker().fileRows;

  for (const [relPath, mode, sites] of templateRefRows) {
    const row = rows.find((candidate) => candidate.relPath === relPath && candidate.mode === mode);
    assert.ok(row, `${relPath} (${mode})`);
    assert.equal(row.surfaceCounts.s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_carton ?? 0, 0, relPath);

    const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
    assert.doesNotMatch(source, /\bvize_carton\b/u, relPath);
  }
});
