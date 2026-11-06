import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { test } from "node:test";
import { defaultNavigationRenderRoutes } from "../../docs/scripts/navigation-render-routes.ts";
import { expectedRulePages, verifyRuleSources } from "../../docs/scripts/rule-render-assertions.ts";
import { operationalDiagnosticPages } from "../../docs/scripts/rules/operational-diagnostics.ts";

const root = resolve(import.meta.dirname, "../..");
const read = (file: string) => readFileSync(resolve(root, file), "utf8");
const pageAt = (locale: string, slug: string) => `docs/content/${locale}rules/reference/${slug}.md`;
const fences = (page: string) =>
  [...page.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)\n```$/gm)].map(([, language, source]) => [
    language,
    source,
  ]);

void test("operational diagnostic references retain their own English/Japanese authority and examples", () => {
  assert.deepEqual(operationalDiagnosticPages, [
    ["parser/template", "parser-template"],
    ["parser/sfc", "parser-sfc"],
    ["parser/jsx", "parser-jsx"],
    ["type/project-configuration", "type-project-configuration"],
  ]);
  for (const [id, slug] of operationalDiagnosticPages) {
    const english = read(pageAt("", slug));
    const japanese = read(pageAt("ja/", slug));
    assert.equal(english.match(/^# `([^`]+)`$/m)?.[1], id);
    assert.equal(japanese.match(/^# `([^`]+)`$/m)?.[1], id);
    assert.ok(
      japanese.includes(`<!-- Reviewed translation; source: rules/reference/${slug}.md -->`),
    );
    assert.deepEqual(fences(japanese), fences(english), `${id}: complete example source`);
    assert.equal(fences(english).length, 2, `${id}: failing and repaired input`);
    assert.notDeepEqual(
      fences(english)[0],
      fences(english)[1],
      `${id}: Bad remains an unrepaired input`,
    );
    for (const locale of ["", "ja/"]) {
      const path = pageAt(locale, slug);
      const source = read(path);
      for (const [, href] of source.matchAll(/\]\(([^\s)]+)\)/g)) {
        if (/^https?:/.test(href)) continue;
        const target = resolve(dirname(resolve(root, path)), href.split("#")[0]);
        assert.ok(existsSync(target), `${path}: ${href}`);
        assert.ok(
          target.startsWith(resolve(root, `docs/content/${locale}`) + "/"),
          `${path}: related reading stays in the same locale`,
        );
      }
      const guide = read(`docs/content/${locale}guide/lint-execution.md`);
      assert.ok(
        guide.includes(`](../rules/reference/${slug}.md)`),
        `${path}: discoverable guide link`,
      );
      assert.equal(
        defaultNavigationRenderRoutes.filter(
          (route) => route === `/${locale}rules/reference/${slug}`,
        ).length,
        1,
        `${path}: exact built route is reviewed`,
      );
    }
  }
});

void test("the operational pages preserve all 280 configurable rules and 66 project rules in every locale", () => {
  for (const locale of ["en", "ja", "zh-CN", "pt-BR", "fr"] as const) {
    const pages = expectedRulePages(locale);
    assert.equal(pages.length, 346);
    assert.deepEqual(
      pages.filter((page) => operationalDiagnosticPages.some(([id]) => id === page.id)),
      [],
      `${locale}: operational failures are not opt-in rules`,
    );
    verifyRuleSources(`/${locale === "en" ? "" : `${locale}/`}rules/all`);
  }
});

void test("native Markdown renders all operational identities, examples and related links", () => {
  const require = createRequire(new URL("../../docs/package.json", import.meta.url));
  const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
  const { Window } = createRequire(new URL("../package.json", import.meta.url))("happy-dom");
  for (const [id, slug] of operationalDiagnosticPages) {
    for (const locale of ["", "ja/"]) {
      const file = pageAt(locale, slug);
      const source = read(file);
      const rendered = native.transform(source, {
        convertMdLinks: true,
        baseUrl: "/",
        sourcePath: resolve(root, file),
      });
      assert.deepEqual(rendered.errors, [], file);
      const document = new Window().document;
      document.body.innerHTML = rendered.html;
      assert.equal(document.querySelector("h1")?.textContent, id, file);
      assert.deepEqual(
        [...document.querySelectorAll("pre code")].map((code) =>
          code.textContent.replace(/\n$/, ""),
        ),
        fences(source).map(([, code]) => code),
        `${file}: native renderer preserves the complete examples`,
      );
      const links = [...document.querySelectorAll("a[href]")].map((link) =>
        link.getAttribute("href"),
      );
      assert.ok(links.includes("../../../guide/lint-execution/index.html"), file);
      assert.ok(
        links.every((href) => !href.endsWith(".md") || href.startsWith("https://")),
        file,
      );
    }
  }
});
