import { expect, test, type Browser, type Page } from "@playwright/test";
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
import { killProcess } from "../../_helpers/server";
import {
  DEAD_NUXT_UI_SSR_BRIDGE,
  isHealthyNuxtUiSsrResponse,
  startNuxtUiDevServer,
} from "./nuxt-ui-dev-server";
import { verifyNuxtUiAuthoredSourceHmr } from "./nuxt-ui-hmr";
import { normalizeNuxtUiSnapshotHtml } from "./nuxt-ui-snapshot";

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
const SSR_WARMUP_REQUEST_TIMEOUT_MS = 90_000;
const BROWSER_WARMUP_NAVIGATION_TIMEOUT_MS = 90_000;

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
        const res = await fetch(target, {
          signal: AbortSignal.timeout(SSR_WARMUP_REQUEST_TIMEOUT_MS),
        });
        const body = await res.text();
        if (DEAD_NUXT_UI_SSR_BRIDGE.test(body)) {
          throw new Error(`Nuxt UI SSR bridge closed while warming ${pathname}`);
        }
        const churning =
          !isHealthyNuxtUiSsrResponse(res.status, body) || OPTIMIZE_DEP_ERROR.test(body);
        if (!churning) {
          settled = true;
          break;
        }
      } catch (error) {
        if (error instanceof Error && error.message.includes("SSR bridge closed")) {
          throw error;
        }
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
 * Browser-side warmup. The `fetch()` warmup above only settles the SSR transform;
 * the FIRST real browser navigation still triggers a fresh client-side
 * optimize-deps pass (the `/_nuxt/...` module graph), which is what produces the
 * 504 -> failed dynamic import -> SSR 500 cascade on this very heavy playground.
 * Drive each route through an actual browser page (retrying past the churn) so
 * Vite finishes the client-side pre-bundle BEFORE the timed assertions run.
 */
async function warmUpNuxtUiInBrowser(browser: Browser): Promise<void> {
  const context = await browser.newContext();
  try {
    const page = await context.newPage();
    for (const pathname of WARMUP_PATHS) {
      const target = new URL(pathname, app.url).toString();
      const deadline = Date.now() + 120_000;
      let settled = false;
      while (Date.now() < deadline) {
        try {
          const res = await page.goto(target, {
            waitUntil: "domcontentloaded",
            timeout: BROWSER_WARMUP_NAVIGATION_TIMEOUT_MS,
          });
          const status = res?.status() ?? 0;
          const html = await page.content().catch(() => "");
          if (DEAD_NUXT_UI_SSR_BRIDGE.test(html)) {
            throw new Error(`Nuxt UI SSR bridge closed during browser warmup for ${pathname}`);
          }
          const churning =
            !isHealthyNuxtUiSsrResponse(status, html) || OPTIMIZE_DEP_ERROR.test(html);
          if (!churning) {
            settled = true;
            break;
          }
        } catch (error) {
          if (error instanceof Error && error.message.includes("SSR bridge closed")) {
            throw error;
          }
          // Navigation aborted mid-rebundle (e.g. failed dynamic import); retry.
        }
        await page.waitForTimeout(2_000);
      }
      if (!settled) {
        console.log(
          `[${app.name}] browser warmup for ${pathname} did not fully settle; continuing`,
        );
      }
    }
  } finally {
    await context.close();
  }
}

/**
 * Navigate to a nuxt-ui route, reloading if the dev server serves a transient
 * optimize-deps error (504 / failed dynamic import / SSR 500) instead of the
 * playground. Bounded retries keep this from masking real failures.
 *
 * A navigation that never reaches `load` is the same churn in its worst form:
 * the HMR suite's authored-source edits make Nuxt regenerate its templates, so
 * the next route Vite serves re-transforms the (very heavy) module graph and can
 * full-reload the page mid-navigation. Retry those the same way, bounded by an
 * overall budget so a genuinely stuck page still fails with its own timeout.
 */
async function gotoNuxtUi(page: Page, pathname = "/") {
  const target = new URL(pathname, app.url).toString();
  const maxAttempts = 6;
  const deadline = Date.now() + 120_000;
  let response: Awaited<ReturnType<Page["goto"]>> = null;

  for (let attempt = 1; attempt <= maxAttempts; attempt++) {
    let status = 0;
    let churning: boolean;
    try {
      response = await page.goto(target, {
        waitUntil: app.waitUntil ?? "networkidle",
        timeout: BROWSER_WARMUP_NAVIGATION_TIMEOUT_MS,
      });

      status = response?.status() ?? 0;
      const html = await page.content().catch(() => "");
      if (DEAD_NUXT_UI_SSR_BRIDGE.test(html)) {
        throw new Error(`Nuxt UI SSR bridge closed while loading ${pathname}`);
      }
      churning = status === 504 || status >= 500 || OPTIMIZE_DEP_ERROR.test(html);
    } catch (error) {
      const navigationTimedOut = error instanceof Error && error.name === "TimeoutError";
      if (!navigationTimedOut || attempt === maxAttempts || Date.now() >= deadline) throw error;
      churning = true;
    }

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

async function waitForVueHydration(page: Page, buttonName: string): Promise<void> {
  await page.waitForFunction(
    (name) => {
      const button = Array.from(document.querySelectorAll("button")).find(
        (el) => el.textContent?.trim() === name || el.ariaLabel === name,
      ) as (HTMLButtonElement & { __vueParentComponent?: unknown }) | undefined;
      return Boolean(button?.__vueParentComponent);
    },
    buttonName,
    { timeout: 30_000 },
  );
}

test.describe("nuxt-ui dev", () => {
  let devServer: ChildProcess;
  let hmrStartupLogStart = 0;

  test.beforeAll(async ({ browser }) => {
    // setup + install + dev:prepare + server start (with a bounded restart when
    // the SSR bridge dies) + route warmup can exceed the default hook timeout for
    // this heavy playground.
    test.setTimeout(900_000);
    if (app.setup) app.setup();

    const started = await startNuxtUiDevServer();
    devServer = started.devServer;
    hmrStartupLogStart = started.startupLogStart;
    console.log(`${app.name} server is ready`);

    // Pre-bundle the routes the suite visits so Vite finishes optimize-deps churn
    // (504 Outdated Optimize Dep / failed dynamic import / SSR 500) before the
    // timed browser assertions run.
    console.log(`Warming up ${app.name} routes (SSR)...`);
    await warmUpNuxtUi();
    console.log(`Warming up ${app.name} routes (browser / client optimize-deps)...`);
    await warmUpNuxtUiInBrowser(browser);
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
      page.locator(app.mountSelector).getByRole("link", { name: "Button" }).first(),
    ).toBeVisible();

    const html = await verifySSRContent(page, app.url);
    expect(normalizeNuxtUiSnapshotHtml(html, { cwd: app.cwd })).toMatchSnapshot("home-ssr");

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

    await waitForVueHydration(page, "Loading auto");
    await loadingAutoButton.click();
    await expect(loadingAutoButton).toBeDisabled();
    await expect(loadingAutoButton).toBeEnabled({ timeout: 10_000 });

    const html = await verifySSRContent(page, `${app.url}/components/button`);
    expect(normalizeNuxtUiSnapshotHtml(html, { cwd: app.cwd })).toMatchSnapshot("button-ssr");

    expect(consoleErrors.filter(isFatalError)).toHaveLength(0);
    const unexpectedHydrationErrors = hydrationErrors.filter((error) => !/Hydration/i.test(error));
    expect(unexpectedHydrationErrors).toHaveLength(0);
  });

  test("authored component source hot-updates without reloading", async ({ page }) => {
    // Above the sum of the helper's inner budgets (180s initial client module,
    // 120s readiness, 60s per DOM assertion plus fixed sleeps), so a failure
    // surfaces as the specific inner assertion instead of a generic timeout.
    test.setTimeout(540_000);
    await verifyNuxtUiAuthoredSourceHmr({
      page,
      devServer,
      startupLogStart: hmrStartupLogStart,
      cwd: app.cwd,
      mountSelector: app.mountSelector,
      appName: app.name,
      goto: () => gotoNuxtUi(page, "/components/button"),
      waitForHydration: () => waitForVueHydration(page, "Button"),
    });
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
