import { test, expect, type Page } from "@playwright/test";

const TABS = ["atelier", "patina", "glyph", "canon", "croquis", "cross-file", "musea"] as const;

/**
 * Selectors that confirm the tab's output has fully rendered.
 * Each targets an element that only appears after WASM processing
 * completes and the result is displayed.
 */
const OUTPUT_READY: Record<(typeof TABS)[number], string> = {
  atelier: ".compile-time",
  patina: ".perf-badge",
  glyph: ".perf-badge",
  canon: ".perf-badge",
  croquis: ".perf-badge",
  "cross-file": ".status-time",
  musea: ".perf-badge",
};

async function waitForReady(page: Page, tab: (typeof TABS)[number]) {
  await page.waitForFunction(
    () => document.querySelector(".wasm-status")?.textContent?.includes("WASM"),
    { timeout: 15_000 },
  );

  const selector = OUTPUT_READY[tab];
  try {
    await page.waitForSelector(selector, { timeout: 10_000 });
  } catch {
    // Fallback for tabs whose output may vary
  }

  // Settle time for code highlighting and animations
  await page.waitForTimeout(1500);
}

for (const tab of TABS) {
  test.describe(tab, () => {
    test(`light`, async ({ page }) => {
      await page.goto(`/?tab=${tab}`);
      await waitForReady(page, tab);
      // GitHub Actions on Linux is the canonical baseline for these snapshots.
      await expect(page).toHaveScreenshot(`${tab}-light.png`);
    });

    test(`dark`, async ({ page }) => {
      await page.goto(`/?tab=${tab}`);
      await waitForReady(page, tab);
      await page.getByRole("button", { name: "Dark mode" }).click();
      await page.waitForTimeout(300);
      await expect(page).toHaveScreenshot(`${tab}-dark.png`);
    });
  });
}
