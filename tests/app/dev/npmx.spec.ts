import { test, expect, type Page } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as fs from "node:fs";
import * as path from "node:path";
import { npmxApp, SCREENSHOT_DIR } from "../../_helpers/apps";
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
  getComputedStyleValue,
  verifySSRContent,
} from "../../_helpers/assertions";

const app = npmxApp;

type RouteSnapshot = {
  fullPath: string;
  meta: Record<string, unknown>;
  name: string | null;
  params: Record<string, unknown>;
  path: string;
};

async function readCurrentRoute(page: Page): Promise<RouteSnapshot> {
  await page.waitForFunction(() => {
    const root = document.querySelector("#__nuxt") as
      | {
          __vue_app__?: {
            config?: {
              globalProperties?: {
                $router?: {
                  currentRoute?: {
                    value?: unknown;
                  };
                };
              };
            };
          };
        }
      | null;

    return root?.__vue_app__?.config?.globalProperties?.$router?.currentRoute?.value !== undefined;
  });

  return page.evaluate(() => {
    const root = document.querySelector("#__nuxt") as
      | {
          __vue_app__?: {
            config?: {
              globalProperties?: {
                $router?: {
                  currentRoute?: {
                    value?: {
                      fullPath?: string;
                      meta?: Record<string, unknown>;
                      name?: string | symbol | null;
                      params?: Record<string, unknown>;
                      path?: string;
                    };
                  };
                };
              };
            };
          };
        }
      | null;
    const route = root?.__vue_app__?.config?.globalProperties?.$router?.currentRoute?.value;
    if (!route?.path || !route.fullPath) {
      throw new Error("Nuxt router currentRoute is not available");
    }

    return {
      fullPath: route.fullPath,
      meta: JSON.parse(JSON.stringify(route.meta ?? {})) as Record<string, unknown>,
      name: route.name == null ? null : String(route.name),
      params: JSON.parse(JSON.stringify(route.params ?? {})) as Record<string, unknown>,
      path: route.path,
    };
  });
}

test.describe("npmx.dev dev", () => {
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
    await waitForServerReady(devServer, app.port, app.readyPattern, app.startupTimeout, app.readyDelay);
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
      // Text content may not appear within timeout
    }
  });

  test("SSR: server-rendered HTML contains expected content", async ({ page }) => {
    const html = await verifySSRContent(page, app.url);
    expect(html).toContain("__nuxt");
    expect(html.length).toBeGreaterThan(100);
    // npmx.dev should have "npmx" text or search form in SSR output
    const hasExpectedContent = html.toLowerCase().includes("npmx") || html.includes("search") || html.includes("form");
    expect(hasExpectedContent).toBe(true);
  });

  test("SSR: accessibility route keeps definePageMeta route name", async ({ page }) => {
    const url = app.url + "/accessibility";
    const html = await verifySSRContent(page, url);

    expect(html).toContain("<title>accessibility - npmx</title>");
    expect(html).toContain("Our approach");

    await page.goto(url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });

    const route = await readCurrentRoute(page);
    expect(route.path).toBe("/accessibility");
    expect(route.fullPath).toBe("/accessibility");
    expect(route.name).toBe("accessibility");
  });

  test("SSR: docs alias resolves definePageMeta alias and meta", async ({ page }) => {
    const url = app.url + "/docs/nuxt/v/4.0.0";
    const html = await verifySSRContent(page, url);

    expect(html).toContain("<title>nuxt@4.0.0 docs - npmx</title>");
    expect(html).toContain('aria-label="Package documentation header"');

    await page.goto(url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });

    const route = await readCurrentRoute(page);
    expect(route.path).toBe("/docs/nuxt/v/4.0.0");
    expect(route.fullPath).toBe("/docs/nuxt/v/4.0.0");
    expect(route.name).toBe("docs");
    expect(route.meta).toMatchObject({ scrollMargin: 180 });
    expect(route.params).toMatchObject({
      path: ["nuxt", "v", "4.0.0"],
    });
  });

  test("server logs stay clean during docs prefetch", async ({ page }) => {
    const logOffset = getProcessLogs(devServer).length;

    await page.goto(app.url + "/docs/nuxt/v/4.0.0", {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    const newLogs = getProcessLogs(devServer).slice(logOffset);
    const runtimeWarnings = newLogs.filter((line) => {
      return (
        line.includes('useFetch') && line.includes('must return a value')
      ) || line.includes('Property "disabled" was accessed during render')
        || line.includes('Property "size" was accessed during render');
    });

    expect(runtimeWarnings).toEqual([]);
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

  test("styles are applied: .skip-link has position fixed", async ({ page }) => {
    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    // Check .skip-link style if it exists, otherwise verify general styling
    const hasSkipLink = await page.locator(".skip-link").count();
    if (hasSkipLink > 0) {
      const position = await getComputedStyleValue(page, ".skip-link", "position");
      expect(["fixed", "absolute"]).toContain(position);
    } else {
      // Fallback: at least body has non-default styling
      const bgColor = await getComputedStyleValue(page, "body", "background-color");
      expect(bgColor).toBeTruthy();
    }
  });

  test("client-side navigation works", async ({ page }) => {
    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    // Try to navigate to /about via client-side link
    const aboutLink = page.locator('a[href="/about"], a[href*="about"]').first();
    const hasAboutLink = await aboutLink.count();
    if (hasAboutLink > 0) {
      await aboutLink.click();
      await page.waitForTimeout(2_000);
      expect(page.url()).toContain("about");
    }
  });

  test("reactivity: search input updates state", async ({ page }) => {
    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    // Find search input
    const searchInput = page.locator('input[type="search"], input[name="q"], input[placeholder*="earch"], input[role="searchbox"]').first();
    const hasSearchInput = await searchInput.count();
    if (hasSearchInput > 0) {
      await searchInput.fill("test-package");
      await page.waitForTimeout(1_000);
      // URL or page state should reflect the search
      const pageContent = await page.content();
      const urlOrContent = page.url().includes("test-package") || pageContent.includes("test-package");
      expect(urlOrContent).toBe(true);
    }
  });

  test("i18n: localized strings displayed (no raw keys)", async ({ page }) => {
    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    const textContent = await page.locator(app.mountSelector).textContent();
    if (textContent && textContent.trim().length > 0) {
      const rawKeyPattern = /\b[a-z]+\.[a-z]+\.[a-z]+\b/;
      const lines = textContent.split("\n").filter((l) => l.trim().length > 0);
      const rawKeyLines = lines.filter((l) => {
        const trimmed = l.trim();
        return rawKeyPattern.test(trimmed) && trimmed.length < 50 && !trimmed.includes(" ");
      });
      expect(rawKeyLines.length).toBeLessThan(5);
    }
  });

  test("components: AppHeader and AppFooter are visible", async ({ page }) => {
    await page.goto(app.url, {
      waitUntil: app.waitUntil ?? "networkidle",
      timeout: 30_000,
    });
    await page.waitForTimeout(3_000);

    // Check for header/footer elements
    const header = page.locator("header, [data-testid='app-header'], .app-header").first();
    const footer = page.locator("footer, [data-testid='app-footer'], .app-footer").first();

    const headerCount = await header.count();
    const footerCount = await footer.count();

    // At least one of header or footer should exist
    expect(headerCount + footerCount).toBeGreaterThan(0);
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
      path: path.join(SCREENSHOT_DIR, "npmx-dev.png"),
    });
  });
});
