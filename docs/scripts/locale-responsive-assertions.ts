import assert from "node:assert/strict";
import type { Page } from "playwright";
import { measureLocaleDropdown } from "./locale-render-assertions.ts";

/** The same live disclosure crosses the site's existing mobile breakpoint. */
export async function verifyResponsiveLocaleDropdown(page: Page) {
  const originalViewport = page.viewportSize();
  assert(originalViewport, "responsive check has an explicit viewport");
  const mounted = await page.locator(".docs-locale").elementHandle();
  assert(mounted, "responsive check uses the existing control");
  const button = page.locator(".docs-locale-select");
  const panel = page.locator(".docs-locale-options");
  await button.press("ArrowDown");
  await page.keyboard.press("ArrowDown");
  const receipts = [];
  for (const width of [1440, 390, 769, 360, 768, 320, 1440]) {
    await page.setViewportSize({ width, height: originalViewport.height });
    await page.waitForFunction(() => {
      const control = document.querySelector(".docs-locale");
      return control?.parentElement?.classList.contains(
        innerWidth <= 768 ? "header" : "header-actions",
      );
    });
    assert.equal(
      await mounted.evaluate((element) => element === document.querySelector(".docs-locale")),
      true,
      "resize reuses the exact mount",
    );
    assert.equal(await page.locator(".docs-locale").count(), 1);
    assert.equal(await panel.isVisible(), true, "resize preserves the open disclosure");
    const focused = await page.evaluate(
      () => (document.activeElement as HTMLElement | null)?.dataset.locale ?? null,
    );
    assert.equal(focused, "ja", "resize preserves the focused language link");
    receipts.push({ width, focused, metrics: await measureLocaleDropdown(page) });
  }
  await page.setViewportSize({ width: 390, height: originalViewport.height });
  await page.waitForFunction(() =>
    document.querySelector(".docs-locale")?.parentElement?.classList.contains("header"),
  );
  await page.evaluate(() => {
    const navigation = (
      globalThis as typeof globalThis & {
        __vizeDocsNavigation?: { initialize: (root: Document) => void };
      }
    ).__vizeDocsNavigation;
    if (!navigation) throw new Error("The shipped initializer is missing");
    navigation.initialize(document);
    navigation.initialize(document);
  });
  assert.equal(
    await mounted.evaluate((element) => element === document.querySelector(".docs-locale")),
    true,
    "mobile initialization reuses the direct-header mount",
  );
  assert.equal(await panel.isVisible(), false, "initialization closes the prior popup");
  assert.equal(await page.locator(".docs-locale").count(), 1);
  await page.setViewportSize(originalViewport);
  await mounted.dispose();
  return { receipts, repeatedMobile: { controls: 1, sameMount: true, open: false } };
}
