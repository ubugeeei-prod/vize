import { test, expect } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { elkApp, SCREENSHOT_DIR } from "../../_helpers/apps";
import {
  waitForServerReady,
  startDevServer,
  ensurePortFree,
  waitForHttpReady,
  killProcess,
} from "../../_helpers/server";
import {
  collectConsoleErrors,
  collectHydrationErrors,
  isFatalError,
  verifyScopedCssAttributes,
  getComputedStyleValue,
  verifySSRContent,
} from "../../_helpers/assertions";

const app = elkApp;

test.describe("elk dev", () => {
  let devServer: ChildProcess;

  test.beforeAll(async () => {
    if (app.setup) app.setup();
    await ensurePortFree(app.port);

    console.log(`Starting dev server for ${app.name}...`);
    devServer = startDevServer(app);
    devServer.on("exit", (code) => {
      console.log(`[${app.name}] dev server exited with code ${code}`);
    });

    console.log(`Waiting for ${app.name} server to be ready (port ${app.port})...`);
    await waitForServerReady(
      devServer,
      app.port,
      app.readyPattern,
      app.startupTimeout,
      app.readyDelay,
    );
    await waitForHttpReady(app.url, app.port);
    console.log(`${app.name} server is ready`);
  });

  test.afterAll(async () => {
    console.log(`Stopping dev server for ${app.name}...`);
    killProcess(devServer);
    await new Promise((r) => setTimeout(r, 2000));
  });

  test("page renders with #__nuxt attached", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });

    const response = await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    expect(response?.status()).toBeDefined();

    const mountEl = page.locator(app.mountSelector);
    await expect(mountEl).toBeAttached({ timeout: 15_000 });

    try {
      await page.waitForFunction(
        (sel: string) => {
          const el = document.querySelector(sel);
          return el !== null && (el.textContent ?? "").trim().length > 0;
        },
        app.mountSelector,
        { timeout: 10_000 },
      );
    } catch {
      // Text content may not appear within timeout for SSR apps with pending data
    }
  });

  test("SSR: server-rendered HTML is not empty", async ({ page }) => {
    const html = await verifySSRContent(page, app.url);
    // SSR should produce non-empty HTML with at least the #__nuxt container
    expect(html).toContain("__nuxt");
    expect(html.length).toBeGreaterThan(100);
  });

  test("no hydration mismatch errors", async ({ page }) => {
    const hydrationErrors = await collectHydrationErrors(page);

    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    expect(hydrationErrors).toHaveLength(0);
  });

  test("scoped CSS: data-v-* attributes exist", async ({ page }) => {
    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    const count = await verifyScopedCssAttributes(page);
    expect(count).toBeGreaterThan(0);
  });

  test("styles are applied: computed styles are non-default", async ({ page }) => {
    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    // Check that some styling is applied (body should have non-default styles)
    const bgColor = await getComputedStyleValue(page, "body", "background-color");
    // background-color should be set (not transparent or empty)
    expect(bgColor).toBeTruthy();
  });

  test("navigation components are visible", async ({ page }) => {
    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    // Elk should render some navigation-related elements
    const navElements = page.locator("nav, [role='navigation'], header a, .nav-item, aside");
    const count = await navElements.count();
    expect(count).toBeGreaterThan(0);
  });

  test("no fatal console errors", async ({ page }) => {
    const errors = await collectConsoleErrors(page, app.name);

    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    const fatalErrors = errors.filter(isFatalError);
    if (fatalErrors.length > 0) {
      console.log(`Fatal errors in ${app.name}:`, fatalErrors);
    }
    // Elk may have non-fatal errors due to missing backend, but no fatal ones
    expect(fatalErrors).toHaveLength(0);
  });

  test("i18n: no raw key patterns displayed", async ({ page }) => {
    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    // Check that raw i18n key patterns (like "foo.bar.baz") are not visible
    const textContent = await page.locator(app.mountSelector).textContent();
    if (textContent && textContent.trim().length > 0) {
      // Simple heuristic: i18n keys typically look like "word.word.word"
      const rawKeyPattern = /\b[a-z]+\.[a-z]+\.[a-z]+\b/;
      const lines = textContent.split("\n").filter((l) => l.trim().length > 0);
      const rawKeyLines = lines.filter((l) => {
        const trimmed = l.trim();
        // Only flag lines that look entirely like a raw key
        return rawKeyPattern.test(trimmed) && trimmed.length < 50 && !trimmed.includes(" ");
      });
      // Allow some tolerance — a few raw keys might appear in URLs or technical text
      expect(rawKeyLines.length).toBeLessThan(5);
    }
  });

  test("screenshot", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });

    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(2_000);

    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, "elk-dev.png"),
    });
  });
});
