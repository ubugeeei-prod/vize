import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const english = readFileSync(
  new URL("../../docs/content/architecture/language-engineering-practices.md", import.meta.url),
  "utf8",
);
const japanese = readFileSync(
  new URL("../../docs/content/ja/architecture/language-engineering-practices.md", import.meta.url),
  "utf8",
);

test("the Japanese practices guide preserves every source command and external reference", () => {
  const code = (source: string) => [...source.matchAll(/`([^`\n]+)`/g)].map((m) => m[1]).sort();
  const references = (source: string) =>
    [...source.matchAll(/\]\((https:\/\/[^)]+)\)/g)].map((m) => m[1]).sort();
  assert.deepEqual(code(japanese), code(english));
  assert.deepEqual(references(japanese), references(english));
  const tables = (source: string) =>
    (source.match(/(?:^\|[^\n]+\n)+/gm) ?? []).map((table) => table.trim().split("\n").length - 2);
  assert.deepEqual(tables(japanese), [6, 3, 10]);
  assert.deepEqual(tables(japanese), tables(english));
});

test("natural Japanese sections retain published fragment links and complete review scope", () => {
  assert.match(japanese, /Reviewed translation;[^\n]+scope: complete document/);
  for (const [anchor, heading] of [
    ["vize-クラス変更", "変更の種類"],
    ["保証レーン", "追加の検証"],
    ["ベースラインポリシー", "ベースラインの扱い"],
    ["エスカレーションのトリガー", "検証を広げる条件"],
    ["運用上のガードレール", "継続して確認する仕組み"],
    ["ソース信号", "参考資料"],
  ]) {
    assert.ok(japanese.includes(`<a id="${anchor}"></a>\n\n## ${heading}`), anchor);
  }
  for (const [, fragment] of japanese.matchAll(/\]\(#([^)]+)\)/g)) {
    assert.ok(
      japanese.includes(`id="${fragment}"`),
      `missing introduction destination: ${fragment}`,
    );
  }
});
