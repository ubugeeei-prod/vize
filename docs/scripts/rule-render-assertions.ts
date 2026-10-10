import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "playwright";
import { verifyRenderedVueRulePackets } from "./vue-rule-render-assertions.ts";
import { verifyRenderedCategoryPackets } from "./category-rule-render-assertions.ts";
import { categoryFiles } from "./rules/catalogue-routes.ts";
import { catalogueSubgroups } from "./rules/catalogue-subgroups.ts";

export const ruleRenderRoutes = [
  "/rules/all",
  "/rules/vue",
  ...categoryFiles.map((file) => `/rules/${file}`),
  "/rules/cross-file",
  "/rules/reference/script-define-props-destructuring",
  "/rules/reference/a11y-form-control-has-label",
  "/rules/project/vize-croquis-cf-array-mutation",
  "/rules/project/vize-croquis-cf-circular-reactive-dependency",
];
const root = resolve(import.meta.dirname, "../..");

export async function verifyRenderedRulePackets(page: Page, route: string) {
  if (/^\/(?:ja\/|zh-CN\/|pt-BR\/|fr\/)?rules\/vue$/.test(route))
    return verifyRenderedVueRulePackets(page, route);
  const category = route.match(/^\/(ja\/|zh-CN\/|pt-BR\/|fr\/)?rules\/([^/]+)$/);
  if (
    category &&
    (categoryFiles.includes(category[2] as (typeof categoryFiles)[number]) ||
      category[2] === "cross-file" ||
      (category[1] && category[1] !== "ja/" && category[2] === "all") ||
      (!category[1] && category[2] in catalogueSubgroups))
  )
    return verifyRenderedCategoryPackets(page, route);
  if (!/^\/(?:ja\/)?rules\/all$/.test(route)) return null;
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
        assert.match(source, /Example qualification: `illustrative-source-pair`/);
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
  assert.equal(expected.length, 317, "251 source rules plus 66 project entries per locale");
  const inline = await page.evaluate(
    (expected) => {
      const ids = new Map(
        expected.map(({ id }) => [id.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase(), id]),
      );
      const packets: Record<string, { code: string[]; bad: boolean; good: boolean }> = {};
      let current: string | undefined;
      const content = document.querySelector(".content");
      if (!content) throw new Error("Inline rule catalogue requires its content authority");
      const contentIds = [...content.querySelectorAll("[id]")].map((element) => element.id);
      if (new Set(contentIds).size !== contentIds.length)
        throw new Error("Inline rule catalogue contains duplicate IDs");
      for (const element of content.querySelectorAll("h3[id], pre code")) {
        const id = ids.get(element.id);
        if (id !== undefined) {
          current = id;
          if (packets[current]) throw new Error(`Duplicate inline rule ${current}`);
          packets[current] = {
            code: [],
            bad: Boolean(document.getElementById(`${element.id}-bad`)),
            good: Boolean(document.getElementById(`${element.id}-good`)),
          };
        } else if (element.parentElement?.tagName === "PRE" && current) {
          if (
            element.closest(".vize-command-tabs") &&
            !element.hasAttribute("data-command-tabs-original")
          )
            continue;
          const text = element.textContent;
          if (text === null) throw new Error(`Missing inline rule code ${current}`);
          packets[current].code.push(text.replace(/\n$/, ""));
        }
      }
      return packets;
    },
    expected.map(({ id }) => ({ id })),
  );
  assert.equal(Object.keys(inline).length, expected.length, "every rule is on the same page");
  for (const authored of expected) {
    assert.equal(inline[authored.id]?.bad, true, `${authored.id}: inline Bad anchor`);
    assert.equal(inline[authored.id]?.good, true, `${authored.id}: inline Good anchor`);
    assert.deepEqual(inline[authored.id]?.code, authored.code, `${authored.id}: whole inline code`);
  }
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
  return { locale: locale || "en", pages: receipts.length, receipts };
}
