import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import type { Page } from "playwright";

const root = resolve(import.meta.dirname, "../..");

export async function verifyRenderedVueRulePackets(page: Page, route: string) {
  const locale = route.match(/^\/(ja|zh-CN|pt-BR|fr)\//)?.[1] ?? "en";
  const directory = resolve(root, `docs/content/${locale === "ja" ? "ja/" : ""}rules/reference`);
  const expected = readdirSync(directory)
    .filter((file) => file.startsWith("vue-"))
    .map((file) => {
      const source = readFileSync(resolve(directory, file), "utf8");
      const id = source.match(/^# `([^`]+)`/m)?.[1];
      assert(id?.startsWith("vue/"), `${file}: actual Vue rule`);
      return {
        id: file.replace(/\.md$/, ""),
        name: id,
        code: [...source.matchAll(/```\w+\n([\s\S]*?)\n```/g)].map((match) => match[1]),
        sourceSha256: createHash("sha256").update(source).digest("hex"),
      };
    });
  assert.equal(expected.length, 104, "every implemented Vue rule");
  const actual = await page.evaluate(
    (expected) => {
      const names = new Set(expected.map(({ id }) => id));
      const packets: Record<string, { bad: boolean; good: boolean; code: string[] }> = {};
      let current: string | undefined;
      const content = document.querySelector(".content");
      if (!content) throw new Error("Vue category requires its actual content authority");
      const ids = [...content.querySelectorAll("[id]")].map((element) => element.id);
      if (new Set(ids).size !== ids.length) throw new Error("Vue category contains duplicate IDs");
      for (const element of content.querySelectorAll("h3[id], pre code")) {
        if (names.has(element.id)) {
          current = element.id;
          if (packets[current]) throw new Error(`Duplicate Vue packet ${current}`);
          packets[current] = {
            bad: Boolean(document.getElementById(`${current}-bad`)),
            good: Boolean(document.getElementById(`${current}-good`)),
            code: [],
          };
        } else if (element.parentElement?.tagName === "PRE" && current) {
          const text = element.textContent;
          if (text === null) throw new Error(`Missing complete Vue source ${current}`);
          packets[current].code.push(text.replace(/\n$/, ""));
        }
      }
      const localExamples = [...content.querySelectorAll("a[href]")].filter((link) =>
        /#vue-.+-(?:bad|good)$/.test(link.getAttribute("href") ?? ""),
      );
      for (const link of localExamples) {
        if (!(link instanceof HTMLAnchorElement))
          throw new Error("Vue example link is not an anchor");
        const target = new URL(link.href);
        if (target.pathname !== location.pathname || target.origin !== location.origin)
          throw new Error(`Vue examples leave the current page: ${link.href}`);
        if (!document.getElementById(decodeURIComponent(target.hash.slice(1))))
          throw new Error(`Missing local Vue example: ${link.href}`);
      }
      return {
        packets,
        localExampleLinks: localExamples.length,
        addedLines: content.querySelectorAll(".ox-code-line--add").length,
        removedLines: content.querySelectorAll(".ox-code-line--remove").length,
      };
    },
    expected.map(({ id }) => ({ id })),
  );
  assert.equal(Object.keys(actual.packets).length, expected.length);
  assert(actual.localExampleLinks >= expected.length * 2, `${locale}: same-page Bad/Good links`);
  assert(actual.addedLines > 0 && actual.removedLines > 0, `${locale}: native diff annotations`);
  const receipts = expected.map((authored) => {
    const packet = actual.packets[authored.id];
    assert.equal(packet.bad, true, `${locale} ${authored.name}: Bad anchor`);
    assert.equal(packet.good, true, `${locale} ${authored.name}: Good anchor`);
    assert.deepEqual(
      packet.code,
      authored.code,
      `${locale} ${authored.name}: complete copied source`,
    );
    return {
      route,
      id: authored.name,
      sourceSha256: authored.sourceSha256,
      wholeCodeSha256: createHash("sha256").update(JSON.stringify(packet.code)).digest("hex"),
      codeBlocks: packet.code.length,
      graphBlocks: 0,
    };
  });
  return { locale, pages: receipts.length, receipts };
}
