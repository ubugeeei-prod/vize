import { test, expect, type Page } from "@playwright/test";

async function openCrossFile(page: Page) {
  await page.goto("/?tab=cross-file");
  await page.waitForFunction(
    () => document.querySelector(".wasm-status")?.textContent?.includes("WASM"),
    { timeout: 15_000 },
  );
  await page.waitForSelector(".cross-file-playground .status-time", { timeout: 15_000 });
}

test.describe("cross-file playground", () => {
  test("switches validation scope and filters diagnostics", async ({ page }) => {
    await openCrossFile(page);

    await page.locator(".profile-switch").getByRole("button", { name: "Validation" }).click();

    await expect(page.locator(".analysis-mode-badge")).toHaveText("VALIDATION");
    await expect(
      page.locator(".diagnostics-pane .issue-group .group-badge", {
        hasText: "Props Validation",
      }),
    ).toBeVisible({ timeout: 10_000 });

    await page.locator(".severity-filters").getByRole("button", { name: "Errors" }).click();
    await expect(page.locator(".diagnostics-empty")).toContainText("No matching diagnostics");

    await page.locator(".diagnostics-stats").getByRole("button", { name: /all$/ }).click();
    await expect(page.locator(".diagnostics-pane .issue-card.warning").first()).toBeVisible();
  });

  test("surfaces setup context diagnostics from the preset library", async ({ page }) => {
    await openCrossFile(page);

    await page.locator(".preset-item", { hasText: "Setup Context" }).click();
    await page.locator(".profile-switch").getByRole("button", { name: "Validation" }).click();

    await expect(
      page.locator(".diagnostics-pane .issue-group .group-badge", {
        hasText: "Setup Context",
      }),
    ).toBeVisible({ timeout: 10_000 });
  });

  test("shows provider locations for inject reactivity loss", async ({ page }) => {
    await openCrossFile(page);

    await page.locator(".preset-item", { hasText: "Provide/Inject Tree" }).click();
    await page.locator(".profile-switch").getByRole("button", { name: "Signals" }).click();

    const issue = page.locator(".issue-card", {
      hasText: "Destructuring inject('legacyData')",
    });
    await expect(issue).toBeVisible({ timeout: 10_000 });
    await expect(issue.locator(".issue-related")).toContainText("App.vue");
    await expect(issue.locator(".related-msg")).toContainText("provide('legacyData') source");
  });
});
