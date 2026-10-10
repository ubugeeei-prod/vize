import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  docsRoot,
  moreGuideRoutes,
  moreGuidesCoverage,
  renderedMoreGuideRoutes,
} from "../../docs/scripts/more-guides-coverage.ts";

function content(locale: string, route: string) {
  return readFileSync(
    path.join(docsRoot, "content", locale === "en" ? "" : locale, `${route.slice(1)}.md`),
    "utf8",
  );
}

test("every current English and Japanese More route has an explicit audit entry", () => {
  const pages = moreGuidesCoverage.pages;
  assert.equal(moreGuidesCoverage.version, 1);
  assert.equal(moreGuidesCoverage.issue, 8365);
  assert.equal(new Set(pages.map(({ route }) => route)).size, pages.length);
  for (const locale of ["en", "ja"] as const) {
    const audited = new Set(
      pages.filter(({ locales }) => locales.includes(locale)).map(({ route }) => route),
    );
    assert.deepEqual(
      moreGuideRoutes(locale).filter((route) => !audited.has(route)),
      [],
      `${locale}: new More pages need an audit`,
    );
  }
  for (const page of pages) {
    assert.ok(page.note.length > 20, `${page.route}: explain the actual review boundary`);
    for (const locale of page.locales) assert.match(content(locale, page.route), /^# .+/m);
  }
});

test("audited Musea guides add native captures without removing any revised route", () => {
  assert.equal(new Set(renderedMoreGuideRoutes).size, renderedMoreGuideRoutes.length);
  for (const page of moreGuidesCoverage.pages.filter(({ entry }) => entry === "revised")) {
    for (const locale of page.locales) {
      assert.ok(
        renderedMoreGuideRoutes.includes(`${locale === "en" ? "" : `/${locale}`}${page.route}`),
      );
    }
  }
  for (const route of ["/guide/musea-hosting", "/guide/musea-snapshots"]) {
    const page = moreGuidesCoverage.pages.find((page) => page.route === route);
    assert.ok(page, `${route}: an explicit complete audit is required`);
    assert.equal(page.entry, "audited");
    assert.equal(page.capture, true, `${route}: audit classification cannot omit native capture`);
    assert.deepEqual(page.locales, ["en", "ja"]);
    for (const locale of page.locales) {
      assert.ok(renderedMoreGuideRoutes.includes(`${locale === "en" ? "" : "/ja"}${route}`));
    }
  }
});

test("revised pages offer a descriptive next-step link before the first reference section", () => {
  for (const page of moreGuidesCoverage.pages.filter(({ entry }) => entry === "revised")) {
    for (const locale of page.locales) {
      const source = content(locale, page.route);
      const intro = source.split(/^## /m)[0];
      const links = [...intro.matchAll(/\[([^\]]+)\]\(([^)]+)\)/g)];
      assert.ok(
        links.some(
          ([, label, href]) =>
            label.length >= 4 &&
            !/^(?:here|click here|こちら)$/i.test(label) &&
            !href.startsWith("https:"),
        ),
        `${locale}${page.route}: missing a descriptive practical link`,
      );
    }
  }
});

test("practical instructions precede long implementation reference tables", () => {
  for (const [locale, steps, research, api, options] of [
    [
      "en",
      "## Vize Change Classes",
      "## Source Signals",
      "### Compile SFC",
      "### Compiler option compatibility",
    ],
    [
      "ja",
      "## Vize クラス変更",
      "## ソース信号",
      "### SFC をコンパイルする",
      "### コンパイラオプションの互換性",
    ],
  ]) {
    const practices = content(locale, "/architecture/language-engineering-practices");
    assert.ok(
      practices.indexOf(steps) >= 0 && practices.indexOf(steps) < practices.indexOf(research),
    );
    const wasm = content(locale, "/guide/wasm");
    assert.ok(wasm.indexOf(api) >= 0 && wasm.indexOf(api) < wasm.indexOf(options));
  }
  assert.match(content("ja", "/guide/cli"), /<span id="check"><\/span>/);
  assert.match(content("ja", "/architecture/crates"), /^title: クレート$/m);
});
