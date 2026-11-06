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
  void test(`navigation files Content Mapper under Type checker and keeps Getting started open (${code})`, () => {
    const prefix = code === "en" ? "" : `/${code}`;
    // The rendered sidebar lists English pages; the theme localizes placed links.
    const document = createNavigationDocument([
      ["/getting-started", "Getting Started"],
      ["/philosophy", "Philosophy"],
      ["/guide/content-mapper", "Content Mapper"],
      ["/guide/musea", "Musea"],
    ]);
    applyNavigation(document, `${prefix}/guide/content-mapper/index.html`);
    const strings = globalThis.__vizeDocsLocales[code];
    assert.deepEqual(sections(document), [
      {
        title: strings.ui.groups.start,
        labels: [strings.labels["/getting-started"], strings.labels["/philosophy"]],
      },
      {
        title: strings.ui.groups.typeChecker,
        labels: [strings.labels["/guide/content-mapper"]],
      },
      { title: strings.ui.groups.components, labels: [strings.labels["/guide/musea"]] },
    ]);
    assert.deepEqual(
      document.querySelectorAll(".nav-section").map((group) => group.getAttribute("open") !== null),
      [true, true, false],
    );
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
      title: "Getting started",
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
      title: "Getting started",
      labels: ["Overview", "Quick start"],
    },
    {
      title: "Project and architecture",
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
      title: "Components",
      labels: ["Musea", "UI styles"],
    },
  ]);
});

void test("applyNavigationOrder files former More pages under the task they belong to", () => {
  const document = createNavigationDocument([
    ["/guide/init", "Project Setup"],
    ["/guide/lint-execution", "Native lint execution settings"],
    ["/guide/formatter-indent-width", "Indentation width"],
    ["/rules/options", "Rule Options"],
    ["/rules/vue", "Vue rules"],
    ["/rules/options-vue", "Vue Rule Options"],
    ["/rules/options-events", "Event Rule Options"],
    ["/rules/options-attributes", "Attribute Rule Options"],
    ["/rules/options-props", "Prop Rule Options"],
    ["/rules/type-and-script", "Type and script rules"],
    ["/rules/options-script", "Script Rule Options"],
    ["/rules/options-type", "Type Rule Options"],
    ["/rules/musea-and-css", "Musea and CSS rules"],
    ["/rules/css", "CSS rules"],
    ["/rules/musea", "Musea rules"],
    ["/rules/options-musea", "Musea Rule Options"],
    ["/guide/musea-hosting", "Musea Hosting"],
    ["/integrations/mcp", "MCP Server"],
    ["/guide/doctor", "Doctor Diagnostics"],
    ["/guide/check-performance", "Type-checking performance"],
    ["/guide/vite-plus-editor", "Vite+ editor setup"],
    ["/guide/compiler-inspector", "Compiler Inspector"],
    ["/contributing", "Contributing"],
    ["/guide/experimentals-reference", "Experimentals Reference"],
  ]);

  applyNavigation(document);

  // No trailing "More" section: every page has a named group, and the
  // experimentals reference is reached from the Experimentals page instead.
  assert.deepEqual(sections(document), [
    { title: "Getting started", labels: ["Project initializer"] },
    { title: "Linter", labels: ["Lint run settings"] },
    { title: "Formatter", labels: ["Indent width"] },
    { title: "Type checker", labels: ["Type-check performance"] },
    { title: "Editor", labels: ["Editor setup"] },
    { title: "Compiler and integrations", labels: ["Compiler inspector"] },
    { title: "Components", labels: ["Musea hosting", "MCP server"] },
    { title: "CLI and tools", labels: ["Doctor"] },
    {
      title: "Rule reference",
      labels: [
        "Rule options",
        "Overview",
        "Vue",
        "Events",
        "Attributes",
        "Props",
        "Overview",
        "Script",
        "Type",
        "Overview",
        "Overview",
        "Musea",
      ],
    },
    { title: "Vue", labels: ["Overview", "Vue", "Events", "Attributes", "Props"] },
    { title: "Type & script", labels: ["Overview", "Script", "Type"] },
    { title: "CSS", labels: ["Overview"] },
    { title: "Musea", labels: ["Overview", "Musea"] },
    { title: "Project and architecture", labels: ["Contributing"] },
  ]);
});

void test("applyNavigationOrder lists the option references under Configuration", () => {
  const document = createNavigationDocument([
    ["/guide/configuration", "Configuration"],
    ["/guide/configuration-reference", "Shared Configuration Reference"],
    ["/guide/compiler-configuration-reference", "Compiler Configuration Reference"],
    ["/guide/experimentals", "Experimentals"],
    ["/guide/experimentals-vue-rfcs", "Vue RFC experimentals"],
  ]);

  applyNavigation(document, "/guide/compiler-configuration-reference/index.html");

  assert.deepEqual(sections(document), [
    {
      title: "Configuration",
      labels: [
        "Overview",
        "Configuration reference",
        "Compiler options reference",
        "Experimental features",
      ],
    },
  ]);
  assert.notEqual(document.querySelector(".nav-section").getAttribute("open"), null);
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
      title: "Project and architecture",
      labels: [
        "Architecture overview",
        "Crates",
        "Source guide",
        "Language engineering",
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

void test("applyNavigationOrder points English links at the reader's locale and localizes labels", () => {
  const document = createNavigationDocument([
    ["/getting-started", "Getting Started"],
    ["/guide/static-analysis", "Static Analysis"],
    ["/rules/options", "Rule Options"],
    ["/rules/vue", "Vue rules"],
    ["/rules/options-vue", "Vue Rule Options"],
    ["/guide/doctor", "Doctor"],
  ]);

  applyNavigation(document, "/ja/guide/static-analysis/index.html");

  assert.deepEqual(sections(document), [
    {
      title: "はじめに",
      labels: ["クイックスタート"],
    },
    {
      title: "リンター",
      labels: ["静的解析"],
    },
    { title: "CLI とツール", labels: ["Doctor"] },
    { title: "ルールリファレンス", labels: ["ルールのオプション", "概要", "Vue"] },
    { title: "Vue", labels: ["概要", "Vue"] },
  ]);
  assert.deepEqual(
    document
      .querySelectorAll(".nav-link[href]")
      .map((link) => [link.getAttribute("href"), link.className]),
    [
      ["/ja/getting-started", "nav-link"],
      ["/ja/guide/static-analysis", "nav-link active"],
      ["/ja/guide/doctor", "nav-link"],
      ["/ja/rules/options", "nav-link"],
      ["/ja/rules/vue", "nav-link"],
      ["/ja/rules/options-vue", "nav-link"],
    ],
  );
  assert.equal(
    navigation.canonicalPath("/ja/guide/static-analysis/index.html"),
    "/guide/static-analysis",
  );

  // zh-CN publishes neither the options pages nor Doctor: no section, no fallback link.
  const chinese = createNavigationDocument([
    ["/getting-started", "Getting Started"],
    ["/rules/options", "Rule Options"],
    ["/rules/options-vue", "Vue Rule Options"],
    ["/guide/doctor", "Doctor"],
  ]);
  applyNavigation(chinese, "/zh-CN/getting-started/index.html");
  assert.deepEqual(
    chinese.querySelectorAll(".nav-link[href]").map((link) => link.getAttribute("href")),
    ["/zh-CN/getting-started"],
  );
});

void test("navigation opens the current goal and files rule pages under their category", () => {
  const document = createNavigationDocument([
    ["/getting-started", "Getting Started"],
    ["/guide/vite-plus", "Vite+"],
    ["/guide/migration", "Migration"],
    ["/rules/all", "All Rules"],
    ["/rules/vue", "Vue rules"],
    ["/rules/reference/vue-no-v-html", "vue/no-v-html"],
    ["/rules/reference/vue-attribute-order", "vue/attribute-order"],
    ["/rules/html", "HTML rules"],
    ["/rules/reference/html-no-dupe-attrs", "html/no-dupe-attrs"],
    ["/guide/ui", "UI"],
    ["/guide/ui/button", "Button"],
    ["/guide/composables", "Composables"],
    ["/guide/composables/use-toggle", "useToggle"],
  ]);
  applyNavigation(document, "/rules/reference/vue-no-v-html/index.html");
  assert.deepEqual(sections(document), [
    { title: "Getting started", labels: ["Quick start", "Vite+ integration", "Migration"] },
    { title: "Components", labels: ["UI components", "Composables"] },
    {
      title: "Rule reference",
      labels: [
        "All rules",
        "Overview",
        "vue/attribute-order",
        "vue/no-v-html",
        "Overview",
        "html/no-dupe-attrs",
      ],
    },
    { title: "Vue", labels: ["Overview", "vue/attribute-order", "vue/no-v-html"] },
    { title: "HTML", labels: ["Overview", "html/no-dupe-attrs"] },
  ]);
  const groups = document.querySelectorAll(".nav-section");
  assert.deepEqual(
    groups.map((group) => group.tagName),
    ["DETAILS", "DETAILS", "DETAILS", "DETAILS", "DETAILS"],
  );
  // Only the current page's group and rule category open.
  assert.deepEqual(
    groups.map((group) => group.getAttribute("open") !== null),
    [true, false, true, true, false],
  );
  assert.deepEqual(
    groups.map((group) => group.querySelector(".nav-list").inert),
    [false, true, false, false, true],
  );
  assert.equal(groups[0].querySelector(".nav-title").tagName, "SUMMARY");
});
