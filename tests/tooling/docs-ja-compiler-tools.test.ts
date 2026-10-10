import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const baseline = JSON.parse(
  readFileSync(resolve(import.meta.dirname, "fixtures/docs-ja-compiler-tools.json"), "utf8"),
) as { source: string; pages: Record<string, { fences: string[]; headings: string[] }> };
const read = (path: string) => readFileSync(resolve(root, path), "utf8");

void test("Japanese compiler guides retain all original complete examples and inbound headings", () => {
  assert.equal(baseline.source, "29cf6512674b0516caba21df9b530b299710f1ea");
  assert.deepEqual(Object.keys(baseline.pages), ["jsx", "jsx-babel-compat", "compiler-inspector"]);
  let examples = 0;
  for (const [name, page] of Object.entries(baseline.pages)) {
    const japanese = read(`docs/content/ja/guide/${name}.md`);
    const english = read(`docs/content/guide/${name}.md`);
    const fences = (source: string) =>
      [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```$/gm)].map((match) => match[0]);
    assert.deepEqual(fences(japanese), page.fences, `${name}: every original recipe stays whole`);
    assert.deepEqual(
      fences(japanese),
      fences(english),
      `${name}: English source examples stay exact`,
    );
    examples += page.fences.length;
    const rendered = native.transform(japanese, {});
    assert.deepEqual(rendered.errors, [], name);
    const ids = [...rendered.html.matchAll(/\bid="([^"]+)"/g)].map((match) => match[1]);
    for (const id of page.headings)
      assert.equal(ids.filter((value) => value === id).length, 1, `${name}: #${id}`);
    assert.match(japanese, /<!-- Reviewed translation; source: guide\//);
  }
  assert.equal(examples, 28);
});

void test("Babel compatibility prose follows the original compiler and corpus contracts", () => {
  const verdicts = read("crates/vize_atelier_jsx/tests/babel_compat/verdicts.rs");
  assert.match(verdicts, /\("slots\/dynamic_slot_name", Same\)/);
  assert.match(verdicts, /"options\/resolve_type_on",\s*Todo\(/);
  const module = read("crates/vize_atelier_jsx/src/compile.rs");
  assert.match(
    module,
    /VDOM output preserves\s*\/\/\/ authored declarations, exports and lexical scopes/,
  );
  for (const locale of ["", "ja/"]) {
    const source = read(`docs/content/${locale}guide/jsx-babel-compat.md`);
    const rows = source.split("\n").filter((line) => /^\| `(?:options|slots)\//.test(line));
    assert.equal(rows.length, 1, `${locale}: only the retained type-resolution row is deferred`);
    assert.match(rows[0], /options\/resolve_type_on/);
    assert.match(source, /`slots\/dynamic_slot_name`[^\n]*`equivalent`/);
    const rendered = native.transform(source, {});
    const inventory = [...rendered.html.matchAll(/<a\b[^>]*href="([^"]+)"[^>]*>/g)].filter(
      (match) => decodeURI(match[1]).endsWith("/BABEL_COMPAT_INVENTORY.md"),
    );
    assert.equal(inventory.length, 1);
    assert.match(inventory[0][1], /^https:\/\/github\.com\//);
    assert.match(inventory[0][0], /target="_blank"/);
    assert.match(inventory[0][0], /rel="noopener noreferrer"/);
  }
});
