import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const read = (file: string): string => readFileSync(resolve(root, file), "utf8");
const baseline = JSON.parse(read("tests/tooling/fixtures/docs-ja-philosophy.json")) as {
  source: string;
  fences: string[];
  headings: string[];
};
const japanese = read("docs/content/ja/philosophy.md");
const english = read("docs/content/philosophy.md");
const fences = (source: string): string[] =>
  [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```$/gm)].map((match) => match[0]);

void test("the full philosophy review keeps its original native bookmarks and complete pipeline", () => {
  assert.equal(baseline.source, "0cfbdcb98c8df2aff2e526076deb44fb1d9a1ee5");
  assert.equal(baseline.headings.length, 14);
  assert.equal(baseline.fences.length, 1);
  assert.deepEqual(fences(japanese), baseline.fences);
  assert.deepEqual(fences(japanese), fences(english));
  const rendered = native.transform(japanese, {});
  assert.deepEqual(rendered.errors, []);
  const ids = [...rendered.html.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]);
  for (const id of baseline.headings)
    assert.equal(ids.filter((value) => value === id).length, 1, `original #${id}`);
  assert.match(japanese, /<!-- Reviewed translation; source: philosophy\.md -->/);
});

void test("art names keep all twelve crate-role associations and Davinci naming stays explicit", () => {
  const rows = (source: string): string[][] =>
    source
      .split("\n")
      .filter((line) => line.startsWith("| **"))
      .map((line) =>
        line
          .split("|")
          .slice(1, -1)
          .map((cell) => cell.trim()),
      );
  const sourceRows = rows(english);
  const reviewedRows = rows(japanese);
  assert.equal(sourceRows.length, 12);
  assert.equal(reviewedRows.length, 12);
  assert.deepEqual(
    reviewedRows.map((row) => row[0]),
    sourceRows.map((row) => row[0]),
  );
  for (const row of reviewedRows) assert.equal(row.length, 3, row[0]);
  const roles = [
    "共通ユーティリティ",
    "AST",
    "パーサー",
    "意味解析",
    "コンパイラ",
    "バインディング",
    "型チェッカー",
    "リンター",
    "フォーマッタ",
    "LSP",
    "コンポーネントギャラリー",
    "TUI",
  ];
  for (let i = 0; i < roles.length; i++)
    assert.ok(reviewedRows[i][2].includes(roles[i]), `${reviewedRows[i][0]}: ${roles[i]}`);
  assert.match(english, /Davinci's internal crates use level names/);
  assert.match(japanese, /Davinci の内部クレートは、別の規則に従ってレベル名/);
});

void test("compatibility limits and existing external research destinations survive native rendering", () => {
  const hrefs = (source: string): string[] =>
    [...native.transform(source, {}).html.matchAll(/\bhref="([^"]+)"/g)].map((match) => match[1]);
  const external = (source: string): string[] =>
    hrefs(source).filter((href) => href.startsWith("https://"));
  assert.deepEqual(external(japanese), external(english));
  assert.ok(hrefs(japanese).includes("./guide/vite-plugin.md#drop-in-scope"));
  const target = native.transform(read("docs/content/ja/guide/vite-plugin.md"), {}).html;
  assert.match(target, /id="drop-in-scope"/);
  assert.match(japanese, /Vue 2 と 2\.7 \(`vue\.version`\).*開発段階/);
  assert.match(japanese, /webpack はこの置き換えの対象に含みません/);
  assert.match(japanese, /プラグインオプションの互換性も未完成/);
  assert.match(japanese, /498ms/);
  assert.match(japanese, /本番環境での利用に向けた準備は完了していません/);
});
