import { test, expect, type Browser, type Page } from "@playwright/test";
import type { ChildProcess } from "node:child_process";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { createElkVisualParityApps, type AppConfig } from "../../_helpers/apps";
import { disableViteHmrClient } from "../../_helpers/nuxtRuntime";
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
import {
  ELK_RENDER_ROUTE,
  elkRequiredRouteLinks,
  readElkRenderRouteSourceEvidence,
} from "../dev/elk-route-contract";

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
  path.resolve(__dirname, "../../../.vize/artifacts/elk-vrt/artifacts");
const DEFAULT_VIEWPORT = { width: 1280, height: 720 };
const DEFAULT_MAX_DIFF_RATIO = 0.04;
const ELK_MIN_RENDER_ROUTE_ELEMENTS = 100;
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
  { name: "settings-shell", path: ELK_RENDER_ROUTE },
  { name: "settings-shell-mobile", path: ELK_RENDER_ROUTE, viewport: MOBILE_VIEWPORT },
  { name: "explore", path: "/explore" },
  { name: "explore-users", path: "/explore/users" },
  { name: "explore-tags", path: "/explore/tags" },
  { name: "explore-links", path: "/explore/links" },
  { name: "public", path: "/public" },
  { name: "public-local", path: "/public/local" },
  { name: "search", path: "/search" },
  { name: "hashtags", path: "/hashtags" },
  { name: "settings-interface", path: "/settings/interface" },
  { name: "settings-language", path: "/settings/language" },
  { name: "settings-preferences", path: "/settings/preferences" },
  { name: "notifications", path: "/notifications" },
  { name: "compose", path: "/compose" },
  { name: "share-target", path: "/share-target?text=hello" },
];

test.describe("elk visual parity", () => {
  test.describe.configure({ mode: "serial" });

  let candidateServer: ChildProcess | undefined;
  let referenceServer: ChildProcess | undefined;

  test.beforeAll(async ({ browser }) => {
    referenceServer = await startApp(apps.reference);
    candidateServer = await startApp(apps.candidate);
    await warmUpApp(browser, apps.reference);
    await warmUpApp(browser, apps.candidate);
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
  readElkRenderRouteSourceEvidence(app.cwd);
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

async function warmUpApp(browser: Browser, app: AppConfig): Promise<void> {
  const context = await browser.newContext({
    colorScheme: "light",
    deviceScaleFactor: 1,
    reducedMotion: "reduce",
    viewport: DEFAULT_VIEWPORT,
  });

  try {
    const page = await context.newPage();
    const route = routes[0];
    await setupPage(page, route);
    await openRoute(page, app.url, route);
    await prepareStableVisualState(page);
  } finally {
    await context.close();
  }
}

async function setupPage(page: Page, route: VisualRoute): Promise<void> {
  await disableViteHmrClient(page);
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
  await waitForElkPageContent(page, route);
  await page.waitForLoadState("networkidle", { timeout: 10_000 }).catch(() => undefined);
  await page.waitForTimeout(1000);
}

async function waitForElkPageContent(page: Page, route: VisualRoute): Promise<void> {
  const requiredLinks = elkRequiredRouteLinks(route.path);
  await expect
    .poll(() => elkRouteContentState(page, requiredLinks), {
      intervals: [250, 500, 1_000],
      timeout: 90_000,
    })
    .toBe("ready");
}

async function elkRouteContentState(page: Page, requiredLinks: readonly string[]): Promise<string> {
  return page.evaluate(
    ({ links, minElements, selector }) => {
      const root = document.querySelector(selector);
      if (!root) {
        return "missing-root";
      }

      const elementCount = root.querySelectorAll("*").length;
      const missingLinks = links.filter((href) => !root.querySelector(`a[href="${href}"]`));
      if (elementCount >= minElements && missingLinks.length === 0) {
        return "ready";
      }

      return `incomplete:elements=${elementCount}:missing=${missingLinks.join(",")}`;
    },
    {
      links: requiredLinks,
      minElements: ELK_MIN_RENDER_ROUTE_ELEMENTS,
      selector: "#__nuxt",
    },
  );
}
