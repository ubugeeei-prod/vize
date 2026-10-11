import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

void test("Japanese Vite guide retains published recipes and every current integration option", async () => {
  const content = resolve(import.meta.dirname, "../../docs/content");
  const path = "guide/vite-plugin.md";
  const english = readFileSync(resolve(content, path), "utf8");
  const japanese = readFileSync(resolve(content, "ja", path), "utf8");
  const retained = await collectReviewedTranslations([path], resolve(content, "ja"));
  assert.equal(retained.get(path), japanese, "regeneration retains the complete authored review");

  const fences = [...japanese.matchAll(/^```[^\n]*\n[\s\S]*?^```/gm)].map((match) => match[0]);
  // The ten whole published JA examples at signed main d55b8238f89b,
  // including the original localized TresJS comment, remain byte-exact.
  assert.equal(fences.length, 10);
  assert.equal(
    createHash("sha256").update(fences.join("\n")).digest("hex"),
    "848af26f7762fa9a4a3d35ebff40cd8fe6fb09cb4011bb6aca58374fd0450328",
  );

  const optionKeys = (source: string) =>
    [...source.matchAll(/^\| (`[^`]+`)\s*\|/gm)].map((match) => match[1]);
  assert.deepEqual(optionKeys(japanese), optionKeys(english), "no current option row is lost");
  const hrefs = (source: string) =>
    [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]);
  assert.deepEqual(
    hrefs(japanese).filter((href) => href !== "./configuration.md"),
    hrefs(english),
    "all English reading destinations retain their order and multiplicity",
  );
  assert.equal(hrefs(japanese).filter((href) => href === "./configuration.md").length, 1);
  for (const id of ["drop-in-scope", "compiler-options", "置換の定義", "nuxt-の互換性"]) {
    assert.equal(japanese.split(`<span id="${id}"></span>`).length - 1, 1, id);
  }
  assert.doesNotMatch(japanese, /蒸気モード|レーヨン|それでおしまい|構成されています/);
  assert.match(japanese, /Vize は `\.vue` ファイルをコンパイルせず/);
  assert.match(japanese, /API シムも公開せず/);
  assert.match(japanese, /機能フラグも挿入しません/);
});
