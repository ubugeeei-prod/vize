import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const baseline = JSON.parse(
  readFileSync(resolve(import.meta.dirname, "fixtures/docs-ja-static-analysis.json"), "utf8"),
) as { base: string; file: string; headings: string[]; code: string[] };
const japanese = readFileSync(resolve(root, baseline.file), "utf8");
const english = readFileSync(resolve(root, "docs/content/guide/static-analysis.md"), "utf8");
const rendered = native.transform(japanese, {});

void test("Japanese analysis review retains complete recipes and inbound headings", () => {
  assert.equal(baseline.base, "2e7f5e1e2cbd740c0ed4bfd97286c4ee7c43ff10");
  assert.deepEqual(rendered.errors, []);
  const code = (source: string) =>
    Array.from(source.matchAll(/```[^\n]*\n([\s\S]*?)\n```/g), (match) => match[1]);
  assert.equal(baseline.code.length, 20);
  assert.deepEqual(code(japanese), baseline.code);
  assert.deepEqual(code(japanese), code(english));
  const ids = new Set(Array.from(rendered.html.matchAll(/\bid="([^"]+)"/g), (match) => match[1]));
  assert.equal(baseline.headings.length, 11);
  for (const id of baseline.headings) assert.ok(ids.has(id), `original #${id}`);
});

void test("Japanese analysis tables preserve rule identities and complete pipeline cells", () => {
  const rules = (source: string) =>
    Array.from(
      source.matchAll(/`((?:vue|script|css|a11y|html|ssr|vapor|musea|type)\/[^`]+)`/g),
      (match) => match[1],
    ).toSorted();
  assert.deepEqual(rules(japanese), rules(english));
  for (const layer of ["Armature", "Croquis", "Patina", "Canon", "Maestro"]) {
    const row = japanese.split("\n").find((line) => line.startsWith(`| ${layer} |`));
    assert.ok(row, `${layer}: source row exists`);
    assert.equal(row.split("|").length, 5, `${layer}: three complete cells`);
  }
  const headings = Array.from(
    rendered.html.matchAll(/<h[1-6][^>]*>(.*?)<\/h[1-6]>/g),
    (match) => match[1],
  ).join("\n");
  assert.doesNotMatch(headings, /糸くず|緑青|オクスリント/);
  assert.match(japanese, /<!-- Reviewed translation; source: guide\/static-analysis\.md -->/);
});
