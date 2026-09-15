import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { repoRoot } from "./_helpers/moonbit.ts";

const referenceHeadings = [
  "## Entry Points",
  "## Flag Contracts",
  "## Direct API Fields",
  "## Config Recipes",
  "## Failure Examples",
  "## Slot Cardinality",
  "## Implementation Coverage",
  "## Release Safety Checklist",
];

const rfcLinks = [
  "https://github.com/vuejs/rfcs/pull/823",
  "https://github.com/vuejs/rfcs/pull/831",
  "https://github.com/vuejs/rfcs/pull/833",
  "https://github.com/vuejs/rfcs/pull/734",
];

test("experimentals reference documents entrypoints, proofs, and low-level APIs", () => {
  const reference = fs.readFileSync(
    path.join(repoRoot, "docs/content/guide/experimentals-reference.md"),
    "utf8",
  );

  assertReference(reference);
  assert.match(reference, /No aliases, no `\{\}` switch object/);
  assert.match(reference, /caller owns those rules/);
  assert.match(reference, /Do not group unrelated RFC switches/);
  assert.match(reference, /do not promote an experimental flag to default-on/);
  assert.match(reference, /`patternedTemplate` stays fail-closed/);
  assert.match(reference, /`inTagComment` is not a second expression grammar/);
  assert.match(reference, /direct Vite plugin values can enable and explicitly opt out/);
  assert.match(
    reference,
    /`false`, `null`, `true`, and `\{\}` keep the documented switch semantics/,
  );
});

test("Japanese experimentals reference mirrors entrypoints and proofs", () => {
  const reference = fs.readFileSync(
    path.join(repoRoot, "docs/content/ja/guide/experimentals-reference.md"),
    "utf8",
  );

  assertReference(reference);
  assert.match(reference, /alias なし、`\{\}` switch object なし/);
  assert.match(reference, /caller がその解決ルールを所有/);
  assert.match(reference, /無関係な RFC switch を 1 つの/);
  assert.match(reference, /experimental flag を shared config で default-on に昇格しません/);
  assert.match(reference, /`patternedTemplate` は構造が違うと fail-closed/);
  assert.match(reference, /第 2 の expression grammar ではありません/);
  assert.match(reference, /direct Vite plugin value が shared config を有効化でき/);
  assert.match(reference, /`false`、`null`、`true`、`\{\}` が documented switch semantics/);
});

function assertReference(reference: string): void {
  for (const heading of referenceHeadings) {
    assert.match(reference, new RegExp(escapeRegExp(heading)), `${heading} must be documented`);
  }
  for (const link of rfcLinks) {
    assert.match(reference, new RegExp(escapeRegExp(link)), `${link} must be linked`);
  }

  assert.match(reference, /`compile`, `compileVapor`, `parseTemplate`/);
  assert.match(reference, /`compileSfc`, `compileSfcBatch`, `compileSfcBatchWithResults`/);
  assert.match(reference, /Smallest useful proof|最小の proof/);
  assert.match(reference, /\(\) => \[HTMLInputElement, HTMLInputElement\]/);
  assert.match(reference, /Boundary tests?|Boundary test/);
  assert.doesNotMatch(reference, /compileTemplate\(source, \{/);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
