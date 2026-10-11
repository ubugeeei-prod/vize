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
  return codes;
}
