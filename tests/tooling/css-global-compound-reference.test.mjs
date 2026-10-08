import assert from "node:assert/strict";
import { test } from "node:test";
import { compoundCorpus } from "./css-global-compound-reference.mjs";
import { globalCorpus } from "./css-global-ownership-reference.mjs";

await test("compound ownership freezes all originals and changes only two whole semantic packets", () => {
  const { source, cases } = compoundCorpus();
  assert.equal(source.history.length, 13);
  assert.equal(cases.filter((row) => row.expectedStarts.length === 0).length, 21);
  assert.equal(cases.filter((row) => row.expectedStarts.length > 0).length, 41);
  const old = new Map(globalCorpus().cases.map((row) => [row.id, row]));
  for (const id of ["global-compound-unknown", "outer-compound-unknown"]) {
    assert.deepEqual(old.get(id).expectedStarts, [], id);
    assert.equal(old.get(id).expectedCssStarts.length, 1, id);
    assert.equal(old.get(id).expectedPlain, "Patina lint report: No problems found in 1 file(s)\n");
  }
  const byId = new Map(cases.map((row) => [row.id, row]));
  for (const id of [
    "outer-missing-inner-local-class",
    "outer-missing-inner-local-id",
    "outer-missing-inner-local-type",
    "absent-first-then-pseudo",
    "absent-first-then-attribute",
    "absent-first-then-list",
    "multiple-global-foreign-first",
    "multiple-global-local-first",
  ]) {
    assert.equal(byId.get(id).expectedStarts.length, 1, id);
    assert.equal(byId.get(id).expectedCli[0].warningCount, 1, id);
  }
  const nested = byId.get("global-parent-local-child");
  assert.equal(nested.expectedCssStarts.length, 2);
  assert.deepEqual(nested.expectedStarts, [nested.expectedCssStarts[1]]);
});
