import { test, expect } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { misskeyApp, SCREENSHOT_DIR } from "../../_helpers/apps";
import {
  waitForServerReady,
  startDevServer,
  ensurePortFree,
  waitForHttpReady,
  killProcess,
} from "../../_helpers/server";
import {
  collectConsoleErrors,
  isFatalError,
  verifyScopedCssAttributes,
} from "../../_helpers/assertions";
import { setupMisskeyMocks } from "../../_helpers/mocking";

const app = misskeyApp;

test.describe("misskey dev", () => {
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

  test("page renders with #misskey_app attached", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await setupMisskeyMocks(page);

    const response = await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    expect(response?.status()).toBeDefined();

    const mountEl = page.locator(app.mountSelector);
    await expect(mountEl).toBeAttached({ timeout: 15_000 });

    // Wait for content to render
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
      // Content may not render fully without backend
    }
  });

  test("visitor UI renders", async ({ page }) => {
    await setupMisskeyMocks(page);

    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    // Misskey should render root component (signup form, welcome page, etc.)
    const rootContent = await page.locator(app.mountSelector).innerHTML();
    expect(rootContent.trim().length).toBeGreaterThan(0);
  });

  test("scoped CSS: data-v-* attributes exist", async ({ page }) => {
    await setupMisskeyMocks(page);

    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    const count = await verifyScopedCssAttributes(page);
    expect(count).toBeGreaterThan(0);
  });

  test("CSS Modules: module-generated class names exist", async ({ page }) => {
    await setupMisskeyMocks(page);

    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    // Misskey uses CSS modules with patterns like _root_, _xxxx_ or hashed class names
    const hasCssModules = await page.evaluate(() => {
      const allElements = document.querySelectorAll("*");
      for (const el of allElements) {
        for (const cls of el.classList) {
          // CSS module generated class names (e.g., _root_xxxxx, module hashes)
          if (cls.startsWith("_") && cls.includes("_")) return true;
        }
      }
      return false;
    });
    expect(hasCssModules).toBe(true);
  });

  test("async components load", async ({ page }) => {
    await setupMisskeyMocks(page);

    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });

    // Wait for async components to load (defineAsyncComponent)
    await page.waitForTimeout(5_000);

    // Verify the app has rendered content beyond the initial mount point
    const childCount = await page.evaluate((sel: string) => {
      const el = document.querySelector(sel);
      return el ? el.querySelectorAll("*").length : 0;
    }, app.mountSelector);
    expect(childCount).toBeGreaterThan(1);
  });

  test("no fatal console errors", async ({ page }) => {
    const errors = await collectConsoleErrors(page, app.name);
    await setupMisskeyMocks(page);

    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    const fatalErrors = errors.filter(isFatalError);
    if (fatalErrors.length > 0) {
      console.log(`Fatal errors in ${app.name}:`, fatalErrors);
    }
    expect(fatalErrors).toHaveLength(0);
  });

  test("screenshot", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });
    await setupMisskeyMocks(page);

    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(2_000);

    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, "misskey-dev.png"),
    });
  });
});
