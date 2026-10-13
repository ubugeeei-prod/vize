import assert from "node:assert/strict";
import type { Page } from "playwright";

export async function verifyTableCodeReadability(page: Page, route: string) {
  const codes = await page.evaluate(() => {
    const bounds = (rect: DOMRect) => ({
      left: rect.left,
      right: rect.right,
      top: rect.top,
      bottom: rect.bottom,
    });
    return [...document.querySelectorAll(".content table :is(th, td) code")]
      .filter((code) => code.checkVisibility() && !code.closest("pre"))
      .map((code) => {
        const cell = code.closest("th, td");
        if (!cell) throw new Error("Table code has no owning cell");
        const walker = document.createTreeWalker(code, NodeFilter.SHOW_TEXT);
        const glyphs: ReturnType<typeof bounds>[] = [];
        for (let node = walker.nextNode(); node; node = walker.nextNode()) {
          if (!node.textContent?.trim()) continue;
          const range = document.createRange();
          range.selectNodeContents(node);
          glyphs.push(...[...range.getClientRects()].map(bounds));
        }
        return {
          text: code.textContent,
          cell: bounds(cell.getBoundingClientRect()),
          glyphs,
        };
      });
  });
  for (const { text, cell, glyphs } of codes) {
    assert(glyphs.length > 0, `${route}: inline table code has no rendered text: ${text}`);
    for (const glyph of glyphs) {
      assert(
        glyph.left >= cell.left - 1 &&
          glyph.right <= cell.right + 1 &&
          glyph.top >= cell.top - 1 &&
          glyph.bottom <= cell.bottom + 1,
        `${route}: table code overlaps another cell: ${text}`,
      );
    }
  }
  if (
    route === "/guide/experimentals" ||
    route === "/ja/guide/experimentals" ||
    route === "/guide/experimentals-vue-rfcs" ||
    route === "/ja/guide/experimentals-vue-rfcs"
  ) {
    const japanese = route.startsWith("/ja/");
    const reference = route.endsWith("/experimentals-vue-rfcs");
    const label = japanese ? "または" : "Or";
    const limitation = reference
      ? japanese
        ? "選択肢を左から右に試す"
        : "tries alternatives from left to right"
      : japanese
        ? "左から右に調べます。それぞれの選択肢の中で変数を束縛する構文は未対応です。"
        : "Alternatives match left to right; bindings inside alternatives are not supported yet";
    const rows = await page
      .locator(".content table tr")
      .evaluateAll(
        (elements, label) =>
          elements
            .filter((row) => row.querySelector("td")?.textContent?.trim() === label)
            .map((row) => [...row.querySelectorAll("td")].map((cell) => cell.textContent?.trim())),
        label,
      );
    assert.deepEqual(
      rows,
      [[label, `v-when="'idle' | 'loading'"`, limitation]],
      `${route}: the complete or-pattern and its limitation must occupy their own native cells`,
    );
  }
  return codes;
}
