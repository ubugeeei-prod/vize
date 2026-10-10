import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const baseline = JSON.parse(
  readFileSync(resolve(import.meta.dirname, "fixtures/docs-ja-references.json"), "utf8"),
) as {
  base: string;
  pages: { file: string; headings: string[]; code: string[]; links: string[] }[];
};
const code = (source: string) =>
  Array.from(source.matchAll(/```[^\n]*\n([\s\S]*?)\n```/g), (match) => match[1]);
const links = (source: string) =>
  Array.from(source.matchAll(/\]\(([^)]+)\)/g), (match) => match[1]);
const technicalCells = (source: string) =>
  source
    .split("\n")
    .filter((line) => line.startsWith("|") && !/^\|[ -]+\|/.test(line))
    .map((line) =>
      line
        .split("|")
        .slice(1, -1)
        .map((cell) => Array.from(cell.matchAll(/`([^`]+)`/g), (m) => m[1]).toSorted()),
    )
    .filter((cells) => cells.some((cell) => cell.length > 0));

void test("Japanese reference review preserves every complete recipe and original bookmark", () => {
  assert.equal(baseline.base, "0af8ed716b77611aff8035385328bc0da181de77");
  assert.equal(baseline.pages.length, 2);
  for (const page of baseline.pages) {
    const japanese = readFileSync(resolve(root, page.file), "utf8");
    const english = readFileSync(resolve(root, page.file.replace("/ja/", "/")), "utf8");
    const rendered = native.transform(japanese, {});
    assert.deepEqual(rendered.errors, [], page.file);
    assert.equal(page.code.length, 6, page.file);
    assert.deepEqual(code(japanese), page.code, `${page.file}: original source bytes`);
    assert.deepEqual(code(japanese), code(english), `${page.file}: complete English recipes`);
    assert.deepEqual(links(japanese), page.links, `${page.file}: original destinations`);
    const ids = new Set(Array.from(rendered.html.matchAll(/\bid="([^"]+)"/g), (m) => m[1]));
    for (const id of page.headings) assert.ok(ids.has(id), `${page.file}: original #${id}`);
    assert.deepEqual(
      technicalCells(japanese),
      technicalCells(english),
      `${page.file}: option values, flags, API identities and slot cardinalities remain associated`,
    );
  }
});

void test("translation regeneration retains both complete reviewed references", async () => {
  const paths = baseline.pages.map((page) => page.file.replace("docs/content/ja/", ""));
  const retained = await collectReviewedTranslations(paths, resolve(root, "docs/content/ja"));
  assert.equal(retained.size, paths.length);
  for (const path of paths) {
    const source = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
    assert.equal(retained.get(path), source, path);
    assert.ok(source.includes(`<!-- Reviewed translation; source: ${path} -->`), path);
  }
});
