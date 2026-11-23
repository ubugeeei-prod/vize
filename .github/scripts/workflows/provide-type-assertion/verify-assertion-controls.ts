import assert from "node:assert/strict";
import { join } from "node:path";
import { readJson, requiredEnv } from "../shared/files.ts";

import { statSync } from "node:fs";
const root = requiredEnv("PACKETS");
const rows = readJson<{ id: string; processes: { command: string }[] }[]>(
  join(root, "manifest.json"),
);
assert.equal(rows.length, 27);
assert.equal(new Set(rows.map((row) => row.id)).size, 27);
for (const row of rows) {
  for (const file of [
    "native.raw.txt",
    "native.complete.txt",
    "native.registry.txt",
    "native.graph.txt",
    "doctor.stdout",
    "doctor.stderr",
    "doctor.json",
    "lint.stdout",
    "lint.stderr",
    "lint.json",
    "processes.json",
  ])
    assert.ok(statSync(join(root, row.id, file)).isFile());
  assert.deepEqual(
    row.processes.map((p) => p.command),
    ["doctor", "lint"],
  );
}
if (requiredEnv("ROLE") === "source")
  assert.deepEqual(
    readJson(join(root, "complete-observations.json")),
    readJson("crates/vize/tests/provide_type_assertion_expected.json"),
  );
