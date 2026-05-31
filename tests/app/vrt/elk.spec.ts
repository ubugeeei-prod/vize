import { test, expect, type Browser, type Page } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { createElkVisualParityApps, type AppConfig } from "../../_helpers/apps";
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
  maxDiffRatio?: number;
  name: string;
  path: string;
  storage?: Record<string, string>;
  viewport?: { height: number; width: number };
}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR =
  process.env.VIZE_ELK_VRT_OUTPUT_DIR ??
  path.resolve(__dirname, "../../../__agent_only/elk-vrt/artifacts");
const DEFAULT_VIEWPORT = { width: 1280, height: 720 };
const DEFAULT_MAX_DIFF_RATIO = 0.04;
const MOBILE_VIEWPORT = { width: 390, height: 844 };
const apps = createElkVisualParityApps();

const defaultStorage = {
  "elk-hide-explore-news-tips": "true",
  "elk-hide-explore-posts-tips": "true",
  "elk-hide-explore-tags-tips": "true",
  "elk-pwa-hide-install": "true",
  "elk-settings": JSON.stringify({
    colorMode: "light",
    disabledTranslationLanguages: [],
    fontSize: "16px",
    language: "en-US",
    preferences: {
      enableDataSaving: true,
      experimentalVirtualScroller: false,
      optimizeForLowPerformanceDevice: true,
    },
  }),
} satisfies Record<string, string>;

const routes: VisualRoute[] = [
  { name: "home", path: "/" },
  { name: "home-mobile", path: "/", viewport: MOBILE_VIEWPORT },
  { name: "explore", path: "/explore" },
  { name: "explore-users", path: "/explore/users" },
  { name: "explore-tags", path: "/explore/tags" },
  { name: "explore-links", path: "/explore/links" },
  { name: "public", path: "/public" },
  { name: "public-local", path: "/public/local" },
  { name: "search", path: "/search" },
  { name: "hashtags", path: "/hashtags" },
  { name: "settings", path: "/settings" },
  { name: "settings-interface", path: "/settings/interface" },
  { name: "settings-language", path: "/settings/language" },
  { name: "settings-preferences", path: "/settings/preferences" },
  { name: "settings-about", path: "/settings/about" },
  { name: "notifications", path: "/notifications" },
  { name: "compose", path: "/compose" },
  { name: "share-target", path: "/share-target?text=hello" },
];

test.describe("elk visual parity", () => {
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

    await Promise.all([setupPage(referencePage, route), setupPage(candidatePage, route)]);
    await Promise.all([
      openRoute(referencePage, apps.reference.url, route),
      openRoute(candidatePage, apps.candidate.url, route),
    ]);

    await Promise.all([
      prepareStableVisualState(referencePage),
      prepareStableVisualState(candidatePage),
    ]);

    await expectVisualParity(referencePage, candidatePage, {
      maxDiffRatio: route.maxDiffRatio ?? DEFAULT_MAX_DIFF_RATIO,
      name: route.name,
      outputDir: OUTPUT_DIR,
    });
  } finally {
    await context.close();
  }
}

async function setupPage(page: Page, route: VisualRoute): Promise<void> {
  await installVisualStabilityHooks(page);
  await page.addInitScript(
    (storage) => {
      localStorage.clear();
      for (const [key, value] of Object.entries(storage)) {
        localStorage.setItem(key, value);
      }
    },
    { ...defaultStorage, ...route.storage },
  );
}

async function openRoute(page: Page, baseUrl: string, route: VisualRoute): Promise<void> {
  const response = await page.goto(`${baseUrl}${route.path}`, {
    timeout: 60_000,
    waitUntil: "domcontentloaded",
  });
  expect(response?.status()).toBeLessThan(500);
  await expect(page.locator("#__nuxt")).toBeAttached({ timeout: 15_000 });
  await expect
    .poll(
      () =>
        page.evaluate(() => {
          const el = document.querySelector("#__nuxt");
          return el?.textContent?.trim().length ?? 0;
        }),
      { timeout: 30_000 },
    )
    .toBeGreaterThan(0);
  await page.waitForLoadState("networkidle", { timeout: 10_000 }).catch(() => undefined);
  await page.waitForTimeout(1000);
}
