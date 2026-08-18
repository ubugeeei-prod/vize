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
  elkRouteReadinessExpectation,
  elkRouteReadinessState,
  readElkRenderRouteSourceEvidence,
} from "../dev/elk-route-contract";
import {
  DEFAULT_MAX_DIFF_RATIO,
  DEFAULT_VIEWPORT,
  elkVisualRoutes,
  type ElkVisualRouteConfig,
} from "./elk-routes";

type VisualRoute = ElkVisualRouteConfig;

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR =
  process.env.VIZE_ELK_VRT_OUTPUT_DIR ??
  path.resolve(__dirname, "../../../.vize/artifacts/elk-vrt/artifacts");
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

const routes: VisualRoute[] = elkVisualRoutes;

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
  const readiness = elkRouteReadinessExpectation(route.path);
  await expect
    .poll(() => elkRouteContentState(page, route.path, readiness.links), {
      intervals: [250, 500, 1_000],
      timeout: 90_000,
    })
    .toBe("ready");
}

async function elkRouteContentState(
  page: Page,
  routePath: string,
  requiredLinks: readonly string[],
): Promise<string> {
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
      links: [...requiredLinks],
      selector: "#__nuxt",
    },
  );

  return elkRouteReadinessState(routePath, observation);
}
