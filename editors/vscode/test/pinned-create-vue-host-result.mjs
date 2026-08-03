import assert from "node:assert/strict";
import fs from "node:fs";

const expectedResult = {
  brokenDiagnostics: [
    {
      code: 2322,
      message: "Type 'string' is not assignable to type 'number'.",
      range: [1, 6, 1, 11],
      severity: 0,
      source: "vize/types",
    },
  ],
  documentDirty: true,
  extensionActive: true,
  fixtureId: "create-vue",
  repairedDiagnostics: [],
  schemaVersion: 1,
};

export function readPinnedCreateVueHostResult(resultPath) {
  assert.ok(fs.existsSync(resultPath), `packaged host did not write its result: ${resultPath}`);
  const actual = JSON.parse(fs.readFileSync(resultPath, "utf8"));
  assert.deepEqual(actual, expectedResult);
  return actual;
}
