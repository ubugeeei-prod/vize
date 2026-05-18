import { test, expect, type Page } from "@playwright/test";
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

async function gotoMisskey(page: Page) {
  await setupMisskeyMocks(page);

  return page.goto(app.url, {
    waitUntil: app.waitUntil ?? "networkidle",
    timeout: 30_000,
  });
}

async function waitForMisskeyContent(page: Page) {
  const mountEl = page.locator(app.mountSelector);
  let lastError: unknown;

  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      await expect(mountEl).toBeAttached({ timeout: 15_000 });
      await expect
        .poll(
          async () => {
            const html = await mountEl.innerHTML();
            return html.trim().length;
          },
          { timeout: 20_000 },
        )
        .toBeGreaterThan(0);
      return;
    } catch (error) {
      lastError = error;
      if (attempt === 2) {
        break;
      }
      await page.reload({
        waitUntil: app.waitUntil ?? "networkidle",
        timeout: 30_000,
      });
    }
  }

  throw lastError instanceof Error ? lastError : new Error(String(lastError));
}

async function hasCssModuleClass(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const allElements = document.querySelectorAll("*");
    for (const el of allElements) {
      for (const cls of el.classList) {
        if (cls.startsWith("_") && cls.includes("_")) return true;
      }
    }
    return false;
  });
}

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

    const response = await gotoMisskey(page);
    expect(response?.status()).toBeDefined();
    await waitForMisskeyContent(page);
  });

  test("visitor UI renders", async ({ page }) => {
    await gotoMisskey(page);
    await waitForMisskeyContent(page);
  });

  test("scoped CSS: data-v-* attributes exist", async ({ page }) => {
    await gotoMisskey(page);
    await waitForMisskeyContent(page);
    await expect
      .poll(() => verifyScopedCssAttributes(page), { timeout: 30_000 })
      .toBeGreaterThan(0);
  });

  test("CSS Modules: module-generated class names exist", async ({ page }) => {
    await gotoMisskey(page);
    await waitForMisskeyContent(page);
    await expect.poll(() => hasCssModuleClass(page), { timeout: 30_000 }).toBe(true);
  });

  test("async components load", async ({ page }) => {
    await gotoMisskey(page);
    await expect
      .poll(
        () =>
          page.evaluate((sel: string) => {
            const el = document.querySelector(sel);
            return el ? el.querySelectorAll("*").length : 0;
          }, app.mountSelector),
        { timeout: 30_000 },
      )
      .toBeGreaterThan(1);
  });

  test("no fatal console errors", async ({ page }) => {
    const errors = await collectConsoleErrors(page, app.name);

    await gotoMisskey(page);
    await waitForMisskeyContent(page);

    const fatalErrors = errors.filter(isFatalError);
    if (fatalErrors.length > 0) {
      console.log(`Fatal errors in ${app.name}:`, fatalErrors);
    }
    expect(fatalErrors).toHaveLength(0);
  });

  test("screenshot", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });

    await gotoMisskey(page);
    await waitForMisskeyContent(page);

    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, "misskey-dev.png"),
    });
  });
});
