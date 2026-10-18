import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import vm from "node:vm";
import { renderReferenceDocs } from "../../npm/ui/scripts/generate-reference-docs.ts";

const root = resolve(import.meta.dirname, "../..");
const context = vm.createContext({});
for (const name of ["en", "ja"]) {
  vm.runInContext(
    readFileSync(resolve(root, `docs/theme/i18n/locales/${name}.js`), "utf8"),
    context,
  );
}
const locales = context.__vizeDocsLocales as Record<string, { labels: Record<string, string> }>;

void test("every existing Japanese blog navigation label follows its authored page title", () => {
  const routes = Object.keys(locales.en.labels).filter((route) =>
    /^\/blog\/(notes|releases)\//.test(route),
  );
  assert.equal(routes.length, 14);
  for (const route of routes) {
    const page = readFileSync(resolve(root, `docs/content/ja${route}.md`), "utf8");
    const heading = /^# (.+)$/m.exec(page)?.[1].replaceAll("`", "");
    assert(heading, `${route}: complete authored Japanese page title`);
    assert.equal(
      locales.ja.labels[route],
      heading,
      `${route}: no English fallback for the Japanese reader`,
    );
  }
});

void test("generated Japanese hubs explain their API language and link to the Japanese setup guide", () => {
  const pages = renderReferenceDocs();
  const composables = pages.get("ja/guide/composables/index.md");
  const ui = pages.get("ja/guide/ui/index.md");
  assert(composables && ui, "the real generator produces both complete Japanese hub pages");
  assert.match(composables, /個別の API 説明は英語です。/);
  assert.match(composables, /SSR とハイドレーション時の動作、リソースの解放、対応する実行環境/);
  assert.match(ui, /\[Vite\+ 連携\]\(\/ja\/guide\/vite-plus\/\)/);
  assert.ok(readFileSync(resolve(root, "docs/content/ja/guide/vite-plus.md"), "utf8"));
  assert.match(composables, /実機での接続や許可後の動作まで検証済みとは限りません/);
  assert.match(composables, /例以外については、ライブプレビューでの検証をまだ行っていません/);
  for (const locale of ["", "zh-CN/", "pt-BR/", "fr/"]) {
    const other = pages.get(`${locale}guide/ui/index.md`);
    assert(other, `${locale || "en"}: generated hub remains present`);
    assert.ok(!other.includes("[Vite+ 連携]"), "the Japanese wording stays in its own locale");
  }
});
