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
  readFileSync(resolve(import.meta.dirname, "fixtures/docs-ja-blog-notes.json"), "utf8"),
) as {
  base: string;
  pages: {
    file: string;
    headings: string[];
    code: string[];
    destinations: string[];
    publication: string | null;
    listItems: number;
    cards: number;
    inlineCode: string[];
  }[];
};
const fences = (source: string) =>
  Array.from(source.matchAll(/```[^\n]*\n[\s\S]*?\n```/g), (m) => m[0]);

void test("Japanese blog review retains original publication, destinations and complete sections", () => {
  assert.equal(baseline.base, "a5661941e8760ca85960d0a2ef80f7c1b423d69e");
  assert.equal(baseline.pages.length, 4);
  assert.equal(
    baseline.pages.reduce((n, p) => n + p.headings.length, 0),
    21,
  );
  for (const page of baseline.pages) {
    const japanese = readFileSync(resolve(root, page.file), "utf8");
    const english = readFileSync(resolve(root, page.file.replace("/ja/", "/")), "utf8");
    const rendered = native.transform(japanese, {});
    assert.deepEqual(rendered.errors, [], page.file);
    assert.deepEqual(fences(japanese), page.code, `${page.file}: full original code and diagrams`);
    assert.deepEqual(fences(japanese), fences(english), `${page.file}: complete source examples`);
    assert.deepEqual(
      Array.from(japanese.matchAll(/\]\(([^)]+)\)|\b(?:href|src)="([^"]+)"/g), (m) => m[1] || m[2]),
      page.destinations,
      `${page.file}: original ordered link and image destinations`,
    );
    assert.deepEqual(
      Array.from(japanese.matchAll(/(?<!`)`([^`]+)`(?!`)/g), (m) => m[1]),
      page.inlineCode,
      `${page.file}: authoring paths and filename contract`,
    );
    assert.equal(
      japanese.match(/<div class="blog-post-meta">[\s\S]*?<\/div>/)?.[0] ?? null,
      page.publication,
      `${page.file}: original dated publication and author`,
    );
    const ids = new Set(Array.from(rendered.html.matchAll(/\bid="([^"]+)"/g), (m) => m[1]));
    for (const id of page.headings) assert.ok(ids.has(id), `${page.file}: original #${id}`);
    assert.equal(Array.from(rendered.html.matchAll(/<li\b/g)).length, page.listItems, page.file);
    assert.equal(Array.from(japanese.matchAll(/class="blog-post-list-item"/g)).length, page.cards);
    assert.equal(Array.from(rendered.html.matchAll(/<h[1-6]\b/g)).length, page.headings.length);
    const visible = rendered.html.replace(/<[^>]*>/g, "");
    assert.doesNotMatch(
      visible,
      /蒸気|水分補給|どの州|美術館|博物館|部族協定|ムセア|表面の差分|書き込みレーン|Rリリース|生息/,
      page.file,
    );
    if (page.publication) {
      const original = native.transform(english, {}).html;
      assert.equal(Array.from(original.matchAll(/<li\b/g)).length, page.listItems);
      if (page.file.includes("vapor-mode"))
        for (const term of ["Vapor", "DOM", "SSR", "SFC", "getter", "props", "hydration"])
          assert.ok(visible.includes(term), `${page.file}: ${term}`);
      else {
        for (const term of ["Musea", "AI", "Vite", "MCP", "Figma", "README"])
          assert.ok(visible.includes(term), `${page.file}: ${term}`);
        assert.match(visible, /Musea が目指すもの/);
        assert.match(visible, /日常的に行えるようにしたい/);
      }
    }
  }
});

void test("regeneration retains the complete reviewed posts and both authored indexes", async () => {
  const paths = baseline.pages.map((p) => p.file.replace("docs/content/ja/", ""));
  const retained = await collectReviewedTranslations(paths, resolve(root, "docs/content/ja"));
  assert.equal(retained.size, 4);
  for (const path of paths) {
    const original = readFileSync(resolve(root, "docs/content/ja", path), "utf8");
    assert.equal(retained.get(path), original, path);
    assert.ok(original.includes(`<!-- Reviewed translation; source: ${path} -->`), path);
  }
});
