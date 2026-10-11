import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

void test("Japanese RFC details retain complete syntax, destinations and authored regeneration", async () => {
  const content = resolve(import.meta.dirname, "../../docs/content");
  const path = "guide/experimentals-vue-rfcs.md";
  const english = readFileSync(resolve(content, path), "utf8");
  const japanese = readFileSync(resolve(content, "ja", path), "utf8");
  const retained = await collectReviewedTranslations([path], resolve(content, "ja"));
  assert.equal(retained.get(path), japanese, "regeneration retains the complete authored review");

  const digest = (source: string) => createHash("sha256").update(source).digest("hex");
  const fences = (source: string) =>
    [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```/gm)].map((match) => match[0]);
  // The sixteen complete examples at 24cd18c34626, including invalid
  // placements and slot cardinalities, remain exact and match current EN.
  assert.equal(fences(japanese).length, 16);
  assert.equal(
    digest(fences(japanese).join("\n")),
    "087cfbc98dc8ff364ff59828f05e862c667c22e8dd48ca06b3b56a5e9fd20da3",
  );
  assert.deepEqual(fences(japanese), fences(english));
  const hrefs = (source: string) =>
    [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map((match) => match[1]);
  assert.equal(
    digest(JSON.stringify(hrefs(japanese))),
    "0a144dcf4a0fd86f0453d36a784b9de8e7aef4875e34e96f0ca0852975c25707",
    "all seven original destinations retain their order and multiplicity",
  );
  assert.deepEqual(hrefs(japanese), hrefs(english));
  const prose = japanese.replace(/^```[^\n]*\n[\s\S]*?^```/gm, "");
  const inline = [
    ...new Set(
      [...prose.matchAll(/(?<!`)`([^`\n]+)`(?!`)/g)].map((match) =>
        match[1].replaceAll("\\|", "|"),
      ),
    ),
  ].sort();
  assert.equal(inline.length, 109);
  assert.equal(
    digest(JSON.stringify(inline)),
    "232f1964e47bcc247f3e5790619336168efb7df7855f36bcab8d9404d6c73d5e",
    "every original field, type, value, invalid form and compatibility name remains",
  );
  for (const id of [
    "vue-rfc-experimental-details",
    "opt-in-contract",
    "current-scope",
    "patterned-templates",
    "patterned-diagnostics",
    "patterned-type-boundary",
    "in-tag-comments",
    "self-component",
    "strict-slot-children",
    "verification-checklist",
  ]) {
    assert.equal(japanese.split(`<span id="${id}"></span>`).length - 1, 1, id);
  }
  const keys = (source: string) =>
    [
      ...source.matchAll(/^\|\s*\[#\d+\]\([^)]+\)[^\n]*?\|\s*(`[^`]+`)\s*\|\s*(`[^`]+`)\s*\|/gm),
    ].map((match) => [match[1], match[2]]);
  assert.equal(keys(japanese).length, 4);
  assert.deepEqual(keys(japanese), keys(english), "all opt-in and direct compiler fields remain");
  const orPattern = japanese.split("\n").find((line) => /^\| または\s*\|/.test(line));
  assert.ok(orPattern);
  assert.equal(orPattern.split(/(?<!\\)\|/).length, 5);
  assert.match(japanese, /フラグを明示的に有効にしない限り、RFC の機能は有効になりません/);
  assert.match(japanese, /明示的に無効にした場合も同じです/);
  assert.match(japanese, /ガード付きの分岐は、網羅性の証明には使いません/);
  assert.match(japanese, /or パターンの選択肢内での変数束縛も未対応/);
  assert.match(japanese, /網羅漏れは元の判定対象の位置に表示され、`vize check` が失敗/);
  assert.match(japanese, /到達不能な分岐の警告だけであれば成功/);
  assert.match(japanese, /言語サーバーを再起動してください/);
  assert.match(japanese, /エディターのナビゲーションや補完も含まれますが、そこまでの認定は未完了/);
  assert.match(japanese, /Content Mapper[\s\S]*名前変更や補完の全組み合わせの検証は未完了/);
  assert.match(japanese, /実行時のコードは出力しません/);
  assert.match(japanese, /`comments: false` でも[\s\S]*開始タグ内コメントは保持/);
  assert.match(japanese, /`<self>` は特別扱いしません/);
  assert.match(japanese, /render 関数と JSX は、このフラグの対象外/);
  assert.match(japanese, /テンプレートのコード生成や実行時の描画は変わりません/);
  assert.match(japanese, /必須のスロット名の検査は別の仕組み/);
  assert.doesNotMatch(
    japanese,
    /branch-local|unguarded top-level fallback|compile-time-only|advanced component library|experimental behavior を document/,
  );
});
