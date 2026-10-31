/** Actual hosted output successors retain every original VDOM snapshot as historical evidence. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
const root = new URL("../../", import.meta.url);
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const path = "tests/_fixtures/differential/jsx/child-values-6890-vdom-snapshot-successors.json";
const bytes = fs.readFileSync(new URL(path, root));
const pinned = "976e6879731eab447beb11d530dceb0a5d7bdfebfc94f93973e69444590f9376";
const record = JSON.parse(bytes.toString()) as {
  schema: number;
  source: string;
  run: number;
  artifact: number;
  archiveSha256: string;
  originalInputsChanged: boolean;
  runtimeCredit: boolean;
  performanceCredit: boolean;
  wholeCorpusQualified: boolean;
  rows: {
    original: string;
    originalSha256: string;
    successor: string;
    successorSha256: string;
    member: string;
    body: string;
  }[];
};
function verify(read: (path: string) => Buffer): void {
  assert.equal(hash(bytes), pinned);
  assert.equal(record.schema, 1);
  assert.equal(record.source, "a9b493e2ec31426eaa75a22fd23c1f04d9b586fa");
  assert.equal(record.run, 41210732285);
  assert.equal(record.artifact, 13046590004);
  assert.equal(
    record.archiveSha256,
    "c8865710139f5261c5533f8d9a2333ad5c9998d8bb4ea1991d131ab33881854c",
  );
  for (const key of [
    "originalInputsChanged",
    "runtimeCredit",
    "performanceCredit",
    "wholeCorpusQualified",
  ] as const)
    assert.equal(record[key], false);
  assert.equal(record.rows.length, 10);
  assert.equal(new Set(record.rows.map((row) => row.original)).size, 10);
  for (const row of record.rows) {
    assert.equal(row.successor, row.original.replace(/\.snap$/, "_vdom_child_values_6890.snap"));
    assert.equal(hash(read(row.original)), row.originalSha256);
    assert.equal(hash(read(row.successor)), row.successorSha256);
    assert.notEqual(row.originalSha256, row.successorSha256);
  }
}
const read = (path: string) => fs.readFileSync(new URL(path, root));
test("all ten genuine output successors retain complete original snapshot bytes", () =>
  verify(read));
test("custody rejects original or successor byte mutations", () => {
  for (const row of record.rows)
    for (const path of [row.original, row.successor])
      assert.throws(() =>
        verify((file) =>
          file === path ? Buffer.concat([read(file), Buffer.from("changed")]) : read(file),
        ),
      );
});
