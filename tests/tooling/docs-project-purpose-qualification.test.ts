import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { test } from "node:test";
import { Window, type Element } from "happy-dom";
import type { Page } from "playwright";

import {
  verifyJapaneseEditorial,
  verifyRuleSources,
} from "../../docs/scripts/rule-render-assertions.ts";
import {
  japaneseEditorialPlacementSuccessors,
  japaneseEditorialReferenceCases,
  type EditorialCase,
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
const read = (file: string) => readFileSync(resolve(root, file), "utf8");
const digest = (value: string) => createHash("sha256").update(value).digest("hex");
const fixtureSource = read("tests/_fixtures/docs-ja-rule-editorial.json");
const fixture = JSON.parse(fixtureSource) as {
  originalSourceSha: string;
  cases: EditorialCase[];
};
const reviewed = fixture.cases.filter((entry) => /^B0[1-5]$/.test(entry.case));
const fences = (source: string) =>
  [...source.matchAll(/^```(\w+)[^\n]*\n([\s\S]*?)\n```$/gm)].map(([, language, body]) => [
    language,
    body,
  ]);
const prose = (markdown: string) => markdown.replace(/`([^`]+)`/g, "$1");
const frozenExampleHashes: Readonly<Record<string, string>> = {
  "array-mutation": "5d59367aed347e92b78fed8b8147eaab409ff3146f3b460c255462f54a051c54",
  "pinia-getter": "a15d1b4e53c819ab7c1edd581c55c3b3144ce68c8e0e7c8abb706b02ff622ace",
  "setup-context-violation": "2a212a98a60ae7944c7fd3008d966624132415f553012271f6ca0a2cc551e866",
  "object-identity-comparison": "28aa25a5b6b8f058645927e07a991838f548291ff3dc02e2ec5b6ea971dc0bea",
  "watcheffect-async": "4af363d7b8cc110b50327e52c3e8638c06a0ceb694a708bb3ea72a52e7355dcb",
};
const require = createRequire(new URL("../../docs/package.json", import.meta.url));
const native = createRequire(require.resolve("@ox-content/vite-plugin"))("@ox-content/napi") as {
  transform(
    source: string,
    options: { convertMdLinks: boolean; baseUrl: string; sourcePath: string },
  ): { html: string; errors: unknown[] };
};
const window = new Window();
const documentFor = (html: string) => new window.DOMParser().parseFromString(html, "text/html");
const following = (element: Element | null | undefined) => {
  assert.ok(element, "the reviewed heading exists");
  const next = element.nextElementSibling;
  assert.ok(next, "the reviewed paragraph follows its heading");
  return next;
};

function render(file: string) {
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
    "da41aa4a66062f23542b137fefa6d92eb636e0bf633f2a99742081df6a582024",
  );
  assert.deepEqual(Object.keys(japaneseEditorialPlacementSuccessors), [
    "B01",
    "B02",
    "B03",
    "B04",
    "B05",
  ]);
  const placed = japaneseEditorialReferenceCases(fixture.cases);
  const moved = new Set<string>(Object.keys(japaneseEditorialPlacementSuccessors));
  for (const entry of placed) {
    const original = fixture.cases.find((row) => row.case === entry.case);
    assert.ok(original);
    if (!moved.has(entry.case)) {
      assert.deepEqual(entry, { ...original, targetSection: original.section });
      continue;
    }
    assert.ok(entry.targetSection === "qualification");
    assert.equal(entry.qualificationHeading, "範囲と制約");
    assert.equal(entry.after, original.after);
    const name = entry.rule.slice("vize:croquis/cf/".length);
    assert.equal(projectPurpose(name, "foreign", true), entry.expectedPurpose);
    assert.equal(projectScopeQualification(name, true), original.after);
    assert.notEqual(entry.expectedPurpose, original.before);
  }
  const edits: Array<(entries: EditorialCase[]) => EditorialCase[]> = [
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
  ];
  for (const edit of edits)
    assert.throws(
      () => japaneseEditorialReferenceCases(edit(fixture.cases)),
      assert.AssertionError,
    );
});

void test("all five locales render separate purposes and full qualifications with unchanged code", () => {
  const locales: Record<
    "en" | "ja" | "zh-CN" | "pt-BR" | "fr",
    { heading: string; shared?: Readonly<Record<string, string>> }
  > = {
    en: { heading: "Scope and limitations" },
    ja: { heading: "範囲と制約" },
    "zh-CN": { heading: "适用范围与限制", shared: chineseShared },
    "pt-BR": { heading: "Escopo e limitações", shared: portugueseShared },
    fr: { heading: "Champ d’application et limites", shared: frenchShared },
  };
  for (const locale of Object.keys(locales) as Array<keyof typeof locales>) {
    const { heading, shared } = locales[locale];
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
      assert.ok(typeof purpose === "string");
      const originalParagraph = projectScopeQualification(name, false);
      assert.ok(typeof originalParagraph === "string");
      let qualification = entry.after;
      if (locale === "en") qualification = originalParagraph;
      else if (locale !== "ja") {
        assert.ok(shared);
        qualification = shared[originalParagraph];
      }
      assert.ok(qualification, `${file}: complete translated qualification`);
      const document = documentFor(render(file));
      const first = following(document.querySelector("h1"));
      assert.equal(first.tagName, "P");
      assert.equal(first.textContent, prose(purpose), `${file}: exact purpose`);
      assert.notEqual(first.textContent, prose(entry.before));
      const headings = [...document.querySelectorAll("h2")].filter(
        (node) => node.textContent === heading,
      );
      assert.equal(headings.length, 1, file);
      const paragraph = following(headings[0]);
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

const renderedRoutes = new Map<string, string>();
type HtmlEdit = (route: string, html: string) => string;
function browserPage(edit: HtmlEdit = (_route, html) => html): Page {
  return {
    async evaluate(callback: (args: unknown) => Promise<unknown>, args: unknown) {
      const globals = {
        DOMParser: window.DOMParser,
        HTMLParagraphElement: window.HTMLParagraphElement,
        fetch: async (url: string) => {
          const route = new URL(url, "https://docs.test").pathname.replace(/\/index\.html$/, "");
          if (!renderedRoutes.has(route))
            renderedRoutes.set(route, render(`docs/content${route}.md`));
          const html = renderedRoutes.get(route);
          assert.ok(html !== undefined);
          return { status: 200, text: async () => edit(route, html) };
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
          else Reflect.deleteProperty(globalThis, key);
        }
      }
    },
  } as unknown as Page;
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
          following(document.querySelector("h1")).textContent = prose(entry.before);
          return document.body.innerHTML;
        }),
      ),
      assert.AssertionError,
    );
  }
  const route = "/ja/rules/project/vize-croquis-cf-array-mutation";
  const alterations: Array<(heading: Element) => void> = [
    (heading) => following(heading).remove(),
    (heading) => {
      following(heading).textContent += " 追加の誤った断定。";
    },
    (heading) => {
      following(heading).textContent = following(heading).textContent.replace("Vue 2", "Vue  2");
    },
    (heading) => {
      heading.before(heading.cloneNode(true));
    },
    (heading) => {
      following(heading).textContent = "Vue 2 と Vue 3 の一部の説明だけ。";
    },
  ];
  for (const alter of alterations)
    await assert.rejects(
      verifyJapaneseEditorial(
        browserPage((at, html) => {
          if (at !== route) return html;
          const document = documentFor(html);
          const heading = [...document.querySelectorAll("h2")].find(
            (node) => node.textContent === "範囲と制約",
          );
          assert.ok(heading);
          alter(heading);
          return document.body.innerHTML;
        }),
      ),
      assert.AssertionError,
    );
});
