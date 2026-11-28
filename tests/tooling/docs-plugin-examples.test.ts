import assert from "node:assert/strict";
import { test } from "node:test";

import {
  markdownSourceExamples,
  typecheckMarkdownExamples,
} from "../../npm/ui/scripts/reference-docs/markdown-source-examples.ts";

const examples = ["npm/plugin-sdk/README.md", "npm/devtools/README.md"].flatMap(
  markdownSourceExamples,
);

void test("every displayed SDK and devtools recipe typechecks against public contracts", () => {
  assert.ok(examples.length >= 11, "retain all existing authoring and host examples");
  assert.deepEqual(typecheckMarkdownExamples(examples), []);
});

void test("the documentation gate rejects an unsupported API version in the displayed source", () => {
  const original = examples.find((example) => example.source.includes("apiVersion: 2"));
  assert.ok(original);
  const invalid = {
    ...original,
    source: original.source.replace("apiVersion: 2", 'apiVersion: "2"'),
  };
  assert.ok(typecheckMarkdownExamples([invalid]).some((diagnostic) => /TS2322/u.test(diagnostic)));
});
