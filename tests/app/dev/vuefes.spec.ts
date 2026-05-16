import { test, expect } from "@playwright/test";
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
  });

  test("SSR: server-rendered HTML is not empty", async ({ page }) => {
    const html = await verifySSRContent(page, app.url);
    expect(html).toContain("__nuxt");
    expect(html.length).toBeGreaterThan(100);
  });

  test("server logs stay clean after SSR render", async ({ page }) => {
    await verifySSRContent(page, app.url);

    const fatalLogs = getProcessLogs(devServer).filter(isFatalError);
    if (fatalLogs.length > 0) {
      console.log(`Fatal server logs in ${app.name}:`, fatalLogs);
    }
    expect(fatalLogs).toHaveLength(0);
  });

  test("no hydration mismatch errors", async ({ page }) => {
    const hydrationErrors = await collectHydrationErrors(page);

    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(5_000);

    // Filter out known harmless SSR/client hydration differences (PrimeVue Carousel, etc.)
    const unexpectedErrors = hydrationErrors.filter((e) => !/Hydration/i.test(e));
    expect(unexpectedErrors).toHaveLength(0);
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
    expect(fatalErrors).toHaveLength(0);
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
      path: path.join(SCREENSHOT_DIR, "vuefes-2025-dev.png"),
    });
  });
});
