import assert from "node:assert/strict";
import { test } from "node:test";
import { frozenCorpus } from "./css-external-target-reference.mjs";

await test("CSS target corpus retains both distinct originals and whole conservative controls", () => {
  const { cases } = frozenCorpus();
  const row = (id) => cases.find((entry) => entry.id === id);
  assert.notEqual(row("original-7976").source, row("original-7984").source);
  for (const id of [
    "local",
    "mixed-targets",
    "mixed-parent",
    "has-local",
    "not-local",
    "global-local",
    "root-local",
    "sibling-subject",
    "nested-sibling",
    "attribute-string",
    "inherited-is-sibling",
    "inherited-where-sibling",
    "inherited-is-mixed",
    "inherited-where-mixed",
    "inherited-has-local",
    "inherited-not-local",
    "local-flat-sibling-descendant",
    "local-nested-sibling-descendant",
  ])
    assert.equal(row(id).expectedCli[0].warningCount, 1);
  for (const id of [
    "original-7976",
    "original-7984",
    "slotted",
    "external-nested",
    "external-layer",
    "is-external",
    "where-external",
    "inherited-is-target",
    "inherited-where-target",
    "external-nested-adjacent",
    "external-nested-later",
    "external-flat-adjacent",
    "external-flat-child-adjacent",
  ])
    assert.deepEqual(row(id).expectedCli[0].messages, []);
  assert.equal(row("ordered-mixed").expectedCli[0].messages.length, 2);
  assert.equal(row("utf8-crlf").source.includes("\r\n"), true);
});
