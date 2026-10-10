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

const japaneseReferenceHeadings = [
  "## 設定を渡す場所",
  "## フラグごとの契約",
  "## ネイティブ API の設定項目",
  "## 設定例",
  "## 拒否される例",
  "## スロットの子要素の個数",
  "## 実装の検証範囲",
  "## リリース前の確認",
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

  assertReference(reference, japaneseReferenceHeadings);
  assert.match(reference, /別名、`\{\}` の設定オブジェクト/);
  assert.match(reference, /呼び出し元がそれらの解決ルールを管理/);
  assert.match(reference, /無関係な RFC の設定を、一つの/);
  assert.match(reference, /共通設定で実験フラグを既定で有効にしません/);
  assert.match(reference, /`patternedTemplate` は構造が不正な場合、処理を拒否/);
  assert.match(reference, /属性に別の式の文法を追加する機能ではありません/);
  assert.match(reference, /Vite プラグインに直接渡す値で、共通設定を有効化でき/);
  assert.match(reference, /`false`、`null`、`true`、`\{\}` が、記載された有効・無効の意味/);
});

function assertReference(reference: string, headings = referenceHeadings): void {
  for (const heading of headings) {
    assert.match(reference, new RegExp(escapeRegExp(heading)), `${heading} must be documented`);
  }
  for (const link of rfcLinks) {
    assert.match(reference, new RegExp(escapeRegExp(link)), `${link} must be linked`);
  }

  assert.match(reference, /`compile`, `compileVapor`, `parseTemplate`/);
  assert.match(reference, /`compileSfc`, `compileSfcBatch`, `compileSfcBatchWithResults`/);
  assert.match(reference, /Smallest useful proof|最小限の検証/);
  assert.match(reference, /\(\) => \[HTMLInputElement, HTMLInputElement\]/);
  assert.match(reference, /Boundary tests?|対応範囲の境界/);
  assert.doesNotMatch(reference, /compileTemplate\(source, \{/);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
