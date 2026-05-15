import { test, expect, type Browser, type Page } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { createNpmxVisualParityApps, type AppConfig } from "../../_helpers/apps";
import {
  ensurePortFree,
  killProcess,
  startDevServer,
  waitForHttpReady,
  waitForServerReady,
} from "../../_helpers/server";
import {
  expectVisualParity,
  installVisualStabilityHooks,
  prepareStableVisualState,
} from "../../_helpers/visual-parity";

interface VisualRoute {
  action?: (page: Page) => Promise<void>;
  maxDiffRatio?: number;
  name: string;
  path: string;
  storage?: Record<string, string>;
  viewport?: { height: number; width: number };
}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR = path.resolve(__dirname, "../results/visual-parity/npmx.dev");
const DEFAULT_VIEWPORT = { width: 1280, height: 720 };
const apps = createNpmxVisualParityApps();

const routes: VisualRoute[] = [
  { name: "home", path: "/" },
  { name: "about", path: "/about" },
  { name: "accessibility", path: "/accessibility" },
  { name: "privacy", path: "/privacy" },
  { name: "recharging", path: "/recharging" },
  { name: "settings", path: "/settings" },
  {
    name: "settings-saved-theme",
    path: "/settings",
    storage: {
      "npmx-settings": JSON.stringify({
        accentColorId: "coral",
        preferredBackgroundTheme: "slate",
      }),
    },
  },
  { name: "compare", path: "/compare" },
  { name: "compare-packages", path: "/compare?packages=vue,react", maxDiffRatio: 0.004 },
  { name: "package-vue", path: "/package/vue", maxDiffRatio: 0.004 },
  { name: "org-vue", path: "/org/vue", maxDiffRatio: 0.004 },
  { name: "user-sindresorhus", path: "/~sindresorhus?p=npm", maxDiffRatio: 0.004 },
  { name: "user-orgs-disconnected", path: "/~sindresorhus/orgs", maxDiffRatio: 0.004 },
  { name: "diff-vue", path: "/diff/vue/v/3.5.28...3.5.29", maxDiffRatio: 0.004 },
  {
    name: "search-query",
    path: "/search",
    action: async (page) => {
      const input = page
        .locator('input[type="search"], input[name="q"], input[role="searchbox"]')
        .first();
      await expect(input).toBeVisible({ timeout: 10_000 });
      await input.fill("vue");
      await expect(page.getByText(/Found\s+[\d,]+\s+packages/)).toBeVisible({
        timeout: 20_000,
      });
    },
  },
  {
    name: "mobile-home",
    path: "/",
    viewport: { width: 390, height: 844 },
  },
  { name: "docs-nuxt", path: "/docs/nuxt/v/4.0.0", maxDiffRatio: 0.004 },
];

test.describe("npmx.dev visual parity", () => {
  test.describe.configure({ mode: "serial" });

  let candidateServer: ChildProcess | undefined;
  let referenceServer: ChildProcess | undefined;

  test.beforeAll(async () => {
    referenceServer = await startApp(apps.reference);
    candidateServer = await startApp(apps.candidate);
  });

  test.afterAll(async () => {
    killProcess(candidateServer);
    killProcess(referenceServer);
  });

  for (const route of routes) {
    test(route.name, async ({ browser }) => {
      await compareRoute(browser, route);
    });
  }
});

async function startApp(app: AppConfig): Promise<ChildProcess> {
  if (app.setup) app.setup();
  await ensurePortFree(app.port);

  const server = startDevServer(app);
  await waitForServerReady(server, app.port, app.readyPattern, app.startupTimeout, app.readyDelay);
  await waitForHttpReady(app.url, app.port);
  return server;
}

async function compareRoute(browser: Browser, route: VisualRoute): Promise<void> {
  const context = await browser.newContext({
    colorScheme: "light",
    deviceScaleFactor: 1,
    reducedMotion: "reduce",
    viewport: route.viewport ?? DEFAULT_VIEWPORT,
  });

  try {
    const referencePage = await context.newPage();
    const candidatePage = await context.newPage();

    await Promise.all([setupPage(referencePage), setupPage(candidatePage)]);
    await Promise.all([setupRoute(referencePage, route), setupRoute(candidatePage, route)]);
    await Promise.all([
      openRoute(referencePage, apps.reference.url, route),
      openRoute(candidatePage, apps.candidate.url, route),
    ]);

    if (route.action) {
      await Promise.all([route.action(referencePage), route.action(candidatePage)]);
    }

    await Promise.all([
      prepareStableVisualState(referencePage),
      prepareStableVisualState(candidatePage),
    ]);

    await expectVisualParity(referencePage, candidatePage, {
      maxDiffRatio: route.maxDiffRatio,
      name: route.name,
      outputDir: OUTPUT_DIR,
    });
  } finally {
    await context.close();
  }
}

async function setupPage(page: Page): Promise<void> {
  await installVisualStabilityHooks(page);
  await page.addInitScript(() => {
    localStorage.setItem("npmx-color-mode", "light");
  });
}

async function setupRoute(page: Page, route: VisualRoute): Promise<void> {
  if (!route.storage) return;

  await page.addInitScript((storage) => {
    for (const [key, value] of Object.entries(storage)) {
      localStorage.setItem(key, value);
    }
  }, route.storage);
}

async function openRoute(page: Page, baseUrl: string, route: VisualRoute): Promise<void> {
  const response = await page.goto(`${baseUrl}${route.path}`, {
    timeout: 60_000,
    waitUntil: "domcontentloaded",
  });
  expect(response?.status()).toBeLessThan(500);
  await expect(page.locator("#__nuxt")).toBeAttached({ timeout: 15_000 });
  await page.waitForLoadState("networkidle", { timeout: 10_000 }).catch(() => undefined);
  await page.waitForTimeout(1000);
}
