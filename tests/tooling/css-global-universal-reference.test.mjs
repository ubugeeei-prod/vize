import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { compoundCorpus, root } from "./css-global-compound-reference.mjs";
import { universalCorpus } from "./css-global-universal-reference.mjs";

await test("universal ownership freezes every original and authors only two whole deltas", () => {
  const { source, cases } = universalCorpus();
  assert.equal(source.history.length, 18);
  assert.equal(cases.filter((row) => row.expectedStarts.length === 0).length, 11);
  assert.equal(cases.filter((row) => row.expectedStarts.length > 0).length, 32);
  const original = JSON.parse(
    fs.readFileSync(
      path.join(root, "crates/vize_patina/tests/fixtures/issue-7976-compound/cases.json"),
      "utf8",
    ),
  ).cases;
  const after = compoundCorpus().cases;
  assert.equal(original.length, 62);
  assert.equal(after.length, 62);
  assert.deepEqual(
    after
      .filter((row, index) => JSON.stringify(row) !== JSON.stringify(original[index]))
      .map((row) => row.id)
      .sort(),
    ["inner-universal", "outer-universal"],
  );
  for (const id of ["inner-universal", "outer-universal"]) {
    const row = after.find((row) => row.id === id);
    assert.deepEqual(row.expectedStarts, [], id);
    assert.deepEqual(row.expectedCssStarts, [152], id);
    assert.deepEqual(row.expectedCli, [
      { file: row.filename, messages: [], errorCount: 0, warningCount: 0 },
    ]);
    assert.equal(row.expectedPlain, "Patina lint report: No problems found in 1 file(s)\n");
  }
  const byId = new Map(cases.map((row) => [row.id, row]));
  for (const id of [
    "local-class",
    "local-id",
    "bare-universal",
    "outer-absent-inner-local",
    "inner-any-namespace",
    "inner-empty-namespace",
    "outer-any-namespace",
    "outer-empty-namespace",
    "inner-descendant",
    "inner-child",
    "inner-sibling",
    "inner-attribute",
    "inner-pseudo",
    "inner-comment",
    "inner-whitespace",
    "multiple-globals",
    "repeated-star",
    "nonleading-star",
    "comma-in-global",
    "mixed-selector-list",
    "foreign-filter-local-subject",
    "single-root-fallthrough",
    "dynamic-class-unknown",
    "component-root-unknown",
    "foreign-namespace-template",
    "external-template-unknown",
    "non-html-template-unknown",
  ]) {
    const row = byId.get(id);
    assert.equal(row.expectedStarts.length, 1, id);
    assert.equal(row.expectedCli[0].warningCount, 1, id);
  }
  const nested = byId.get("foreign-global-parent-local-child");
  assert.equal(nested.expectedCssStarts.length, 2);
  assert.deepEqual(nested.expectedStarts, [nested.expectedCssStarts[1]]);
});
