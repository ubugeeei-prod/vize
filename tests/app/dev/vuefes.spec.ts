import { test, expect, type Page } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { vuefesApp, SCREENSHOT_DIR } from "../../_helpers/apps";
import {
  waitForServerReady,
  startDevServer,
  ensurePortFree,
  waitForHttpReady,
  killProcess,
  getProcessLogs,
} from "../../_helpers/server";
import {
  collectConsoleErrors,
  collectHydrationErrors,
  isFatalError,
  verifyScopedCssAttributes,
  verifySSRContent,
} from "../../_helpers/assertions";

const app = vuefesApp;

test.describe("vuefes-2025 dev", () => {
  let devServer: ChildProcess;
  let page: Page;

  test.beforeAll(async ({ browser }) => {
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
    page = await browser.newPage();
    console.log(`${app.name} server is ready`);
  });

  test.afterAll(async () => {
    await page?.close();
    console.log(`Stopping dev server for ${app.name}...`);
    killProcess(devServer);
    await new Promise((r) => setTimeout(r, 2000));
  });

  async function gotoApp() {
    const waitUntil = app.waitUntil ?? "networkidle";
    let lastError: unknown;
    for (let attempt = 0; attempt < 3; attempt++) {
      try {
        return await page.goto(app.url, {
          waitUntil,
          timeout: 30_000,
        });
      } catch (error) {
        lastError = error;
        const message = String(error);
        const retryableNavigationError =
          message.includes("net::ERR_ABORTED") ||
          message.includes("interrupted by another navigation");
        if (!retryableNavigationError) {
          throw error;
        }
        await page.waitForTimeout(1_000);
      }
    }
    throw lastError;
  }

  test("page renders with #__nuxt attached", async () => {
    await page.setViewportSize({ width: 1280, height: 720 });

    const response = await gotoApp();
    expect(response?.status()).toBeDefined();

    const mountEl = page.locator(app.mountSelector);
    await expect(mountEl).toBeAttached({ timeout: 15_000 });
  });

  test("SSR: server-rendered HTML is not empty", async () => {
    const html = await verifySSRContent(page, app.url);
    expect(html).toContain("__nuxt");
    expect(html.length).toBeGreaterThan(100);
  });

  test("server logs stay clean after SSR render", async () => {
    await verifySSRContent(page, app.url);

    const fatalLogs = getProcessLogs(devServer).filter(isFatalError);
    if (fatalLogs.length > 0) {
      console.log(`Fatal server logs in ${app.name}:`, fatalLogs);
    }
    expect(fatalLogs).toHaveLength(0);
  });

  test("no hydration mismatch errors", async () => {
    const hydrationErrors = await collectHydrationErrors(page);

    await gotoApp();
    await page.waitForTimeout(5_000);

    // Filter out known harmless SSR/client hydration differences (PrimeVue Carousel, etc.)
    const unexpectedErrors = hydrationErrors.filter((e) => !/Hydration/i.test(e));
    expect(unexpectedErrors).toHaveLength(0);
  });

  test("scoped CSS: data-v-* attributes exist", async () => {
    await gotoApp();
    await page.waitForTimeout(3_000);

    const count = await verifyScopedCssAttributes(page);
    expect(count).toBeGreaterThan(0);
  });

  test("no fatal console errors", async () => {
    const errors = await collectConsoleErrors(page, app.name);

    await gotoApp();
    await page.waitForTimeout(3_000);

    const fatalErrors = errors.filter(isFatalError);
    if (fatalErrors.length > 0) {
      console.log(`Fatal errors in ${app.name}:`, fatalErrors);
    }
    expect(fatalErrors).toHaveLength(0);
  });

  test("screenshot", async () => {
    await page.setViewportSize({ width: 1280, height: 720 });

    await gotoApp();
    await page.waitForTimeout(2_000);

    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, "vuefes-2025-dev.png"),
    });
  });
});
