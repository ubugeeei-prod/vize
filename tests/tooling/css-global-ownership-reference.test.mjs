import assert from "node:assert/strict";
import { test } from "node:test";
import { globalCorpus } from "./css-global-ownership-reference.mjs";

await test("global ownership corpus retains original history and complete conservative controls", () => {
  const { source, cases } = globalCorpus();
  assert.equal(source.history.length, 11);
  const byId = new Map(cases.map((row) => [row.id, row]));
  for (const id of [
    "foreign-class-fragment",
    "foreign-id-fragment",
    "foreign-tag-single-root",
    "empty-static-class",
    "escaped-foreign-class",
    "mixed-foreign-globals",
    "mixed-deep-global-foreign",
    "foreign-layer",
    "comment-name-foreign",
  ]) {
    assert.deepEqual(byId.get(id).expectedStarts, [], id);
    assert.equal(byId.get(id).expectedCssStarts.length, 1, id);
    assert.equal(
      byId.get(id).expectedPlain,
      "Patina lint report: No problems found in 1 file(s)\n",
    );
  }
  for (const row of cases.filter((row) => row.expectedStarts.length > 0)) {
    assert.equal(row.expectedCli[0].warningCount, 1, row.id);
    assert.equal(row.expectedCli[0].messages[0].ruleId, "css/no-display-none", row.id);
    assert.equal(
      row.expectedCli[0].messages[0].help,
      "v-show toggles visibility without removing from DOM, improving performance for frequent toggles",
    );
  }
  assert.equal(byId.get("nested-global-local-child").expectedCssStarts.length, 2);
  assert.deepEqual(byId.get("nested-global-local-child").expectedStarts, [
    byId.get("nested-global-local-child").expectedCssStarts[1],
  ]);
});
