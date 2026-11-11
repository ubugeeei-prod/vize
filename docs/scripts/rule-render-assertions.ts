import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "playwright";
import { verifyRenderedRuleIndentation } from "./rule-indentation-render-assertions.ts";
import { groupIncludes, ruleGroups } from "./rules/rule-groups.ts";
import { ruleHeadings } from "./rules/rule-page.ts";
import {
  japaneseEditorialReferenceCases,
  type EditorialCase,
} from "./rules/japanese-editorial-successors.ts";
import {
  isOperationalDiagnosticReference,
  operationalDiagnosticPages,
} from "./rules/operational-diagnostics.ts";

const root = resolve(import.meta.dirname, "../..");
const locales = ["en", "ja", "zh-CN", "pt-BR", "fr"] as const;
type Locale = (typeof locales)[number];
const prefix = (locale: Locale) => (locale === "en" ? "" : `${locale}/`);

export const ruleRenderRoutes = [
  "/rules",
  "/rules/all",
  ...ruleGroups.map((group) => `/rules/${group}`),
  "/rules/cross-file",
  "/rules/reference/script-define-props-destructuring",
  "/rules/reference/a11y-form-control-has-label",
  "/rules/reference/vue-no-preprocessor-lang",
  ...operationalDiagnosticPages.map(([, slug]) => `/rules/reference/${slug}`),
  "/rules/project/vize-croquis-cf-array-mutation",
  "/rules/project/vize-croquis-cf-circular-reactive-dependency",
];

const editorial = JSON.parse(
  readFileSync(resolve(root, "tests/_fixtures/docs-ja-rule-editorial.json"), "utf8"),
) as { originalSourceSha: string; cases: EditorialCase[] };
const editorialText = (markdown: string) =>
  markdown
    .replace(/`([^`]+)`/g, "$1")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replace(/\s+/g, " ")
    .trim();

/** Which rule index a route renders, if any. */
export function ruleRouteKind(route: string) {
  return routeOf(route)?.kind ?? null;
}

function routeOf(route: string) {
  const match = route.match(/^\/(?:(ja|zh-CN|pt-BR|fr)\/)?rules\/([^/]+)$/);
  if (!match) return null;
  const locale = (match[1] ?? "en") as Locale;
  const page = match[2];
  if (page === "all") return { kind: "all" as const, locale, page };
  if (page === "cross-file" || ruleGroups.includes(page))
    return { kind: "index" as const, locale, page };
  return null;
}

const fences = (source: string) =>
  [...source.matchAll(/^```[^\n]*\n([\s\S]*?)\n```$/gm)].map((match) => match[1]);
/** The first fenced block after a heading: the executed Bad or Good source. */
function exampleAfter(source: string, heading: string) {
  const section = source.split(`\n## ${heading}\n`)[1];
  const code = section?.match(/^```[^\n]*\n([\s\S]*?)\n```$/m)?.[1];
  assert.ok(code !== undefined, `missing ${heading} example`);
  return code;
}

/** Every rule page of one locale, with the code the English linter test executes. */
export function expectedRulePages(locale: Locale) {
  const pages = ["reference", "project"].flatMap((section) =>
    readdirSync(resolve(root, `docs/content/rules/${section}`))
      .filter((file) => section !== "reference" || !isOperationalDiagnosticReference(file))
      .sort()
      .map((file) => {
        const read = (at: Locale) =>
          readFileSync(resolve(root, `docs/content/${prefix(at)}rules/${section}/${file}`), "utf8");
        const source = read(locale);
        const english = read("en");
        const id = source.match(/^# `([^`]+)`/m)?.[1];
        assert.equal(id, english.match(/^# `([^`]+)`/m)?.[1], `${locale} ${file}: rule identity`);
        const headings = ruleHeadings[locale];
        for (const [key, heading] of [
          ["bad", headings.bad],
          ["good", headings.good],
        ] as const)
          assert.equal(
            exampleAfter(source, heading),
            exampleAfter(english, key === "bad" ? "Bad" : "Good"),
            `${locale} ${file}: ${key} source matches the executed English example`,
          );
        return {
          route: `/${prefix(locale)}rules/${section}/${file.replace(/\.md$/, "")}`,
          id: id!,
          code: fences(source),
          sourceSha256: createHash("sha256").update(source).digest("hex"),
        };
      }),
  );
  assert.equal(pages.length, 380, "314 rule references plus 66 project rules per locale");
  return pages;
}

/** The rule pages an index lists, from its committed Markdown. */
function expectedIndexLinks(locale: Locale, page: string) {
  const source = readFileSync(
    resolve(root, `docs/content/${prefix(locale)}rules/${page}.md`),
    "utf8",
  );
  const linked = [
    ...source.matchAll(
      /^\| <span id="[^"]+"><\/span>\[`([^`]+)`\]\(\.\/(reference|project)\/([^)]+)\.md\)/gm,
    ),
  ].map(([, id, section, slug]) => ({ id, section, slug }));
  for (const { section, slug } of linked)
    assert.ok(
      existsSync(resolve(root, `docs/content/${prefix(locale)}rules/${section}/${slug}.md`)),
      `${locale} ${page}: ${slug} has a page`,
    );
  const ids = (section: string) =>
    readdirSync(resolve(root, `docs/content/rules/${section}`))
      .filter((file) => section !== "reference" || !isOperationalDiagnosticReference(file))
      .map(
        (file) =>
          readFileSync(resolve(root, `docs/content/rules/${section}/${file}`), "utf8").match(
            /^# `([^`]+)`/m,
          )![1],
      );
  const expected =
    page === "all"
      ? [...ids("reference"), ...ids("project")]
      : page === "cross-file"
        ? ids("project")
        : ids("reference").filter((id) => groupIncludes(page, id));
  assert.deepEqual(
    linked.map(({ id }) => id).sort(),
    [...expected].sort(),
    `${locale} ${page}: the index lists exactly its rules`,
  );
  assert.doesNotMatch(
    source,
    new RegExp(`\\[${ruleHeadings[locale].bad}\\]\\(#`),
    `${locale} ${page}: no Bad/Good jump-link columns`,
  );
  return linked;
}

/** The review's source-side rule pins, for the merge queue's browser-free run. */
export function verifyRuleSources(route: string) {
  const target = routeOf(route);
  if (!target) return;
  expectedIndexLinks(target.locale, target.page);
  if (target.kind === "all") expectedRulePages(target.locale);
}

function rulePageRoute(locale: Locale, id: string) {
  const slug = id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
  const section = existsSync(resolve(root, `docs/content/rules/project/${slug}.md`))
    ? "project"
    : "reference";
  return `/${prefix(locale)}rules/${section}/${slug}`;
}

export async function verifyJapaneseEditorial(page: Page) {
  const cases = japaneseEditorialReferenceCases(editorial.cases).map((entry) => ({
    ...entry,
    route: rulePageRoute("ja", entry.rule),
  }));
  const actual = await page.evaluate(
    async ({ cases, headings }) =>
      Promise.all(
        cases.map(async (entry) => {
          const response = await fetch(`${entry.route}/index.html`);
          const document = new DOMParser().parseFromString(await response.text(), "text/html");
          const moved = entry.targetSection === "qualification";
          const headingsAt = [...document.querySelectorAll(".content h2")];
          const qualificationHeadings = moved
            ? headingsAt.filter(
                (heading) => heading.textContent?.trim() === entry.qualificationHeading,
              )
            : [];
          const anchor = moved
            ? qualificationHeadings[0]
            : entry.targetSection === "purpose"
              ? document.querySelector(".content h1")
              : headingsAt.find(
                  (heading) =>
                    heading.textContent?.trim() ===
                    (entry.targetSection === "bad" ? headings.bad : headings.good),
                );
          let next = anchor?.nextElementSibling;
          if (!moved)
            while (next && !(next instanceof HTMLParagraphElement)) next = next.nextElementSibling;
          const purpose = document.querySelector(".content h1")?.nextElementSibling;
          return {
            case: entry.case,
            status: response.status,
            text: moved
              ? next instanceof HTMLParagraphElement
                ? (next.textContent ?? "")
                : ""
              : (next?.textContent?.replace(/\s+/g, " ").trim() ?? ""),
            purpose:
              moved && purpose instanceof HTMLParagraphElement
                ? (purpose.textContent ?? "")
                : undefined,
            qualificationHeadings: qualificationHeadings.length,
          };
        }),
      ),
    { cases, headings: ruleHeadings.ja },
  );
  for (const [index, entry] of cases.entries()) {
    assert.equal(actual[index].status, 200, entry.route);
    assert.equal(
      actual[index].text,
      editorialText(entry.after),
      `${entry.rule}: ${entry.targetSection}`,
    );
    assert.notEqual(actual[index].text, editorialText(entry.before), `${entry.rule}: old wording`);
    if (entry.targetSection === "qualification") {
      assert.equal(
        actual[index].qualificationHeadings,
        1,
        `${entry.rule}: one qualification heading`,
      );
      assert.equal(
        actual[index].purpose,
        entry.expectedPurpose,
        `${entry.rule}: exact concise purpose`,
      );
      assert.notEqual(
        actual[index].purpose,
        editorialText(entry.before),
        `${entry.rule}: old purpose`,
      );
    }
  }
  return {
    originalSourceSha: editorial.originalSourceSha,
    paragraphs: actual.map(({ case: id, text }) => ({
      case: id,
      wholeTextSha256: createHash("sha256").update(text).digest("hex"),
    })),
    placementSuccessors: actual
      .filter((entry) => entry.purpose !== undefined)
      .map(({ case: id, purpose }) => ({
        case: id,
        from: "purpose",
        to: "qualification",
        purposeTextSha256: createHash("sha256").update(purpose!).digest("hex"),
      })),
  };
}

export async function verifyRenderedRulePackets(page: Page, route: string) {
  const target = routeOf(route);
  if (!target) return null;
  const links = expectedIndexLinks(target.locale, target.page);
  const rendered = await page.evaluate(() =>
    [...document.querySelectorAll(".content table a[href]")]
      .map((link) => new URL(link.getAttribute("href")!, location.href).pathname)
      .filter((path) => /\/rules\/(?:reference|project)\//.test(path)),
  );
  assert.equal(rendered.length, links.length, `${route}: every listed rule renders a link`);
  const statuses = await page.evaluate(
    async (paths) => Promise.all(paths.map(async (path) => (await fetch(path)).status)),
    rendered,
  );
  assert.deepEqual(new Set(statuses), new Set([200]), `${route}: every rule link resolves`);
  if (target.kind === "index") return { route, links: rendered.length };

  const expected = expectedRulePages(target.locale);
  const headings = ruleHeadings[target.locale];
  const receipts = [];
  for (let offset = 0; offset < expected.length; offset += 12) {
    const packet = expected.slice(offset, offset + 12);
    const actual = await page.evaluate(
      async ({ packet, headings }) =>
        Promise.all(
          packet.map(async ({ route }) => {
            const response = await fetch(`${route}/index.html`);
            const document = new DOMParser().parseFromString(await response.text(), "text/html");
            const h2 = [...document.querySelectorAll(".content h2")].map((heading) =>
              heading.textContent?.trim(),
            );
            return {
              status: response.status,
              id: document.querySelector("h1")?.textContent?.trim(),
              bad: h2.includes(headings.bad),
              good: h2.includes(headings.good),
              code: [...document.querySelectorAll(".content pre code")]
                .filter(
                  (element) =>
                    !element.closest(".vize-command-tabs") ||
                    element.hasAttribute("data-command-tabs-original"),
                )
                .map((element) => (element.textContent ?? "").replace(/\n$/, "")),
            };
          }),
        ),
      { packet, headings },
    );
    for (const [index, authored] of packet.entries()) {
      const page_ = actual[index];
      assert.equal(page_.status, 200, authored.route);
      assert.equal(page_.id, authored.id, authored.route);
      assert.equal(page_.bad, true, `${authored.route}: Bad heading`);
      assert.equal(page_.good, true, `${authored.route}: Good heading`);
      assert.deepEqual(page_.code, authored.code, `${authored.route}: whole authored code`);
      receipts.push({
        route: authored.route,
        sourceSha256: authored.sourceSha256,
        wholeCodeSha256: createHash("sha256").update(JSON.stringify(page_.code)).digest("hex"),
        codeBlocks: page_.code.length,
      });
    }
  }
  return {
    locale: target.locale,
    pages: receipts.length,
    receipts,
    indentation: await verifyRenderedRuleIndentation(page, prefix(target.locale), headings),
    japaneseEditorial: target.locale === "ja" ? await verifyJapaneseEditorial(page) : undefined,
  };
}
