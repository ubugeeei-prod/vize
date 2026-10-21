import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "playwright";

interface EditorialCase {
  case: string;
  route: string;
  rule: string;
  section: "purpose" | "bad" | "good";
  before: string;
  after: string;
}
const fixture = JSON.parse(
  readFileSync(
    resolve(import.meta.dirname, "../../tests/_fixtures/docs-ja-rule-editorial.json"),
    "utf8",
  ),
) as { originalSourceSha: string; cases: EditorialCase[] };

/** Compare complete authored paragraphs after native Markdown rendering. */
export function editorialText(markdown: string) {
  return markdown
    .replace(/`([^`]+)`/g, "$1")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replace(/\s+/g, " ")
    .trim();
}

export function assertEditorialParagraph(actual: string, expected: EditorialCase) {
  assert.equal(
    actual,
    editorialText(expected.after),
    `${expected.rule}: whole ${expected.section}`,
  );
  assert.notEqual(actual, editorialText(expected.before), `${expected.rule}: original bad wording`);
}

export async function readEditorialParagraphs(page: Page, route: string, standaloneRule?: string) {
  const cases = fixture.cases.filter(
    (entry) => entry.route === route && (!standaloneRule || entry.rule === standaloneRule),
  );
  if (!cases.length) return [];
  const actual = await page.evaluate(
    ({ cases, standalone }) => {
      return cases.map((entry) => {
        const slug = entry.rule.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
        const target = entry.section === "purpose" ? slug : `${slug}-${entry.section}`;
        const anchor = standalone
          ? entry.section === "purpose"
            ? document.querySelector(".content h1")
            : document.getElementById(entry.section === "bad" ? "悪い" : "良い")
          : document.getElementById(target);
        if (!anchor) throw new Error(`${entry.rule}: missing original ${entry.section} anchor`);
        let next = (anchor.closest("p") ?? anchor).nextElementSibling;
        while (next && !/^(?:PRE|H[1-6])$/.test(next.tagName)) {
          if (next instanceof HTMLParagraphElement) {
            const text = next.textContent?.replace(/\s+/g, " ").trim();
            if (text && !/^(?:悪い|良い)$/.test(text)) {
              if (!next.checkVisibility() || next.getBoundingClientRect().height <= 0)
                throw new Error(`${entry.rule}: explanation paragraph is not rendered`);
              return { case: entry.case, text };
            }
          }
          next = next.nextElementSibling;
        }
        throw new Error(`${entry.rule}: missing complete ${entry.section} explanation`);
      });
    },
    { cases, standalone: Boolean(standaloneRule) },
  );
  assert.equal(actual.length, cases.length, `${route}: every authored editorial case`);
  return actual.map((entry, index) => ({ ...entry, expected: cases[index] }));
}

export async function verifyJapaneseRuleEditorial(page: Page, route: string) {
  const standalone = fixture.cases.find((entry) => {
    const section = entry.rule.startsWith("vize:") ? "project" : "reference";
    const slug = entry.rule.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase();
    return route === `/ja/rules/${section}/${slug}`;
  });
  const paragraphs = await readEditorialParagraphs(
    page,
    standalone?.route ?? route,
    standalone?.rule,
  );
  for (const { text, expected } of paragraphs) assertEditorialParagraph(text, expected);
  if (route === "/ja/rules/all") {
    assert.equal(
      await page.locator('[id="プロジェクト-ルールと-analyzer-契約-66"]').count(),
      1,
      "the original native project chapter bookmark remains unique",
    );
  }
  if (route === "/ja/rules/reference/vue-no-preprocessor-lang") {
    const configurationHeading = page.locator("h2").filter({ hasText: "設定（Vite+）" });
    assert.equal(
      await configurationHeading.count(),
      1,
      "the supported configuration remains visible",
    );
    const unsupportedHeading = page
      .locator("h2")
      .filter({ hasText: "現在の SFC lint では診断されません" });
    assert.equal(await unsupportedHeading.count(), 0, "the obsolete unsupported note stays absent");
    assert.equal(await page.locator('[id="設定できる-id-現在の-sfc-検出なし"]').count(), 0);
  }
  return {
    originalSourceSha: fixture.originalSourceSha,
    paragraphs: paragraphs.map(({ case: id, text }) => ({
      case: id,
      wholeTextSha256: createHash("sha256").update(text).digest("hex"),
    })),
  };
}
