import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  docsRoot,
  moreGuideRoutes,
  moreGuidesCoverage,
  renderedMoreGuideRoutes,
} from "../../docs/scripts/more-guides-coverage.ts";
import { collectReviewedTranslations } from "../../docs/scripts/i18n/reviewed.ts";

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
      "## 変更の種類",
      "## 参考資料",
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

test("Musea capture regeneration retains its recipe and localized VRT destination", async () => {
  const retained = await collectReviewedTranslations(
    ["guide/musea-capture.md", "guide/musea.md"],
    path.join(docsRoot, "content/ja"),
  );
  const fences = (source: string) =>
    [...source.matchAll(/^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm)].map(([whole]) => whole);
  const english = content("en", "/guide/musea-capture");
  const japanese = content("ja", "/guide/musea-capture");
  assert.equal(fences(english).length, 1);
  assert.equal(
    createHash("sha256").update(fences(english).join("\n")).digest("hex"),
    "64c7f3512e1da84954d46fb9da95e2de6f98c4a1ebeac30cf25d17cb3f7a2452",
    "complete original capture recipe",
  );
  assert.deepEqual(fences(japanese), fences(english));
  assert.equal(retained.get("guide/musea-capture.md"), japanese);
  assert.equal(retained.get("guide/musea.md"), content("ja", "/guide/musea"));
  for (const locale of ["en", "ja"]) {
    assert.match(
      content(locale, "/guide/musea-capture"),
      /\]\(\.\/musea\.md#visual-regression-testing\)/,
      `${locale}: practical link stays in the localized guide`,
    );
    assert.match(
      content(locale, "/guide/musea"),
      locale === "en"
        ? /^## Visual Regression Testing$/m
        : /<span id="visual-regression-testing"><\/span>\n\n## 視覚的な回帰テスト/,
      `${locale}: destination keeps its genuine heading and stable bookmark`,
    );
  }
});

test("formatter guide regeneration keeps all original recipes, tokens and localized next steps", async () => {
  const originals = [
    [
      "import-sorting",
      "2094b899570049b87991c18f3ee4a0b754cba5b802fb0bd817ae2d812fbba3fb",
      "6570d40104a970bfb804a862b7d1d2d8faff83de9029bf4d69749b24567b90b9",
    ],
    [
      "indent-width",
      "d29b37d752e1c1bb07ece7fdf300a4a331b9f6104966171764740a489305d756",
      "0b5fc51768ea324e69ff95903d939c2bd0b45b4825e6497dd52c56220cd18644",
    ],
    [
      "jsx-quotes",
      "5c4ae10a714327c1705d6f449e2c67ab96d15111ea69a27e70932cd8785ad434",
      "b42d45e8eafe3509203ce10e496ea5c8e52ca36d6c8e745d36f8de720c43729a",
    ],
    [
      "line-endings",
      "701828afcfdb9f309c4e6aadb434b472ac345297184664dae0cc03dd687562f1",
      "b4a3f307e274256a8e2d175bf95e595e8440e7b4c04ab49e1f1da2556025fa3b",
    ],
    [
      "property-quotes",
      "01dc9a31371198ea51aa08851c01fa1b73ac105580b7ad34e485fa31132af36e",
      "fb203dbd8b92f299f05bb964653db6ec96a055b01e6907d2ac269965c453a749",
    ],
  ];
  const sourcePaths = originals.map(([slug]) => `guide/formatter-${slug}.md`);
  const retained = await collectReviewedTranslations(
    sourcePaths,
    path.join(docsRoot, "content/ja"),
  );
  assert.equal(
    retained.size,
    originals.length,
    "all five complete authored guides survive regeneration",
  );
  const digest = (value: string) => createHash("sha256").update(value).digest("hex");
  const fencePattern = /^```[^\n]*\n[\s\S]*?^```[^\n]*$/gm;
  const fences = (source: string) => [...source.matchAll(fencePattern)].map(([whole]) => whole);
  const tokens = (source: string) =>
    [
      ...new Set(
        [...source.replace(fencePattern, "").matchAll(/(?<!`)`([^`\n]+)`(?!`)/g)].map(
          ([, value]) => value,
        ),
      ),
    ].sort();
  const links = (source: string) =>
    [...source.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)].map(([, href]) => href);
  for (const [slug, fenceDigest, tokenDigest] of originals) {
    const route = `/guide/formatter-${slug}`;
    const english = content("en", route);
    const japanese = content("ja", route);
    assert.equal(
      retained.get(`guide/formatter-${slug}.md`),
      japanese,
      `${slug}: entire reviewed input`,
    );
    assert.equal(
      digest(fences(english).join("\n")),
      fenceDigest,
      `${slug}: frozen original complete recipes`,
    );
    assert.deepEqual(fences(japanese), fences(english), `${slug}: byte-exact translated recipes`);
    assert.equal(
      digest(JSON.stringify(tokens(english))),
      tokenDigest,
      `${slug}: frozen original inline contracts`,
    );
    assert.deepEqual(
      tokens(japanese),
      tokens(english),
      `${slug}: all original options and raw examples`,
    );
    assert.deepEqual(links(japanese), links(english), `${slug}: complete ordered destinations`);
    for (const href of links(japanese)) {
      assert.ok(!href.startsWith("/guide/"), `${slug}: retain localized relative navigation`);
      readFileSync(path.resolve(docsRoot, "content/ja/guide", href));
    }
  }
  assert.match(
    content("ja", "/guide/formatter-jsx-quotes"),
    /Node 向けネイティブ API のフォーマッタオプションでは `jsxSingleQuote` をまだ指定できません。/,
  );
});
