import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { renderReferenceDocs } from "../../npm/ui/scripts/generate-reference-docs.ts";

const root = resolve(import.meta.dirname, "../..");
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const locales = ["", "ja", "fr", "pt-BR", "zh-CN"];
type OriginalDocument = { file: string; headings: string[]; code: string[] };
const original = JSON.parse(
  readFileSync(resolve(import.meta.dirname, "fixtures/docs-config-journey.json"), "utf8"),
) as { base: string; documents: OriginalDocument[] };
const fences = (source: string): string[] =>
  Array.from(source.matchAll(/```[^\n]*\n([\s\S]*?)\n```/g), (match) => match[1] ?? "");
let generated: Map<string, string> | undefined;

function sourceFor(file: string): string | undefined {
  const absolute = resolve(root, file);
  if (existsSync(absolute)) return readFileSync(absolute, "utf8");
  generated ??= renderReferenceDocs();
  return generated.get(file.replace(/^docs\/content\//u, ""));
}

function render(file: string): { html: string; source: string } {
  const source = sourceFor(file);
  assert.ok(source !== undefined, `${file}: source or actual generated reference exists`);
  const result = native.transform(source, {});
  assert.deepEqual(result.errors, [], file);
  return { source, html: result.html as string };
}

void test("configuration reorganization retains every original native heading target", () => {
  assert.equal(original.base, "fe3934d8a24c7d2fe0d26b2ca095d61c90f49b1f");
  assert.equal(original.documents.length, 15);
  let total = 0;
  for (const document of original.documents) {
    const { html } = render(document.file);
    const ids = new Set(Array.from(html.matchAll(/\bid="([^"]+)"/g), (match) => match[1]));
    for (const id of document.headings) {
      assert.ok(ids.has(id), `${document.file}: original #${id}`);
      total += 1;
    }
  }
  assert.equal(total, 172);
});

void test("relocated configuration references retain all complete original code examples", () => {
  let count = 0;
  for (const document of original.documents.filter((item) => item.code.length > 0)) {
    const directory = dirname(document.file);
    const retained = ["configuration-reference", "compiler-configuration-reference"].flatMap(
      (name) => fences(render(`${directory}/${name}.md`).source),
    );
    assert.equal(document.code.length, 14, document.file);
    for (const code of document.code) {
      assert.ok(retained.includes(code), `${document.file}: complete original example`);
      count += 1;
    }
  }
  assert.equal(count, 42);
});

void test("every locale publishes the same complete Vite setup and native config recipes", () => {
  const recipes = fences(render("docs/content/getting-started.md").source);
  const configuration = fences(render("docs/content/guide/configuration.md").source);
  for (const locale of locales) {
    const prefix = `docs/content/${locale ? `${locale}/` : ""}`;
    const startup = render(`${prefix}getting-started.md`);
    const config = render(`${prefix}guide/configuration.md`);
    assert.deepEqual(fences(startup.source), recipes, locale);
    assert.deepEqual(fences(config.source), configuration, locale);
    assert.match(config.source, /`tsconfig\.json`/);
    assert.match(config.source, /`vize\.entries`/);
    assert.match(config.source, /`--config`/);
    assert.doesNotMatch(config.html, /<strong>\s*<code>vize\.config/);
    assert.ok(startup.source.includes("> [!NOTE]"), `${locale}: publication scope stays visible`);
    assert.ok(config.source.includes("> [!NOTE]"), `${locale}: publication scope stays visible`);
  }
});

void test("localized configuration reading paths resolve through native rendered targets", () => {
  const rendered = new Map<string, ReturnType<typeof render>>();
  for (const locale of locales) {
    const prefix = `docs/content/${locale ? `${locale}/` : ""}`;
    for (const name of [
      "getting-started.md",
      "guide/configuration.md",
      "guide/configuration-reference.md",
      "guide/compiler-configuration-reference.md",
    ]) {
      const file = `${prefix}${name}`;
      const document = render(file);
      for (const [, href, fragment] of document.html.matchAll(/\bhref="([^"#]*\.md)(#[^"]*)?"/g)) {
        if (!href || /^(?:https?:|\/\/)/u.test(href)) continue;
        const target = resolve(root, dirname(file), href);
        const relative = target.slice(`${root}/`.length);
        const fallback = relative.replace(
          /^docs\/content\/(?:ja|fr|pt-BR|zh-CN)\//u,
          "docs/content/",
        );
        const chosen = sourceFor(relative) === undefined ? fallback : relative;
        assert.ok(sourceFor(chosen) !== undefined, `${file}: ${href}`);
        if (!rendered.has(chosen)) rendered.set(chosen, render(chosen));
        if (fragment) {
          const id = decodeURIComponent(fragment.slice(1));
          const html = rendered.get(chosen)?.html;
          assert.ok(html?.includes(`id="${id}"`), `${file}: ${href}${fragment}`);
        }
      }
    }
  }
  assert.ok(rendered.size >= 30, "the complete localized reading paths were visited");
});

void test("library and Musea shared examples keep the same native scope in every locale", () => {
  const shared = (source: string, section: string) =>
    fences(source).find((code) => code.includes(`    ${section}: {`));
  const musea = shared(render("docs/content/guide/musea.md").source, "musea");
  const lib = shared(render("docs/content/guide/lib-pull.md").source, "lib");
  assert.ok(musea && lib, "complete authoritative English examples exist");
  for (const locale of locales) {
    const prefix = `docs/content/${locale ? `${locale}/` : ""}guide/`;
    assert.equal(shared(render(`${prefix}musea.md`).source, "musea"), musea, locale);
    const library = render(`${prefix}lib-pull.md`);
    assert.equal(shared(library.source, "lib"), lib, locale);
    assert.match(library.source, /`vite\.config\.mjs`/);
    assert.match(library.source, /`vize\.lib`/);
    assert.match(library.source, /`vize\.config\.json`/);
  }
});
