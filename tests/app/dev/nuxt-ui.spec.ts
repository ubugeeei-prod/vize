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

// Routes the suite navigates to. We pre-warm these once before the tests so Vite
// finishes optimize-deps pre-bundling for the (very heavy) nuxt-ui playground
// before the timed `toBeVisible` assertions run.
const WARMUP_PATHS = ["/", "/components/button"] as const;

// Signatures of the transient Vite optimize-deps churn that produces a broken
// initial load (504 Outdated Optimize Dep -> failed dynamic import -> SSR 500).
// These are dev-server infra hiccups, not vize render bugs, and resolve on reload
// once pre-bundling settles.
const OPTIMIZE_DEP_ERROR =
  /Outdated Optimize Dep|Failed to fetch dynamically imported module|504|new dependencies optimized/i;

/**
 * Hit the dev server for each warmup path until it returns a healthy SSR page
 * (no optimize-dep error markup), so Vite has finished pre-bundling before the
 * browser-driven assertions start.
 */
async function warmUpNuxtUi(): Promise<void> {
  const deadline = Date.now() + 120_000;
  for (const pathname of WARMUP_PATHS) {
    const target = new URL(pathname, app.url).toString();
    let settled = false;
    while (Date.now() < deadline) {
      try {
        const res = await fetch(target, { signal: AbortSignal.timeout(20_000) });
        const body = await res.text();
        const churning = res.status >= 500 || res.status === 504 || OPTIMIZE_DEP_ERROR.test(body);
        if (!churning && body.includes("__nuxt")) {
          settled = true;
          break;
        }
      } catch {
        // Server still pre-bundling / restarting; retry.
      }
      await new Promise((r) => setTimeout(r, 2_000));
    }
    if (!settled) {
      console.log(`[${app.name}] warmup for ${pathname} did not fully settle; continuing`);
    }
  }
}

/**
 * Navigate to a nuxt-ui route, reloading if the dev server serves a transient
 * optimize-deps error (504 / failed dynamic import / SSR 500) instead of the
 * playground. Bounded retries keep this from masking real failures.
 */
async function gotoNuxtUi(page: Page, pathname = "/") {
  const target = new URL(pathname, app.url).toString();
  const maxAttempts = 4;
  let response: Awaited<ReturnType<Page["goto"]>> = null;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    response = await page.goto(target, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });

    const status = response?.status() ?? 0;
    const html = await page.content().catch(() => "");
    const churning = status === 504 || status >= 500 || OPTIMIZE_DEP_ERROR.test(html);

    if (!churning) {
      return response;
    }

    if (attempt < maxAttempts) {
      console.log(
        `[${app.name}] transient optimize-deps error on ${pathname} ` +
          `(status ${status}, attempt ${attempt}); reloading...`,
      );
      // Give Vite a moment to finish re-bundling before retrying.
      await page.waitForTimeout(2_000);
    }
  }

  return response;
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
    // setup + install + dev:prepare + server start + route warmup can exceed the
    // default hook timeout for this heavy playground.
    test.setTimeout(600_000);
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

    // Pre-bundle the routes the suite visits so Vite finishes optimize-deps churn
    // (504 Outdated Optimize Dep / failed dynamic import / SSR 500) before the
    // timed browser assertions run.
    console.log(`Warming up ${app.name} routes...`);
    await warmUpNuxtUi();
    console.log(`${app.name} warmup complete`);
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
