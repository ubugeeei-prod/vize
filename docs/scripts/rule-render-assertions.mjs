import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

export const ruleRenderRoutes = [
  "/rules/all",
  "/rules/vue",
  "/rules/petite-vue",
  "/rules/ecosystem",
  "/rules/reference/script-define-props-destructuring",
  "/rules/reference/a11y-form-control-has-label",
  "/rules/project/vize-croquis-cf-array-mutation",
  "/rules/project/vize-croquis-cf-circular-reactive-dependency",
];
const root = resolve(import.meta.dirname, "../..");

export async function verifyRenderedRulePackets(page, route) {
  if (!/^\/(?:ja\/)?rules\/all$/.test(route)) return null;
  const ja = route.startsWith("/ja/");
  const locale = ja ? "ja/" : "";
  const expected = ["reference", "project"].flatMap((section) =>
    readdirSync(resolve(root, `docs/content/${locale}rules/${section}`)).map((file) => {
      const source = readFileSync(
        resolve(root, `docs/content/${locale}rules/${section}/${file}`),
        "utf8",
      );
      return {
        route: `/${locale}rules/${section}/${file.replace(/\.md$/, "")}`,
        id: source.match(/^# `([^`]+)`/m)?.[1],
        code: [...source.matchAll(/```\w+\n([\s\S]*?)\n```/g)].map((match) => match[1]),
        sourceSha256: createHash("sha256").update(source).digest("hex"),
      };
    }),
  );
  assert.equal(expected.length, 317, "251 source rules plus 66 project entries per locale");
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
              id: document.querySelector("h1")?.textContent.trim(),
              bad: Boolean(document.getElementById(ja ? "悪い" : "bad")),
              good: Boolean(document.getElementById(ja ? "良い" : "good")),
              code: [...document.querySelectorAll(".content pre code")].map((element) =>
                element.textContent.replace(/\n$/, ""),
              ),
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
      });
    }
  }
  return { locale: locale || "en", pages: receipts.length, receipts };
}
