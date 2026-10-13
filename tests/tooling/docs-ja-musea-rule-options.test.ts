import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

void test("Japanese Musea options keep the complete authored input and original recipes", async () => {
  const root = resolve(import.meta.dirname, "../..");
  const path = "rules/options-musea.md";
  const english = readFileSync(resolve(root, "docs/content", path), "utf8");
  const japanese = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
  const retained = await collectReviewedTranslations([path], resolve(root, "docs/content/ja"));
  assert.equal(retained.get(path), japanese, "regeneration retains every authored paragraph");
  const pattern = /^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm;
  const contracts = (source: string) => [
    [...source.matchAll(pattern)].map((match) => match[0]),
    [...source.replace(pattern, "").matchAll(/(?<!`)`([^`\n]+)`(?!`)/g)].map((match) => match[1]),
    [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]),
  ];
  assert.deepEqual(contracts(japanese), contracts(english));
  assert.equal(contracts(english)[0].length, 3);
  assert.equal(
    createHash("sha256")
      .update(JSON.stringify(contracts(english)))
      .digest("hex"),
    "aebae9c73d00e8acec07f378bd600636c43a19ccac1482168ab47fe4161634da",
    "three original recipes, original API tokens and the finite explicit off addition",
  );
  assert.equal(japanese.split('<span id="musea-rule-options"></span>').length, 2);
  assert.match(japanese, /^## `musea\/prefer-design-tokens`$/m);
  assert.doesNotMatch(japanese, /<span id="musea-prefer-design-tokens"><\/span>/);
});
