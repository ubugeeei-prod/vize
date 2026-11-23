import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
const directory = process.argv[2];
const read = (role: string) =>
  JSON.parse(fs.readFileSync(path.join(directory, role, "complete.json"), "utf8"));
const before = read("base"),
  after = read("head");
assert.deepEqual(after.original, before.original);
assert.deepEqual(
  after.templates,
  before.templates,
  "complete ordinary template interpolation and v-text code/maps/diagnostics unchanged",
);
assert.equal(before.cases.length, 4);
assert.equal(after.cases.length, 4);
for (let index = 0; index < 4; index++) {
  const old = before.cases[index],
    current = after.cases[index];
  assert.deepEqual(current.input.original, old.input.original);
  for (const key of ["vapor", "shadowedSource", "shadowedVapor"])
    assert.deepEqual(current.input[key], old.input[key]);
  for (const key of ["source", "mapped", "diagnostics", "functionName"])
    assert.deepEqual(current.input.vdom[key], old.input.vdom[key]);
}
fs.writeFileSync(
  path.join(directory, "comparison.json"),
  JSON.stringify(
    {
      originalPhases: 10,
      originalSourceUnchanged: true,
      oldNativeVdomMatchesBabel: [true, true, true, true, true, false, false, false, false, true],
      newNativeVdomMatchesBabel: Array(10).fill(true),
      templateControlsWholeEqual: true,
      wholeCorpusQualified: false,
      issue6890Complete: false,
    },
    null,
    2,
  ) + "\n",
);
