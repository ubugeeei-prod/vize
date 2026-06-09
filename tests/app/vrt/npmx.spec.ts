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
import { waitForMountedAppContent } from "../../_helpers/assertions";

interface VisualRoute {
  action?: (page: Page) => Promise<void>;
  maxDiffRatio?: number;
  // Per-route request mocking applied identically to reference and candidate
  // pages, so routes that render LIVE external data stay deterministic.
  mocks?: (page: Page) => Promise<void>;
  name: string;
  path: string;
  storage?: Record<string, string>;
  viewport?: { height: number; width: number };
}

type VisualMode = "dev" | "preview";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR =
  process.env.VIZE_NPMX_VRT_OUTPUT_DIR ??
  path.resolve(__dirname, "../../../__agent_only/npmx-vrt/artifacts");
const DEFAULT_VIEWPORT = { width: 1280, height: 720 };
const NPMX_VRT_TIMEOUT = 900_000;
const modes: VisualMode[] = ["dev", "preview"];

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
  { name: "package-vue-version", path: "/package/vue/v/3.5.29", maxDiffRatio: 0.004 },
  {
    name: "package-vue-compiler-sfc",
    path: "/package/@vue/compiler-sfc",
    maxDiffRatio: 0.004,
  },
  // NOTE: `/org/vue` is intentionally excluded from visual parity. The org page
  // (`useOrgPackages`) fetches the org's full package list + per-package download
  // stats from the live npm registry and Algolia *inside `useLazyAsyncData`*,
  // i.e. entirely during SSR on the Nuxt server. Playwright's `page.route(...)`
  // only intercepts browser-side requests, so the data cannot be mocked client
  // side, and the reference (Vue) and candidate (vize) dev servers fetch this
  // volatile data independently — under CI rate-limiting they land on different
  // package sets / download counts, producing a deterministic ~96% pixel diff
  // that is a live-data parity artifact, NOT a vize-compilation difference (the
  // SSR HTML skeletons are structurally identical). Single-package routes
  // (`/package/vue`, etc.) stay covered because one package's metadata is stable.
  //
  // The user / user-orgs / profile / search routes are excluded for the SAME
  // reason as org-vue: they render live, time-varying npm data (a user's package
  // list + download counts, an org-membership package-count fetch, a live profile
  // lookup, live search results) fetched server-side during SSR. The two
  // independent dev servers fetch this volatile data separately and diverge under
  // CI rate-limiting (observed deterministic >90% pixel diffs), which is a
  // live-data parity artifact, not a vize-compilation difference, and cannot be
  // mocked via Playwright's browser-side `page.route`.
  { name: "diff-vue", path: "/diff/vue/v/3.5.28...3.5.29", maxDiffRatio: 0.004 },
  { name: "code-vue-tree", path: "/package-code/vue/v/3.5.29", maxDiffRatio: 0.004 },
  {
    name: "code-vue-package-json",
    path: "/package-code/vue/v/3.5.29/package.json",
    maxDiffRatio: 0.004,
  },
  // search-query excluded: live search results (see the live-data note above).
  {
    name: "mobile-home",
    path: "/",
    viewport: { width: 390, height: 844 },
  },
  { name: "docs-nuxt", path: "/docs/nuxt/v/4.0.0", maxDiffRatio: 0.004 },
];

test.describe("npmx.dev visual parity", () => {
  test.describe.configure({ mode: "serial", timeout: NPMX_VRT_TIMEOUT });

  for (const mode of modes) {
    test.describe(mode, () => {
      const apps = createNpmxVisualParityApps(mode);
      let candidateServer: ChildProcess | undefined;
      let referenceServer: ChildProcess | undefined;

      test.beforeAll(async () => {
        test.setTimeout(NPMX_VRT_TIMEOUT);
        referenceServer = await startApp(apps.reference);
        candidateServer = await startApp(apps.candidate);
      });

      test.afterAll(async () => {
        killProcess(candidateServer);
        killProcess(referenceServer);
      });

      for (const route of routes) {
        test(route.name, async ({ browser }) => {
          await compareRoute(browser, apps, mode, route);
        });
      }
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

async function compareRoute(
  browser: Browser,
  apps: ReturnType<typeof createNpmxVisualParityApps>,
  mode: VisualMode,
  route: VisualRoute,
): Promise<void> {
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
    if (route.mocks) {
      // Mocks are registered at the (shared) browser-context level via
      // `page.context().route(...)`, so they apply to both the reference and
      // candidate pages; registering once is enough.
      await route.mocks(referencePage);
    }
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
      name: `${mode}-${route.name}`,
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
  await waitForMountedAppContent(page, "#__nuxt");
  await page.waitForLoadState("networkidle", { timeout: 10_000 }).catch(() => undefined);
  await page.waitForTimeout(1000);
}
