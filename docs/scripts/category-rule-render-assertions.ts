import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "playwright";

const root = resolve(import.meta.dirname, "../..");
export async function verifyRenderedCategoryPackets(page: Page, route: string) {
  const [, prefix = "", file] = route.match(/^\/(ja\/|zh-CN\/|pt-BR\/|fr\/)?rules\/([^/]+)$/)!;
  const locale = prefix.slice(0, -1) || "en";
  const source = readFileSync(
    resolve(root, `docs/content/generated/rules/${locale}/${file}.md`),
    "utf8",
  );
  const names = [...source.matchAll(/^### `([^`]+)`$/gm)].map((match) => match[1]);
  const authorities = new Map(
    ["reference", "project"].flatMap((section) =>
      readdirSync(
        resolve(root, `docs/content/${locale === "ja" ? "ja/" : ""}rules/${section}`),
      ).map((file) => {
        const source = readFileSync(
          resolve(root, `docs/content/${locale === "ja" ? "ja/" : ""}rules/${section}/${file}`),
          "utf8",
        );
        const id = source.match(/^# `([^`]+)`/m)?.[1];
        assert(id);
        return [id, source] as const;
      }),
    ),
  );
  const expected = names.map((name) => {
    const authored = authorities.get(name);
    assert(authored, `${route}: actual current source ${name}`);
    return {
      id: name.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase(),
      name,
      code: [...authored.matchAll(/```\w+\n([\s\S]*?)\n```/g)].map((match) => match[1]),
      sourceSha256: createHash("sha256").update(authored).digest("hex"),
      graphBlocks: [...authored.matchAll(/```text\n/g)].length,
    };
  });
  const actual = await page.evaluate(
    (expected) => {
      const names = new Set(expected.map(({ id }) => id));
      const content = document.querySelector(".content");
      if (!content) throw new Error("Category requires actual rendered content");
      const ids = [...content.querySelectorAll("[id]")].map((element) => element.id);
      if (new Set(ids).size !== ids.length) throw new Error("Category contains duplicate IDs");
      const packets: Record<string, { code: string[]; bad: boolean; good: boolean }> = {};
      let current: string | undefined;
      for (const element of content.querySelectorAll("h3[id], pre code")) {
        if (names.has(element.id)) {
          current = element.id;
          if (packets[current]) throw new Error(`Duplicate category packet ${current}`);
          packets[current] = {
            code: [],
            bad: Boolean(document.getElementById(`${current}-bad`)),
            good: Boolean(document.getElementById(`${current}-good`)),
          };
        } else if (element.parentElement?.tagName === "PRE" && current) {
          const text = element.textContent;
          if (text === null) throw new Error(`Missing complete category source ${current}`);
          packets[current].code.push(text.replace(/\n$/, ""));
        }
      }
      let examples = 0;
      for (const link of content.querySelectorAll("a[href]")) {
        if (!(link instanceof HTMLAnchorElement))
          throw new Error("Category link requires an actual anchor");
        if (!/#[^#]+-(?:bad|good)$/.test(link.getAttribute("href") ?? "")) continue;
        examples += 1;
        const target = new URL(link.href);
        if (target.origin !== location.origin || target.pathname !== location.pathname)
          throw new Error(`Category examples leave this page: ${link.href}`);
        if (!document.getElementById(decodeURIComponent(target.hash.slice(1))))
          throw new Error(`Missing local example ${link.href}`);
      }
      return {
        packets,
        examples,
        added: content.querySelectorAll(".ox-code-line--add").length,
        removed: content.querySelectorAll(".ox-code-line--remove").length,
      };
    },
    expected.map(({ id }) => ({ id })),
  );
  assert.equal(Object.keys(actual.packets).length, expected.length);
  assert(actual.examples >= expected.length * 2, `${route}: complete same-page example links`);
  assert(actual.added > 0 && actual.removed > 0, `${route}: native diff annotations`);
  const receipts = expected.map((authored) => {
    const packet = actual.packets[authored.id];
    assert.equal(packet.bad, true, `${route} ${authored.name}: Bad anchor`);
    assert.equal(packet.good, true, `${route} ${authored.name}: Good anchor`);
    assert.deepEqual(
      packet.code,
      authored.code,
      `${route} ${authored.name}: complete copied source`,
    );
    return {
      route,
      id: authored.name,
      sourceSha256: authored.sourceSha256,
      wholeCodeSha256: createHash("sha256").update(JSON.stringify(packet.code)).digest("hex"),
      codeBlocks: packet.code.length,
      graphBlocks: authored.graphBlocks,
    };
  });
  return { locale, pages: receipts.length, receipts };
}
