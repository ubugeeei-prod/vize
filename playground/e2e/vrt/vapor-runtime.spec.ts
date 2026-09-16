import { expect, test, type ConsoleMessage, type Page } from "@playwright/test";

const MAIN_TABS = [
  { key: "atelier", name: "Atelier", ready: ".compile-time" },
  { key: "patina", name: "Patina", ready: ".perf-badge" },
  { key: "glyph", name: "Glyph", ready: ".perf-badge" },
  { key: "canon", name: "Canon", ready: ".perf-badge" },
  { key: "croquis", name: "Croquis", ready: ".perf-badge" },
  { key: "cross-file", name: "Cross", ready: ".status-time" },
  { key: "musea", name: "Musea", ready: ".perf-badge" },
] as const;

type RuntimeIssue = {
  source: "console" | "pageerror";
  text: string;
};

function shouldIgnoreConsole(message: ConsoleMessage) {
  const text = message.text();
  return text.includes("[vite] connecting") || text.includes("[vite] connected");
}

function collectRuntimeIssues(page: Page) {
  const issues: RuntimeIssue[] = [];

  page.on("console", (message) => {
    if (!["error", "warning"].includes(message.type())) return;
    if (shouldIgnoreConsole(message)) return;
    issues.push({
      source: "console",
      text: message.text(),
    });
  });

  page.on("pageerror", (error) => {
    issues.push({
      source: "pageerror",
      text: String(error),
    });
  });

  return issues;
}

async function waitForWasm(page: Page) {
  await page.waitForFunction(
    () => document.querySelector(".wasm-status")?.textContent?.includes("WASM"),
    { timeout: 15_000 },
  );
}

async function waitForTabReady(page: Page, readySelector: string) {
  await waitForWasm(page);
  await page.waitForSelector(readySelector, { timeout: 10_000 });
  await page.waitForTimeout(800);
}

async function visitOutputTabs(page: Page) {
  const tabs = page.locator(".output-panel .tabs > button");
  const count = await tabs.count();

  for (let index = 0; index < count; index++) {
    await tabs.nth(index).click();
    await page.waitForTimeout(500);
  }
}

async function expectAtelierChrome(page: Page) {
  await expect(page.locator(".logo-icon svg g > path")).toHaveCount(2);

  const tabClasses = await page
    .locator(".output-panel .tabs > button")
    .evaluateAll((els) => els.map((el) => el.className));
  expect(tabClasses.length).toBeGreaterThan(0);
  expect(tabClasses.every((className) => className.includes("tab"))).toBe(true);
}

test("playground main tabs and output tabs stay runtime-clean in Vapor mode", async ({ page }) => {
  const issues = collectRuntimeIssues(page);

  await page.goto("/?tab=atelier");
  await waitForTabReady(page, ".compile-time");
  await expectAtelierChrome(page);

  for (const tab of MAIN_TABS) {
    await page.locator(".main-tabs > button", { hasText: tab.name }).click();
    await expect(page.locator(".main-tabs > button.active .tab-name")).toHaveText(tab.name);
    await waitForTabReady(page, tab.ready);

    if (tab.key === "atelier") {
      await expectAtelierChrome(page);
    }

    await visitOutputTabs(page);
  }

  expect(issues).toEqual([]);
});

test("experimental controls drive compiler, typechecker, and Croquis in Vapor mode", async ({
  page,
}) => {
  await page.goto("/?tab=atelier");
  await waitForTabReady(page, ".compile-time");
  await page.locator(".experimental-features summary").click();
  await page.getByLabel("Experimental example").selectOption("experimentalPatternedTemplate");
  await expect(page.locator(".output-panel")).toContainText("Ready");
  for (const target of ["SSR", "Vapor"]) {
    await page.getByRole("button", { name: target, exact: true }).click();
    await expect(page.locator(".code-header h4")).toHaveText(`${target} Output`);
    await expect(page.locator(".output-panel")).toContainText("Ready");
  }
  await page.getByLabel("Patterned templates", { exact: true }).uncheck();
  await expect(page.locator(".output-panel")).toContainText(
    "require `experimentals.patternedTemplate`",
  );
  await page.getByLabel("Patterned templates", { exact: true }).check();
  await expect(page.getByRole("button", { name: "Vapor", exact: true })).toBeVisible();

  await page.locator(".main-tabs > button", { hasText: "Canon" }).click();
  await waitForTabReady(page, ".perf-badge");
  await page.locator(".experimental-features summary").click();
  await page.getByLabel("Experimental example").selectOption("experimentalStrictSlotChildren");
  await expect(page.locator(".diagnostic-message")).toContainText("HTMLInputElement");
  await expect(page.getByText("8:15", { exact: true })).toBeVisible();
  await page.getByLabel("Strict slot children", { exact: true }).uncheck();
  await expect(page.getByText("No type issues found", { exact: true })).toBeVisible();

  await page.locator(".main-tabs > button", { hasText: "Croquis" }).click();
  await waitForTabReady(page, ".perf-badge");
  await expect(page.locator("main .panel").first()).toHaveClass("panel input-panel");
  await page.locator(".experimental-features summary").click();
  await page.getByLabel("Experimental example").selectOption("experimentalInTagComments");
  await page.getByRole("button", { name: "Bindings", exact: true }).click();
  await expect(page.locator(".binding-name")).toHaveText(["label"]);
  await expect(page.getByText("in-template", { exact: true })).toBeVisible();
  await page.getByLabel("In-tag comments", { exact: true }).uncheck();
  await expect(page.locator(".output-panel")).toContainText("Template parse error");
  await page.getByLabel("In-tag comments", { exact: true }).check();
  await expect(page.getByText("in-template", { exact: true })).toBeVisible();
  await page.reload();
  await waitForTabReady(page, ".perf-badge");
  await page.locator(".experimental-features summary").click();
  await expect(page.getByLabel("In-tag comments", { exact: true })).toBeChecked();
});
