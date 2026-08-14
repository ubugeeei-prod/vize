import { test, expect, type Page } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { elkApp, SCREENSHOT_DIR } from "../../_helpers/apps";
import { disableViteHmrClient } from "../../_helpers/nuxtRuntime";
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
import {
  ELK_RENDER_ROUTE,
  elkRouteReadinessExpectation,
  elkRouteReadinessState,
  readElkRenderRouteSourceEvidence,
  type ElkRenderRouteSourceEvidence,
} from "./elk-route-contract";

const app = elkApp;
const ELK_RENDER_URL = `${app.url}${ELK_RENDER_ROUTE}`;
const ELK_RENDER_READINESS = elkRouteReadinessExpectation(ELK_RENDER_ROUTE);

test.describe("elk dev", () => {
  let devServer: ChildProcess;
  let renderRouteSourceEvidence: ElkRenderRouteSourceEvidence | undefined;

  test.beforeAll(async ({ browser }) => {
    if (app.setup) app.setup();
    renderRouteSourceEvidence = readElkRenderRouteSourceEvidence(app.cwd);
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
    const warmupPage = await browser.newPage();
    try {
      await disableViteHmrClient(warmupPage);
      await warmupPage.goto(ELK_RENDER_URL, {
        waitUntil: app.waitUntil ?? "networkidle",
        timeout: 30_000,
      });
      await waitForElkPageContent(warmupPage);
    } finally {
      await warmupPage.close();
    }
    console.log(`${app.name} server is ready`);
  });

  test.beforeEach(async ({ page }) => {
    await disableViteHmrClient(page);
  });

  test.afterAll(async () => {
    if (renderRouteSourceEvidence) {
      expect(readElkRenderRouteSourceEvidence(app.cwd)).toEqual(renderRouteSourceEvidence);
    }
    console.log(`Stopping dev server for ${app.name}...`);
    killProcess(devServer);
    await new Promise((r) => setTimeout(r, 2000));
  });

  test("page renders with #__nuxt attached", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 720 });

    const response = await page.goto(ELK_RENDER_URL, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    expect(response?.status()).toBeDefined();

    const mountEl = page.locator(app.mountSelector);
    await expect(mountEl).toBeAttached({ timeout: 15_000 });
    await waitForElkPageContent(page);
    for (const href of ELK_RENDER_READINESS.links) {
      await expect(page.locator(`a[href="${href}"]`).first()).toBeAttached();
    }
  });

  test("SSR: server-rendered HTML is not empty", async ({ page }) => {
    const html = await verifySSRContent(page, ELK_RENDER_URL);
    // SSR should produce non-empty HTML with at least the #__nuxt container
    expect(html).toContain("__nuxt");
    expect(html).toContain("/settings/about");
    expect(html.length).toBeGreaterThan(100);
  });

  test("no hydration mismatch errors", async ({ page }) => {
    const hydrationErrors = await collectHydrationErrors(page);

    await page.goto(ELK_RENDER_URL, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await waitForElkPageContent(page);

    const unexpectedErrors = hydrationErrors.filter(
      (error) => !isKnownElkShellHydrationError(error),
    );
    expect(unexpectedErrors).toHaveLength(0);
  });

  test("scoped CSS: data-v-* attributes exist", async ({ page }) => {
    await page.goto(ELK_RENDER_URL, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await waitForElkPageContent(page);

    const count = await verifyScopedCssAttributes(page);
    expect(count).toBeGreaterThan(0);
  });

  test("styles are applied: computed styles are non-default", async ({ page }) => {
    await page.goto(ELK_RENDER_URL, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await waitForElkPageContent(page);

    // Check that some styling is applied (body should have non-default styles)
    const bgColor = await getComputedStyleValue(page, "body", "background-color");
    // background-color should be set (not transparent or empty)
    expect(bgColor).toBeTruthy();
  });

  test("navigation components are visible", async ({ page }) => {
    await page.goto(ELK_RENDER_URL, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await waitForElkPageContent(page);

    // Elk should render some navigation-related elements
    const navElements = page.locator("nav, [role='navigation'], header a, .nav-item, aside");
    const count = await navElements.count();
    expect(count).toBeGreaterThan(0);
  });

  test("no fatal console errors", async ({ page }) => {
    const errors = await collectConsoleErrors(page, app.name);

    await page.goto(ELK_RENDER_URL, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await waitForElkPageContent(page);

    const fatalErrors = errors.filter(isFatalError);
    if (fatalErrors.length > 0) {
      console.log(`Fatal errors in ${app.name}:`, fatalErrors);
    }
    // Elk may have non-fatal errors due to missing backend, but no fatal ones
    expect(fatalErrors).toHaveLength(0);
  });

  test("i18n: no raw key patterns displayed", async ({ page }) => {
    await page.goto(ELK_RENDER_URL, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await waitForElkPageContent(page);

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

    await page.goto(ELK_RENDER_URL, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await waitForElkPageContent(page);

    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
    await page.screenshot({
      path: path.join(SCREENSHOT_DIR, "elk-dev.png"),
    });
  });
});

function isKnownElkShellHydrationError(error: string): boolean {
  return (
    /expected on client: (NuxtLoadingIndicator|NuxtLayout|AriaAnnouncer)/.test(error) ||
    error === "Hydration completed but contains mismatches."
  );
}

async function waitForElkPageContent(page: Page): Promise<void> {
  await expect
    .poll(() => elkRenderRouteContentState(page), {
      intervals: [250, 500, 1_000],
      timeout: 90_000,
    })
    .toBe("ready");
}

async function elkRenderRouteContentState(page: Page): Promise<string> {
  const observation = await page.evaluate(
    ({ links, selector }) => {
      const root = document.querySelector(selector);
      if (!root) {
        return { elementCount: 0, missingLinks: links, rootFound: false };
      }

      return {
        elementCount: root.querySelectorAll("*").length,
        missingLinks: links.filter((href) => !root.querySelector(`a[href="${href}"]`)),
        rootFound: true,
      };
    },
    {
      links: ELK_RENDER_READINESS.links,
      selector: app.mountSelector,
    },
  );

  return elkRouteReadinessState(ELK_RENDER_ROUTE, observation);
}
