// Execute and retain the unchanged original Vapor gate on the pinned source.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { baseline7502, exactKeys7502, hash7502 } from "./native-attribute-values-7502-inputs.ts";

const testPath = "davinci/vize_l4/tests/native_vapor.rs";
const testSha = "5903289dae43f4efa359612a243976365bf0280536fd5dead194b8bc959f19b9";
const fixturePath = "davinci/vize_l4/tests/fixtures/native-vapor-vue-3.6.0-rc.9.json";
const fixture = new URL(`../../../${fixturePath}`, import.meta.url);
const fixtureSha = "c68a4b876c510879d0f70071e6469d175f53c0040be382e95e9e563c73636651";

export interface VaporHistory7502 {
  sourceRevision: string;
  sourceTree: string;
  testSourceSha256: string;
  fixtureSha256: string;
  captureSha256: string;
}
export function vaporHistory7502(history: string, worktree: string): VaporHistory7502 {
  assert.equal(hash7502(readFileSync(path.join(worktree, testPath))), testSha);
  const original = readFileSync(fixture);
  assert.equal(hash7502(original), fixtureSha);
  assert(original.equals(readFileSync(path.join(worktree, fixturePath))));
  const receipt = {
    sourceRevision: baseline7502.revision,
    sourceTree: baseline7502.tree,
    testSourceSha256: testSha,
    fixtureSha256: fixtureSha,
    captureSha256: hash7502(readFileSync(path.join(history, "vapor.capture.json"))),
  };
  validateVaporHistory7502(receipt, history);
  return receipt;
}
export function validateVaporHistory7502(receipt: VaporHistory7502, history: string) {
  exactKeys7502(receipt, [
    "sourceRevision",
    "sourceTree",
    "testSourceSha256",
    "fixtureSha256",
    "captureSha256",
  ]);
  assert.equal(receipt.sourceRevision, baseline7502.revision);
  assert.equal(receipt.sourceTree, baseline7502.tree);
  assert.equal(receipt.testSourceSha256, testSha);
  assert.equal(receipt.fixtureSha256, fixtureSha);
  const bytes = readFileSync(fixture);
  assert.equal(hash7502(bytes), fixtureSha);
  const frozen = JSON.parse(bytes.toString("utf8"));
  assert.equal(frozen.version, "3.6.0-rc.9");
  const capture = readFileSync(path.join(history, "vapor.capture.json"));
  assert.equal(receipt.captureSha256, hash7502(capture));
  const actual = JSON.parse(capture.toString("utf8"));
  assert.equal(actual.length, 12);
  assert.equal(frozen.fixtures.length, 12);
  for (const [index, row] of actual.entries()) {
    exactKeys7502(row, ["id", "source", "code", "map", "nodes", "roots"]);
    const expected = frozen.fixtures[index];
    assert.deepEqual(
      { id: row.id, source: row.source, code: row.code, map: row.map },
      { id: expected.id, source: expected.source, code: expected.code, map: expected.map },
      "all twelve original historical Vapor modules and complete maps stay exact",
    );
    if (row.id === "empty") {
      assert.equal(row.source, "<template></template>");
      assert.deepEqual([row.nodes, row.roots], [0, 0]);
    } else {
      assert(Number.isInteger(row.nodes) && row.nodes > 0);
      assert(Number.isInteger(row.roots) && row.roots > 0 && row.roots <= row.nodes);
    }
  }
}
