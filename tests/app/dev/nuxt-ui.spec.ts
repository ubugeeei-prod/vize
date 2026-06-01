import { expect, test, type Page } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { nuxtUiApp, SCREENSHOT_DIR } from "../../_helpers/apps";
import {
  collectConsoleErrors,
  collectHydrationErrors,
  isFatalError,
  verifySSRContent,
} from "../../_helpers/assertions";
import {
  ensurePortFree,
  killProcess,
  startDevServer,
  waitForHttpReady,
  waitForServerReady,
} from "../../_helpers/server";

const app = nuxtUiApp;

async function gotoNuxtUi(page: Page, pathname = "/") {
  return page.goto(new URL(pathname, app.url).toString(), {
    waitUntil: app.waitUntil ?? "networkidle",
    timeout: 30_000,
  });
}

function normalizeNuxtUiSnapshotHtml(html: string): string {
  const normalizedWorktreePath = encodeURIComponent(app.cwd);
  return html
    .replaceAll(normalizedWorktreePath, "__NUXT_UI_WORKTREE__")
    .replaceAll(app.cwd, "__NUXT_UI_WORKTREE__")
    .replace(
      /<script type="application\/json" data-nuxt-logs="nuxt-app">[\s\S]*?<\/script>/,
      '<script type="application/json" data-nuxt-logs="nuxt-app">__NUXT_UI_LOGS__</script>',
    )
    .replace(/\b\d{13}\b/g, "0");
}

test.describe("nuxt-ui dev", () => {
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
    await new Promise((resolve) => setTimeout(resolve, 2000));
  });

  test("home page renders the playground shell", async ({ page }) => {
    const consoleErrors = await collectConsoleErrors(page, app.name);
    const hydrationErrors = await collectHydrationErrors(page);

    await page.setViewportSize({ width: 1440, height: 960 });

    const response = await gotoNuxtUi(page);
    expect(response?.status()).toBeDefined();

    await expect(page.locator(app.mountSelector)).toBeAttached();
    await expect(
      page.locator(app.mountSelector).getByRole("heading", { name: "Playground" }),
    ).toBeVisible();
    await expect(
      page.locator(app.mountSelector).getByRole("button", { name: "Button" }).first(),
    ).toBeVisible();

    const html = await verifySSRContent(page, app.url);
    expect(normalizeNuxtUiSnapshotHtml(html)).toMatchSnapshot("home-ssr");

    expect(consoleErrors.filter(isFatalError)).toHaveLength(0);
    const unexpectedHydrationErrors = hydrationErrors.filter((error) => !/Hydration/i.test(error));
    expect(unexpectedHydrationErrors).toHaveLength(0);
  });

  test("button page supports loading-auto", async ({ page }) => {
    const consoleErrors = await collectConsoleErrors(page, app.name);
    const hydrationErrors = await collectHydrationErrors(page);

    await page.setViewportSize({ width: 1440, height: 960 });

    const response = await gotoNuxtUi(page, "/components/button");
    expect(response?.status()).toBeDefined();

    const buttonPage = page.locator(app.mountSelector);
    const loadingAutoButton = buttonPage.getByRole("button", { name: "Loading auto" });

    await expect(buttonPage.getByRole("button", { name: "Button" }).last()).toBeVisible();
    await expect(loadingAutoButton).toBeVisible();

    await loadingAutoButton.click();
    await expect(loadingAutoButton).toBeDisabled();
    await expect(loadingAutoButton).toBeEnabled({ timeout: 10_000 });

    const html = await verifySSRContent(page, `${app.url}/components/button`);
    expect(normalizeNuxtUiSnapshotHtml(html)).toMatchSnapshot("button-ssr");

    expect(consoleErrors.filter(isFatalError)).toHaveLength(0);
    const unexpectedHydrationErrors = hydrationErrors.filter((error) => !/Hydration/i.test(error));
    expect(unexpectedHydrationErrors).toHaveLength(0);
  });

  test("screenshot", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 960 });

    await gotoNuxtUi(page);

    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, "nuxt-ui-dev.png"),
    });
  });
});
