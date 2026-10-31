/** Separate actual dynamic-slot successors preserve the ten earlier producer snapshots. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
const root = new URL("../../", import.meta.url);
const hash = (b: Uint8Array) => createHash("sha256").update(b).digest("hex");
const read = (path: string) => fs.readFileSync(new URL(path, root));
const path =
  "tests/_fixtures/differential/jsx/child-values-6890-vdom-slot-snapshot-successors.json";
const bytes = read(path);
const data = JSON.parse(bytes.toString()) as {
  schema: number;
  source: string;
  run: number;
  job: number;
  artifact: number;
  archiveBytes: number;
  archiveSha256: string;
  producerConclusion: string;
  historicalPredecessor: string;
  originalInputsChanged: boolean;
  runtimeCredit: boolean;
  performanceCredit: boolean;
  wholeCorpusQualified: boolean;
  rows: {
    predecessor: string;
    predecessorSha256: string;
    successor: string;
    successorSha256: string;
    member: string;
  }[];
};
function verify(reader: (path: string) => Buffer) {
  assert.equal(hash(bytes), "c73b1c61ae6574612768ae820088764d30a1b7381d1dc4ec663bff0b3b21a85f");
  assert.equal(data.schema, 1);
  assert.equal(data.source, "4b9b4cb797acc412052c6917ef63813b7e5b2088");
  assert.equal(data.run, 41215542556);
  assert.equal(data.job, 124579517264);
  assert.equal(data.artifact, 13048091694);
  assert.equal(data.archiveBytes, 415080);
  assert.equal(
    data.archiveSha256,
    "62d99d5ae77bbc8d8d49d09789ce194599d683725cc18c44ee130d7591eae7d5",
  );
  assert.equal(
    data.producerConclusion,
    "failure",
    "do not transfer whole-workflow credit from a historical output capture",
  );
  for (const key of [
    "originalInputsChanged",
    "runtimeCredit",
    "performanceCredit",
    "wholeCorpusQualified",
  ] as const)
    assert.equal(data[key], false);
  const old = JSON.parse(reader(data.historicalPredecessor).toString()) as {
    rows: { successor: string; successorSha256: string }[];
  };
  assert.equal(data.rows.length, 2);
  assert.equal(new Set(data.rows.map((r) => r.predecessor)).size, 2);
  for (const row of data.rows) {
    assert.equal(
      row.predecessorSha256,
      old.rows.find((r) => r.successor === row.predecessor)?.successorSha256,
    );
    assert.equal(
      row.successor,
      row.predecessor.replace("_vdom_child_values_6890.snap", "_vdom_dynamic_slots_6890.snap"),
    );
    assert.equal(hash(reader(row.predecessor)), row.predecessorSha256);
    assert.equal(hash(reader(row.successor)), row.successorSha256);
    assert.notEqual(row.predecessorSha256, row.successorSha256);
  }
}
test("two complete real dynamic-slot successors retain every earlier snapshot preimage", () =>
  verify(read));
test("dynamic-slot custody rejects a changed predecessor or actual output", () => {
  for (const row of data.rows)
    for (const path of [row.predecessor, row.successor])
      assert.throws(() =>
        verify((file) =>
          file === path ? Buffer.concat([read(file), Buffer.from("changed")]) : read(file),
        ),
      );
});
