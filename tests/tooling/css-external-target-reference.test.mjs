import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fixture, frozenCorpus } from "./css-external-target-reference.mjs";

await test("authored docs-path correction preserves all other complete reference bytes", () => {
  const { source } = frozenCorpus();
  const beforeBytes = fs.readFileSync(path.join(fixture, "cases.before-docs-path.json"), "utf8");
  const afterBytes = fs.readFileSync(path.join(fixture, "cases.json"), "utf8");
  assert.equal(
    afterBytes.replaceAll(
      '"ruleDocsPath": "docs/content/rules/musea-and-css.md"',
      '"ruleDocsPath": "docs/content/rules/css.md"',
    ),
    beforeBytes,
  );
  const before = JSON.parse(beforeBytes);
  const after = JSON.parse(afterBytes);
  let changed = 0;
  for (const row of before.cases)
    for (const report of row.expectedCli)
      for (const message of report.messages) {
        assert.equal(message.ruleDocsPath, "docs/content/rules/css.md");
        message.ruleDocsPath = "docs/content/rules/musea-and-css.md";
        changed++;
      }
  assert.equal(changed, 27);
  assert.equal(source.referenceCorrections[0].scalarCount, changed);
  assert.deepEqual(after, before);
});

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
    "farther-flat-deep",
    "farther-nested-deep",
    "farther-flat-slotted",
    "farther-nested-slotted",
    "farther-flat-is",
    "farther-nested-where",
    "farther-explicit-before-nesting",
  ])
    assert.deepEqual(row(id).expectedCli[0].messages, []);
  assert.equal(row("ordered-mixed").expectedCli[0].messages.length, 2);
  assert.equal(row("utf8-crlf").source.includes("\r\n"), true);
});
