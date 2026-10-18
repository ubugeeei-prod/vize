import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "playwright";
import { verifyRenderedRuleIndentation } from "./rule-indentation-render-assertions.ts";
import {
  expectedVueRulePackets,
  verifyRenderedVueRulePackets,
} from "./vue-rule-render-assertions.ts";
import {
  expectedCategoryPackets,
  verifyRenderedCategoryPackets,
} from "./category-rule-render-assertions.ts";
import { categoryFiles } from "./rules/catalogue-routes.ts";
import { catalogueSubgroups } from "./rules/catalogue-subgroups.ts";
import { verifyJapaneseRuleEditorial } from "./ja-rule-editorial-render-assertions.ts";
import {
  expectedRuleOverview,
  verifyRenderedRuleOverview,
} from "./all-rules-overview-render-assertions.ts";

export const ruleRenderRoutes = [
  "/rules/all",
  "/rules/vue",
  ...categoryFiles.map((file) => `/rules/${file}`),
  "/rules/cross-file",
  "/rules/reference/script-define-props-destructuring",
  "/rules/reference/a11y-form-control-has-label",
  "/rules/reference/vue-no-preprocessor-lang",
  "/rules/project/vize-croquis-cf-array-mutation",
  "/rules/project/vize-croquis-cf-circular-reactive-dependency",
];
const root = resolve(import.meta.dirname, "../..");

type RuleRouteKind = "vue" | "category" | "overview" | "references";

export function ruleRouteKind(route: string): RuleRouteKind | null {
  if (/^\/(?:ja\/|zh-CN\/|pt-BR\/|fr\/)?rules\/vue$/.test(route)) return "vue";
  const category = route.match(/^\/(ja\/|zh-CN\/|pt-BR\/|fr\/)?rules\/([^/]+)$/);
  if (
    category &&
    (categoryFiles.includes(category[2] as (typeof categoryFiles)[number]) ||
      category[2] === "cross-file" ||
      (category[2] in catalogueSubgroups &&
        (!category[1] || (category[1] === "ja/" && catalogueSubgroups[category[2]].jaTitle))))
  )
    return "category";
  if (!/^\/(?:ja\/|zh-CN\/|pt-BR\/|fr\/)?rules\/all$/.test(route)) return null;
  return /^\/(?:ja\/)?rules\/all$/.test(route) ? "references" : "overview";
}

export function expectedRuleReferences(route: string) {
  const ja = route.startsWith("/ja/");
  const locale = ja ? "ja/" : "";
  const expected = ["reference", "project"].flatMap((section) =>
    readdirSync(resolve(root, `docs/content/${locale}rules/${section}`)).map((file) => {
      const source = readFileSync(
        resolve(root, `docs/content/${locale}rules/${section}/${file}`),
        "utf8",
      );
      for (const heading of ja ? ["悪い", "良い"] : ["Bad", "Good"]) {
        const example = source.split(`## ${heading}\n`)[1]?.split("\n## ")[0];
        assert.match(example ?? "", /```(?:vue|ts|html)\n/, `${file}: actual ${heading} source`);
      }
      const graphs = [...source.matchAll(/```text\n([\s\S]*?)\n```/g)];
      if (file === "vize-croquis-cf-circular-reactive-dependency.md") {
        assert.equal(graphs.length, 2, "both retained Bad/Good graph packets are rendered");
        assert.match(
          source,
          ja
            ? /例の検証状況: `illustrative-source-pair`/
            : /Example qualification: `illustrative-source-pair`/,
        );
      }
      const id = source.match(/^# `([^`]+)`/m)?.[1];
      assert.ok(id, `${file}: authored rule heading`);
      return {
        route: `/${locale}rules/${section}/${file.replace(/\.md$/, "")}`,
        id,
        code: [...source.matchAll(/```\w+\n([\s\S]*?)\n```/g)].map((match) => match[1]),
        sourceSha256: createHash("sha256").update(source).digest("hex"),
        graphBlocks: graphs.length,
      };
    }),
  );
  assert.equal(expected.length, 321, "255 source rules plus 66 project entries per locale");
  return { ja, locale, expected };
}

/** The review's source-side rule pins, for the merge queue's browser-free run. */
export function verifyRuleSources(route: string) {
  const kind = ruleRouteKind(route);
  if (kind === "vue") expectedVueRulePackets(route);
  if (kind === "category") expectedCategoryPackets(route);
  if (kind === "overview" || kind === "references") expectedRuleOverview(route);
  if (kind === "references") expectedRuleReferences(route);
}

export async function verifyRenderedRulePackets(page: Page, route: string) {
  const japaneseEditorial = await verifyJapaneseRuleEditorial(page, route);
  const kind = ruleRouteKind(route);
  if (kind === "vue")
    return { ...(await verifyRenderedVueRulePackets(page, route)), japaneseEditorial };
  if (kind === "category")
    return { ...(await verifyRenderedCategoryPackets(page, route)), japaneseEditorial };
  if (!kind) return null;
  const overview = await verifyRenderedRuleOverview(page, route);
  if (kind === "overview") return { overview, japaneseEditorial };
  const { ja, locale, expected } = expectedRuleReferences(route);
  const indentation = await verifyRenderedRuleIndentation(page, locale);
  const receipts = [];
  for (let offset = 0; offset < expected.length; offset += 12) {
    const packet = expected.slice(offset, offset + 12);
    const actual = await page.evaluate(
      async ({ packet, ja }) => {
        return Promise.all(
          packet.map(async ({ route }) => {
            const response = await fetch(`${route}/index.html`);
            const document = new DOMParser().parseFromString(await response.text(), "text/html");
            return {
              route,
              status: response.status,
              id: document.querySelector("h1")?.textContent?.trim(),
              bad: Boolean(document.getElementById(ja ? "悪い" : "bad")),
              good: Boolean(document.getElementById(ja ? "良い" : "good")),
              code: [...document.querySelectorAll(".content pre code")].map((element) => {
                const text = element.textContent;
                if (text === null) throw new Error(`Missing rendered rule code ${route}`);
                return text.replace(/\n$/, "");
              }),
            };
          }),
        );
      },
      { packet, ja },
    );
    for (let index = 0; index < packet.length; index += 1) {
      const authored = packet[index];
      const rendered = actual[index];
      assert.equal(rendered.status, 200, authored.route);
      assert.equal(rendered.id, authored.id, authored.route);
      assert.equal(rendered.bad, true, `${authored.route}: Bad anchor`);
      assert.equal(rendered.good, true, `${authored.route}: Good anchor`);
      assert.deepEqual(
        rendered.code,
        authored.code,
        `${authored.route}: whole authored code packets`,
      );
      receipts.push({
        route: authored.route,
        sourceSha256: authored.sourceSha256,
        wholeCodeSha256: createHash("sha256").update(JSON.stringify(rendered.code)).digest("hex"),
        codeBlocks: rendered.code.length,
        graphBlocks: authored.graphBlocks,
      });
    }
  }
  return {
    locale: locale || "en",
    pages: receipts.length,
    receipts,
    indentation,
    japaneseEditorial,
    overview,
  };
}
