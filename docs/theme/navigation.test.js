import assert from "node:assert/strict";
import test from "node:test";
import { SCRIPT_BASENAMES } from "./background.ts";
import {
  applyNavigation,
  createNavigationDocument,
  sections,
} from "./support/navigation-test-dom.js";

// The docs site has no module loader: the theme scripts are concatenated in
// `SCRIPT_BASENAMES` order and talk through globals. Loading the i18n files in
// that same order is what makes this file exercise the shipped wiring.
for (const basename of SCRIPT_BASENAMES.filter((name) => name.startsWith("i18n/"))) {
  await import(`./${basename}.js`);
}

const navigation = globalThis.__vizeDocsNavigation;

for (const { code } of globalThis.__vizeDocsSitemap.supportedLocales) {
  void test(`navigation puts Philosophy in Start and Content Mapper in analysis (${code})`, () => {
    const prefix = code === "en" ? "" : `/${code}`;
    const document = createNavigationDocument([
      [`${prefix}/getting-started`, "Getting Started"],
      [`${prefix}/philosophy`, "Philosophy"],
      [`${prefix}/guide/content-mapper`, "Content Mapper"],
      [`${prefix}/guide/musea`, "Musea"],
    ]);
    applyNavigation(document, `${prefix}/guide/content-mapper/index.html`);
    const strings = globalThis.__vizeDocsLocales[code];
    assert.deepEqual(sections(document), [
      {
        title: strings.ui.groups.start,
        labels: [strings.labels["/getting-started"], strings.labels["/philosophy"]],
      },
      {
        title: strings.ui.groups.staticAnalysis,
        labels: [strings.labels["/guide/content-mapper"]],
      },
      { title: strings.ui.groups.tooling, labels: [strings.labels["/guide/musea"]] },
    ]);
    assert.deepEqual(
      document.querySelectorAll(".nav-link[href]").map((link) => link.getAttribute("href")),
      [
        `${prefix}/getting-started`,
        `${prefix}/philosophy`,
        `${prefix}/guide/content-mapper`,
        `${prefix}/guide/musea`,
      ],
    );
  });
}

void test("applyNavigationOrder keeps the Blog group compact", () => {
  const document = createNavigationDocument([
    ["/", "index"],
    ["/blog", "Blog"],
    ["/blog/notes", "Notes"],
    ["/blog/releases", "Releases"],
    [
      "/blog/notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain",
      "2026-05-16-performance-tuning-notes-for-a-vue-toolchain",
    ],
    ["/blog/releases/2026-03-26-docs-blog-support", "2026-03-26-docs-blog-support"],
  ]);

  applyNavigation(document);

  assert.deepEqual(sections(document), [
    {
      title: "Start",
      labels: ["Overview"],
    },
    {
      title: "Blog",
      labels: ["Overview", "Notes", "Releases"],
    },
  ]);
});

void test("applyNavigationOrder places the stability contract in detailed guides", () => {
  const document = createNavigationDocument([
    ["/", "index"],
    ["/getting-started", "Getting Started"],
    ["/stability", "Stability"],
    ["/credits", "Credits"],
  ]);

  applyNavigation(document);

  assert.deepEqual(sections(document), [
    {
      title: "Start",
      labels: ["Overview", "Getting Started"],
    },
    {
      title: "More guides",
      labels: ["Stability", "Credits"],
    },
  ]);
});

void test("applyNavigationOrder groups component exploration together", () => {
  const document = createNavigationDocument([
    ["/guide/musea", "Musea"],
    ["/guide/ui-styles", "UI styles"],
  ]);

  applyNavigation(document);

  assert.deepEqual(sections(document), [
    {
      title: "Explore components",
      labels: ["Musea", "UI Styles"],
    },
  ]);
});

void test("applyNavigationOrder keeps developer architecture pages together", () => {
  const document = createNavigationDocument([
    ["/architecture/overview", "Architecture"],
    ["/architecture/crates", "Crates"],
    ["/architecture/source-guide", "Source Guide"],
    ["/architecture/language-engineering-practices", "Language Engineering Practices"],
    ["/architecture/performance", "Performance"],
  ]);

  applyNavigation(document);

  assert.deepEqual(sections(document), [
    {
      title: "More guides",
      labels: [
        "Architecture Overview",
        "Crates",
        "Source Guide",
        "Language Engineering",
        "Performance",
      ],
    },
  ]);
});

void test("applyNavigationOrder hides dated blog posts from the More fallback", () => {
  const document = createNavigationDocument([
    ["/internal/reference", "Internal Reference"],
    [
      "/blog/notes/2026-05-16-performance-tuning-notes-for-a-vue-toolchain",
      "2026-05-16-performance-tuning-notes-for-a-vue-toolchain",
    ],
    [
      "/blog/notes/2026-05-17-long-note-that-is-not-in-the-curated-blog-navigation",
      "2026-05-17-long-note-that-is-not-in-the-curated-blog-navigation",
    ],
    [
      "/blog/releases/2026-05-17-long-release-that-is-not-in-the-curated-blog-navigation",
      "2026-05-17-long-release-that-is-not-in-the-curated-blog-navigation",
    ],
  ]);

  applyNavigation(document);

  assert.deepEqual(sections(document), [
    {
      title: "More",
      labels: ["Internal Reference"],
    },
  ]);
});

void test("applyNavigationOrder keeps only the current locale and localizes labels", () => {
  const document = createNavigationDocument([
    ["/getting-started", "Getting Started"],
    ["/ja/getting-started", "getting-started"],
    ["/ja/guide/static-analysis", "static-analysis"],
    ["/fr/getting-started", "getting-started"],
  ]);

  applyNavigation(document, "/ja/guide/static-analysis/index.html");

  assert.deepEqual(sections(document), [
    {
      title: "スタート",
      labels: ["はじめに"],
    },
    {
      title: "コードを検査する",
      labels: ["静的解析"],
    },
  ]);
  assert.equal(
    navigation.canonicalPath("/ja/guide/static-analysis/index.html"),
    "/guide/static-analysis",
  );
});

void test("navigation opens the current goal and keeps generated leaf pages out of the sidebar", () => {
  const document = createNavigationDocument([
    ["/getting-started", "Getting Started"],
    ["/guide/vite-plus", "Vite+"],
    ["/guide/migration", "Migration"],
    ["/rules/all", "All Rules"],
    ["/rules/reference/vue-no-v-html", "vue/no-v-html"],
    ["/guide/ui", "UI"],
    ["/guide/ui/button", "Button"],
    ["/guide/configuration-reference", "Standalone reference"],
  ]);
  applyNavigation(document, "/rules/reference/vue-no-v-html/index.html");
  assert.deepEqual(sections(document), [
    { title: "Start", labels: ["Getting Started", "Vite+ integration", "Migration"] },
    { title: "Rule reference", labels: ["All Rules"] },
    { title: "Explore components", labels: ["UI Reference"] },
  ]);
  const groups = document.querySelectorAll(".nav-section");
  assert.deepEqual(
    groups.map((group) => group.tagName),
    ["DETAILS", "DETAILS", "DETAILS"],
  );
  assert.deepEqual(
    groups.map((group) => group.getAttribute("open") !== null),
    [true, true, false],
  );
  assert.equal(groups[0].querySelector(".nav-title").tagName, "SUMMARY");
});
