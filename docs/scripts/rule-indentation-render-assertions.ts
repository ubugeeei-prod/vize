import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "playwright";
import { ruleIndentationExamples } from "./rules/indentation-examples.ts";

export async function verifyRenderedRuleIndentation(page: Page, locale: string) {
  const root = resolve(import.meta.dirname, "../..");
  const inputs = ruleIndentationExamples.map((fixture) => {
    const source = readFileSync(resolve(root, fixture.sourcePath), "utf8");
    assert.equal(createHash("sha256").update(source).digest("hex"), fixture.sourceSha256);
    return {
      ...fixture,
      source,
      route: `/${locale}rules/reference/${fixture.name.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase()}`,
    };
  });
  const rendered = await page.evaluate(
    async ({ routes, ja }) => {
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
          const bad = document.getElementById(ja ? "悪い" : "bad");
          const good = document.getElementById(ja ? "良い" : "good");
          const follows = (first: Element | null, second: Element | undefined) =>
            Boolean(first && second && (first.compareDocumentPosition(second) & 4) !== 0);
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
            badBeforeCode: follows(bad, blocks[2]),
            badCodeBeforeGood: follows(blocks[2] ?? null, good ?? undefined),
            goodBeforeCode: follows(good, blocks[3]),
          };
        }),
      );
    },
    { routes: inputs.map((input) => input.route), ja: locale === "ja/" },
  );
  for (let index = 0; index < inputs.length; index += 1) {
    const input = inputs[index];
    const actual = rendered[index];
    assert.equal(actual.route, input.route);
    assert.equal(actual.status, 200, input.route);
    assert.equal(actual.id, input.name);
    assert.equal(actual.code.length, 4, `${input.route}: all four original code blocks`);
    assert.equal(actual.badBeforeCode, true, `${input.route}: Bad/code order`);
    assert.equal(actual.badCodeBeforeGood, true, `${input.route}: Bad/Good order`);
    assert.equal(actual.goodBeforeCode, true, `${input.route}: Good/code order`);
    assert.deepEqual(
      actual.code.slice(2),
      [input.expected.bad.source, input.expected.good.source],
      `${input.route}: whole original-source indentation`,
    );
  }
  // Retain raw original carriers, whole expected/actual code and fetched HTML.
  // The source-bound Docs receipt carries these alongside existing geometry.
  return { inputs, rendered };
}
