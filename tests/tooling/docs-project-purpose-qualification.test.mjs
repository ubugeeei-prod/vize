import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";

import {
  verifyJapaneseEditorial,
  verifyRuleSources,
} from "../../docs/scripts/rule-render-assertions.ts";
import {
  japaneseEditorialPlacementSuccessors,
  japaneseEditorialReferenceCases,
} from "../../docs/scripts/rules/japanese-editorial-successors.ts";
import {
  projectPurpose,
  projectScopeQualification,
} from "../../docs/scripts/rules/project-purposes.ts";
import { chineseShared } from "../../docs/scripts/rules/category-locales/zh-shared.ts";
import { portugueseShared } from "../../docs/scripts/rules/category-locales/pt-shared.ts";
import { frenchShared } from "../../docs/scripts/rules/category-locales/fr-shared.ts";
import { ruleTranslator } from "../../docs/scripts/rules/rule-translations.ts";

const root = resolve(import.meta.dirname, "../..");
const read = (file) => readFileSync(resolve(root, file), "utf8");
const digest = (value) => createHash("sha256").update(value).digest("hex");
const fixtureSource = read("tests/_fixtures/docs-ja-rule-editorial.json");
const fixture = JSON.parse(fixtureSource);
const reviewed = fixture.cases.filter((entry) => /^B0[1-5]$/.test(entry.case));
const fences = (source) =>
  [...source.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)\n```$/gm)].map(([, language, body]) => [
    language,
    body,
  ]);
const prose = (markdown) => markdown.replace(/`([^`]+)`/g, "$1");
const frozenExampleHashes = {
  "array-mutation": "5d59367aed347e92b78fed8b8147eaab409ff3146f3b460c255462f54a051c54",
  "pinia-getter": "a15d1b4e53c819ab7c1edd581c55c3b3144ce68c8e0e7c8abb706b02ff622ace",
  "setup-context-violation": "2a212a98a60ae7944c7fd3008d966624132415f553012271f6ca0a2cc551e866",
  "object-identity-comparison": "28aa25a5b6b8f058645927e07a991838f548291ff3dc02e2ec5b6ea971dc0bea",
  "watcheffect-async": "a7a9d2d56eaf6749b65151c67e816525257cd52b832e520a8ce64915f385bad7",
};
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi");
const { Window } = createRequire(new URL("../package.json", import.meta.url))("happy-dom");
const window = new Window();
const documentFor = (html) => new window.DOMParser().parseFromString(html, "text/html");

function render(file) {
  const transformed = native.transform(read(file), {
    convertMdLinks: true,
    baseUrl: "/",
    sourcePath: resolve(root, file),
  });
  assert.deepEqual(transformed.errors, [], file);
  return `<main class="content">${transformed.html}</main>`;
}

void test("only B01 through B05 move, with the frozen paragraphs and their exact new purposes", () => {
  assert.equal(
    digest(fixtureSource),
    "92e7107ccf0b57d0cb59dce971814f26531b6f2b7b33b0ecf8fb5bdde8ecd39b",
  );
  assert.deepEqual(Object.keys(japaneseEditorialPlacementSuccessors), [
    "B01",
    "B02",
    "B03",
    "B04",
    "B05",
  ]);
  const placed = japaneseEditorialReferenceCases(fixture.cases);
  for (const entry of placed) {
    const original = fixture.cases.find((row) => row.case === entry.case);
    if (!japaneseEditorialPlacementSuccessors[entry.case]) {
      assert.deepEqual(entry, { ...original, targetSection: original.section });
      continue;
    }
    assert.equal(entry.targetSection, "qualification");
    assert.equal(entry.qualificationHeading, "範囲と制約");
    assert.equal(entry.after, original.after);
    const name = entry.rule.slice("vize:croquis/cf/".length);
    assert.equal(projectPurpose(name, "foreign", true), entry.expectedPurpose);
    assert.equal(projectScopeQualification(name, true), original.after);
    assert.notEqual(entry.expectedPurpose, original.before);
  }
  for (const edit of [
    (entries) => entries.filter((entry) => entry.case !== "B01"),
    (entries) => [...entries, reviewed[0]],
    (entries) =>
      entries.map((entry) => (entry.case === "B01" ? { ...entry, rule: "foreign/rule" } : entry)),
    (entries) =>
      entries.map((entry) => (entry.case === "B01" ? { ...entry, section: "good" } : entry)),
    (entries) =>
      entries.map((entry) =>
        entry.case === "B01" ? { ...entry, after: entry.after.slice(0, -1) } : entry,
      ),
  ])
    assert.throws(
      () => japaneseEditorialReferenceCases(edit(fixture.cases)),
      assert.AssertionError,
    );
});

void test("all five locales render separate purposes and full qualifications with unchanged code", () => {
  const locales = {
    en: { heading: "Scope and limitations" },
    ja: { heading: "範囲と制約" },
    "zh-CN": { heading: "适用范围与限制", shared: chineseShared },
    "pt-BR": { heading: "Escopo e limitações", shared: portugueseShared },
    fr: { heading: "Champ d’application et limites", shared: frenchShared },
  };
  for (const [locale, { heading, shared }] of Object.entries(locales)) {
    const prefix = locale === "en" ? "" : `${locale}/`;
    verifyRuleSources(`/${prefix}rules/cross-file`);
    verifyRuleSources(`/${prefix}rules/all`);
    for (const entry of reviewed) {
      const name = entry.rule.slice("vize:croquis/cf/".length);
      const file = `docs/content/${prefix}rules/project/vize-croquis-cf-${name}.md`;
      const source = read(file);
      assert.equal(
        digest(JSON.stringify(fences(source))),
        frozenExampleHashes[name],
        `${file}: complete original examples`,
      );
      const purpose =
        locale === "en" || locale === "ja"
          ? projectPurpose(name, "foreign", locale === "ja")
          : ruleTranslator(entry.rule, locale, true).purpose;
      const originalParagraph = projectScopeQualification(name, false);
      const qualification =
        locale === "ja"
          ? entry.after
          : locale === "en"
            ? originalParagraph
            : shared[originalParagraph];
      assert.ok(qualification, `${file}: complete translated qualification`);
      const document = documentFor(render(file));
      const first = document.querySelector("h1").nextElementSibling;
      assert.equal(first.tagName, "P");
      assert.equal(first.textContent, prose(purpose), `${file}: exact purpose`);
      assert.notEqual(first.textContent, prose(entry.before));
      const headings = [...document.querySelectorAll("h2")].filter(
        (node) => node.textContent === heading,
      );
      assert.equal(headings.length, 1, file);
      const paragraph = headings[0].nextElementSibling;
      assert.equal(paragraph.tagName, "P");
      assert.equal(
        paragraph.textContent,
        prose(qualification),
        `${file}: whole reviewed qualification`,
      );
      assert.deepEqual(
        [...document.querySelectorAll("pre code")].map((code) =>
          code.textContent.replace(/\n$/, ""),
        ),
        fences(source).map(([, body]) => body),
        `${file}: native complete code`,
      );
    }
  }
});

const renderedRoutes = new Map();
function browserPage(edit = (_route, html) => html) {
  return {
    async evaluate(callback, args) {
      const globals = {
        DOMParser: window.DOMParser,
        HTMLParagraphElement: window.HTMLParagraphElement,
        fetch: async (url) => {
          const route = new URL(url, "https://docs.test").pathname.replace(/\/index\.html$/, "");
          if (!renderedRoutes.has(route))
            renderedRoutes.set(route, render(`docs/content${route}.md`));
          return { status: 200, text: async () => edit(route, renderedRoutes.get(route)) };
        },
      };
      const saved = Object.fromEntries(
        Object.keys(globals).map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)]),
      );
      try {
        for (const [key, value] of Object.entries(globals))
          Object.defineProperty(globalThis, key, { configurable: true, writable: true, value });
        return await callback(args);
      } finally {
        for (const [key, descriptor] of Object.entries(saved)) {
          if (descriptor) Object.defineProperty(globalThis, key, descriptor);
          else delete globalThis[key];
        }
      }
    },
  };
}

void test("the actual browser editorial gate accepts every frozen paragraph at its explicit placement", async () => {
  const receipt = await verifyJapaneseEditorial(browserPage());
  assert.equal(receipt.originalSourceSha, fixture.originalSourceSha);
  assert.equal(receipt.paragraphs.length, fixture.cases.length);
  assert.deepEqual(
    receipt.placementSuccessors.map((entry) => [entry.case, entry.from, entry.to]),
    reviewed.map((entry) => [entry.case, "purpose", "qualification"]),
  );
});

void test("the actual browser gate rejects old purposes, missing qualifications and partial paragraphs", async () => {
  for (const entry of reviewed) {
    const route = `/ja/rules/project/${entry.rule.replaceAll(/[^a-zA-Z0-9]+/g, "-")}`;
    await assert.rejects(
      verifyJapaneseEditorial(
        browserPage((at, html) => {
          if (at !== route) return html;
          const document = documentFor(html);
          document.querySelector("h1").nextElementSibling.textContent = prose(entry.before);
          return document.body.innerHTML;
        }),
      ),
      assert.AssertionError,
    );
  }
  const route = "/ja/rules/project/vize-croquis-cf-array-mutation";
  for (const alter of [
    (heading) => heading.nextElementSibling.remove(),
    (heading) => {
      heading.nextElementSibling.textContent += " 追加の誤った断定。";
    },
    (heading) => {
      heading.nextElementSibling.textContent = heading.nextElementSibling.textContent.replace(
        "Vue 2",
        "Vue  2",
      );
    },
    (heading) => {
      heading.before(heading.cloneNode(true));
    },
    (heading) => {
      heading.nextElementSibling.textContent = "Vue 2 と Vue 3 の一部の説明だけ。";
    },
  ])
    await assert.rejects(
      verifyJapaneseEditorial(
        browserPage((at, html) => {
          if (at !== route) return html;
          const document = documentFor(html);
          alter(
            [...document.querySelectorAll("h2")].find((node) => node.textContent === "範囲と制約"),
          );
          return document.body.innerHTML;
        }),
      ),
      assert.AssertionError,
    );
});
