import assert from "node:assert/strict";
import { originalConfigDocument } from "./support/docs-config-archive.ts";
import { currentConfigExample } from "./support/docs-config-successor.ts";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

void test("Japanese experimentals guide retains complete recipes, syntax and public destinations", async () => {
  const content = resolve(import.meta.dirname, "../../docs/content");
  const path = "guide/experimentals.md";
  const english = readFileSync(resolve(content, path), "utf8");
  const japanese = readFileSync(resolve(content, "ja", path), "utf8");
  const retained = await collectReviewedTranslations([path], resolve(content, "ja"));
  assert.equal(retained.get(path), japanese, "regeneration retains the complete authored review");

  const fences = (source: string) =>
    [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```/gm)].map((match) => match[0]);
  const digest = (source: string) => createHash("sha256").update(source).digest("hex");
  const archived = originalConfigDocument(`docs/content/ja/${path}`);
  assert.deepEqual(
    fences(japanese),
    fences(archived).map(currentConfigExample),
    "all complete successor recipes preserve the original values",
  );
  // All seventeen complete examples at signed main 1d988f6bcac8 are retained.
  assert.equal(fences(japanese).length, 17);
  assert.equal(
    digest(fences(archived).join("\n")),
    "1d8014b7305aadbb35a77e021fceda68c537da1828f5f7a56d29877c01391d78",
  );
  assert.deepEqual(fences(japanese), fences(english));
  const hrefs = (source: string) =>
    [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]);
  assert.equal(
    digest(
      JSON.stringify(
        hrefs(japanese).filter((href) => href !== "https://vizejs.dev/play/?tab=canon"),
      ),
    ),
    "0efc7938b4c7c23befdc5968019f3bb805db82fc39db763ebdacf58167cc5325",
    "all original eleven destinations retain their order and multiplicity",
  );
  assert.equal(
    hrefs(japanese).filter((href) => href === "https://vizejs.dev/play/?tab=canon").length,
    1,
  );
  assert.deepEqual(hrefs(japanese), hrefs(english));
  const tableKeys = (source: string) =>
    [...source.matchAll(/^\| (`[^`]+`)\s*\|/gm)].map((match) => match[1]);
  assert.deepEqual(
    tableKeys(japanese),
    tableKeys(english),
    "all switch and compatibility rows remain",
  );

  const prose = archived.replace(/^```[^\n]*\n[\s\S]*?^```/gm, "");
  const inline = [
    ...new Set(
      [...prose.matchAll(/(?<!`)`([^`\n]+)`(?!`)/g)].map((match) =>
        match[1].replaceAll("\\|", "|"),
      ),
    ),
  ].sort();
  assert.equal(inline.length, 88);
  assert.equal(
    digest(JSON.stringify(inline)),
    "461c9b4225bc82001766a34433622208044089d885cec4e67c76b30eb6ebd541",
    "every original public syntax, field, value and compatibility name remains",
  );
  const currentInline = [
    ...new Set(
      [
        ...japanese.replace(/^```[^\n]*\n[\s\S]*?^```/gm, "").matchAll(/(?<!`)`([^`\n]+)`(?!`)/g),
      ].map((match) => match[1].replaceAll("\\|", "|")),
    ),
  ].sort();
  assert.deepEqual(
    currentInline,
    [
      ...new Set([
        ...inline.map((token) => (token === "vize.config.*" ? "vite.config.*" : token)),
        "vue.experimentals",
      ]),
    ].sort(),
    "all original public syntax and the canonical shared experimental namespace",
  );
  for (const id of [
    "experimentals",
    "スイッチ値",
    "surface-matrix",
    "flag-reference",
    "patterned-templates",
    "in-tag-comments",
    "self-component",
    "strict-slot-children",
    "server-script",
    "direct-api-fields",
  ]) {
    assert.equal(japanese.split(`<span id="${id}"></span>`).length - 1, 1, id);
  }
  const orPattern = japanese.split("\n").find((line) => line.startsWith("| または |"));
  assert.ok(orPattern);
  assert.equal(
    orPattern.split(/(?<!\\)\|/).length,
    5,
    "the pattern's pipe stays inside its code cell",
  );
  const englishOr = english.split("\n").find((line) => line.startsWith("| Or |"));
  assert.ok(englishOr);
  assert.equal(
    englishOr.split(/(?<!\\)\|/).length,
    5,
    "the English pattern retains all three cells",
  );
  for (const locale of ["", "ja/"]) {
    const details = readFileSync(
      resolve(content, `${locale}guide/experimentals-vue-rfcs.md`),
      "utf8",
    );
    const label = locale === "ja/" ? "または" : "Or";
    const referenceOr = details
      .split("\n")
      .find((line) => new RegExp(`^\\| ${label}\\s*\\|`).test(line));
    assert.ok(referenceOr);
    assert.equal(
      referenceOr.split(/(?<!\\)\|/).length,
      5,
      `${locale}RFC details retain the whole pattern and original explanation`,
    );
  }
  assert.match(japanese, /すべての機能は既定で無効/);
  assert.match(japanese, /変数を束縛する構文は未対応/);
  assert.match(japanese, /実行時の出力も生成しません/);
  assert.match(japanese, /コンパイルするだけのビルドでは、検査を行いません/);
  assert.match(japanese, /ブラウザーで単一ファイルを検査/);
  assert.match(japanese, /Corsa によるプロジェクト全体の型検査とは、検査範囲が異なります/);
  assert.match(japanese, /アプリケーションで、安定した構文として依存しないでください/);
  assert.doesNotMatch(
    japanese,
    /Missing keys|backend choice|compatibility resolution order|enforce され/,
  );
});
