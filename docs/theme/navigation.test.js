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
  void test(`navigation puts Content Mapper in the open Start section (${code})`, () => {
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
        labels: [
          strings.labels["/getting-started"],
          strings.labels["/guide/content-mapper"],
          strings.labels["/philosophy"],
        ],
      },
      { title: strings.ui.groups.tooling, labels: [strings.labels["/guide/musea"]] },
    ]);
    assert.deepEqual(
      document.querySelectorAll(".nav-link[href]").map((link) => link.getAttribute("href")),
      [
        `${prefix}/getting-started`,
        `${prefix}/guide/content-mapper`,
        `${prefix}/philosophy`,
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

void test("applyNavigationOrder places the stability contract with the project pages", () => {
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
      title: "Project & internals",
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

void test("applyNavigationOrder files former More pages under the task they belong to", () => {
  const document = createNavigationDocument([
    ["/guide/init", "Project Setup"],
    ["/guide/lint-execution", "Native lint execution settings"],
    ["/guide/formatter-indent-width", "Indentation width"],
    ["/rules/options-vue", "Vue Rule Options"],
    ["/rules/vue-sfc", "Vue Rules: SFC Blocks"],
    ["/guide/musea-hosting", "Musea Hosting"],
    ["/guide/doctor", "Doctor Diagnostics"],
    ["/contributing", "Contributing"],
    ["/guide/experimentals-reference", "Experimentals Reference"],
  ]);

  applyNavigation(document);

  // No trailing "More" section: every page has a named group, and the
  // experimentals reference is reached from the Experimentals page instead.
  assert.deepEqual(sections(document), [
    { title: "Start", labels: ["Project initializer"] },
    { title: "Check code", labels: ["Lint Execution"] },
    { title: "Format code", labels: ["Indent Width"] },
    { title: "Rule reference", labels: ["Options: Vue"] },
    { title: "Rules by topic", labels: ["Vue: SFC Blocks"] },
    { title: "Explore components", labels: ["Musea Hosting"] },
    { title: "CLI & tools", labels: ["Doctor"] },
    { title: "Project & internals", labels: ["Contributing"] },
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
      title: "Project & internals",
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
      labels: ["はじめる"],
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
  assert.deepEqual(
    groups.map((group) => group.querySelector(".nav-list").inert),
    [false, false, true],
  );
  assert.equal(groups[0].querySelector(".nav-title").tagName, "SUMMARY");
});
