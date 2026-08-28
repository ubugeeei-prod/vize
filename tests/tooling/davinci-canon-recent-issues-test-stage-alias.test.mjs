import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { scanConsumerMigrationSurfaces } from "../../tools/davinci/lib/consumer-migration-scan.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const recentIssueRows = [
  ["crates/vize_canon/src/batch/type_checker/tests/recent_issues/directive_anchors.rs", "test", 1],
  ["crates/vize_canon/src/batch/type_checker/tests/recent_issues/directive_values.rs", "test", 1],
  [
    "crates/vize_canon/src/batch/type_checker/tests/recent_issues/external_slot_payloads.rs",
    "test",
    1,
  ],
  [
    "crates/vize_canon/src/batch/type_checker/tests/recent_issues/fallthrough_unknown_attrs.rs",
    "test",
    1,
  ],
  ["crates/vize_canon/src/batch/type_checker/tests/recent_issues/generic_emit_guard.rs", "test", 1],
  [
    "crates/vize_canon/src/batch/type_checker/tests/recent_issues/global_html_fallthrough_attrs.rs",
    "test",
    1,
  ],
  [
    "crates/vize_canon/src/batch/type_checker/tests/recent_issues/literal_union_props.rs",
    "test",
    1,
  ],
  [
    "crates/vize_canon/src/batch/type_checker/tests/recent_issues/single_required_camel_prop.rs",
    "test",
    1,
  ],
  [
    "crates/vize_canon/src/batch/type_checker/tests/recent_issues/global_component_callbacks.rs",
    "test",
    8,
  ],
  [
    "crates/vize_canon/src/batch/type_checker/tests/recent_issues/template_handler_ts7006.rs",
    "test",
    16,
  ],
];

function typechecker() {
  const scan = scanConsumerMigrationSurfaces();
  const consumer = scan.consumers.find((candidate) => candidate.id === "typechecker");
  assert.ok(consumer);
  return consumer;
}

void test("Canon recent issue tests import S0 through the preferred name", () => {
  const rows = typechecker().fileRows;

  for (const [relPath, mode, sites] of recentIssueRows) {
    const row = rows.find((candidate) => candidate.relPath === relPath && candidate.mode === mode);
    assert.ok(row, `${relPath} (${mode})`);
    assert.equal(row.surfaceCounts.s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_s0, sites, relPath);
    assert.equal(row.surfaceNameCounts.s0.vize_carton ?? 0, 0, relPath);

    const source = fs.readFileSync(path.join(repoRoot, relPath), "utf8");
    assert.doesNotMatch(source, /\bvize_carton\b/u, relPath);
  }

  const migratedPaths = new Set(recentIssueRows.map(([relPath]) => relPath));
  const compatRows = rows
    .filter((row) => migratedPaths.has(row.relPath))
    .filter((row) => (row.surfaceNameCounts.s0.vize_carton ?? 0) > 0)
    .map((row) => `${row.relPath}:${row.mode}`);
  assert.deepEqual(compatRows, []);
});
