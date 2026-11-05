import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "playwright";
import { ruleIndentationExamples } from "./rules/indentation-examples.ts";

export async function verifyRenderedRuleIndentation(
  page: Page,
  locale: string,
  headings: { bad: string; good: string },
) {
  const root = resolve(import.meta.dirname, "../..");
  const inputs = ruleIndentationExamples.map((fixture) => {
    // The expected sources are compared with the live extraction by
    // docs-rule-example-indentation.test.ts; the rendered page must match them.
    const source = readFileSync(resolve(root, fixture.sourcePath), "utf8");
    return {
      ...fixture,
      source,
      route: `/${locale}rules/reference/${fixture.name.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase()}`,
    };
  });
  const rendered = await page.evaluate(
    async ({ routes, headings }) => {
      return Promise.all(
        routes.map(async (route) => {
          const response = await fetch(`${route}/index.html`);
          const html = await response.text();
          const document = new DOMParser().parseFromString(html, "text/html");
          const blocks = [...document.querySelectorAll(".content pre code")].filter(
            (element) =>
              !element.closest(".vize-command-tabs") ||
              element.hasAttribute("data-command-tabs-original"),
          );
          const h2 = [...document.querySelectorAll(".content h2")];
          const bad = h2.find((heading) => heading.textContent?.trim() === headings.bad) ?? null;
          const good = h2.find((heading) => heading.textContent?.trim() === headings.good) ?? null;
          const follows = (first: Element | null, second: Element | undefined) =>
            Boolean(first && second && (first.compareDocumentPosition(second) & 4) !== 0);
          // The executed examples are the first code blocks after Bad and Good.
          const after = (heading: Element | null) =>
            blocks.find((block) => follows(heading, block));
          const badCode = after(bad);
          const goodCode = after(good);
          return {
            route,
            status: response.status,
            html,
            id: document.querySelector("h1")?.textContent?.trim(),
            code: blocks.map((element) => {
              const text = element.textContent;
              if (text === null) throw new Error(`Missing whole code ${route}`);
              // Drop only the renderer's final line terminator, as the original
              // whole-code oracle does; authored indentation is never normalized.
              return text.replace(/\n$/, "");
            }),
            examples: [badCode, goodCode].map((block) => block?.textContent?.replace(/\n$/, "")),
            badBeforeCode: Boolean(badCode),
            badCodeBeforeGood: follows(badCode ?? null, good ?? undefined),
            goodBeforeCode: Boolean(goodCode),
          };
        }),
      );
    },
    { routes: inputs.map((input) => input.route), headings },
  );
  for (let index = 0; index < inputs.length; index += 1) {
    const input = inputs[index];
    const actual = rendered[index];
    assert.equal(actual.route, input.route);
    assert.equal(actual.status, 200, input.route);
    assert.equal(actual.id, input.name);
    assert.equal(actual.badBeforeCode, true, `${input.route}: Bad/code order`);
    assert.equal(actual.badCodeBeforeGood, true, `${input.route}: Bad/Good order`);
    assert.equal(actual.goodBeforeCode, true, `${input.route}: Good/code order`);
    assert.deepEqual(
      actual.examples,
      [input.expected.bad.source, input.expected.good.source],
      `${input.route}: whole original-source indentation`,
    );
  }
  // Retain raw original carriers, whole expected/actual code and fetched HTML.
  // The source-bound Docs receipt carries these alongside existing geometry.
  return { inputs, rendered };
}
