import { test, expect, type Browser, type Page } from "@playwright/test";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { createFrontendPhpconVisualParityApps, type AppConfig } from "../../_helpers/apps";
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
  name: string;
  path: string;
  viewport?: { height: number; width: number };
}

type VisualMode = "dev" | "preview";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR =
  process.env.VIZE_FRONTEND_PHPCON_VRT_OUTPUT_DIR ??
  path.resolve(__dirname, "../../../__agent_only/frontend-phpcon-vrt/artifacts");
const DEFAULT_VIEWPORT = { width: 1280, height: 720 };
const MOBILE_VIEWPORT = { width: 390, height: 844 };
const FRONTEND_PHPCON_VRT_TIMEOUT = 900_000;
const modes: VisualMode[] = ["dev", "preview"];

const routes: VisualRoute[] = [
  { name: "home", path: "/", maxDiffRatio: 0.004 },
  { name: "home-mobile", path: "/", viewport: MOBILE_VIEWPORT, maxDiffRatio: 0.004 },
  {
    name: "mobile-menu",
    path: "/",
    viewport: MOBILE_VIEWPORT,
    maxDiffRatio: 0.004,
    action: async (page) => {
      await openMobileMenu(page);
      await page.waitForTimeout(1200);
    },
  },
  { name: "about", path: "/about" },
  { name: "news", path: "/news/2026-05-06-social-gathering-ticket" },
  { name: "timetable", path: "/timetable" },
  { name: "job-board", path: "/job-board", maxDiffRatio: 0.004 },
  { name: "english-home", path: "/en", maxDiffRatio: 0.004 },
  { name: "english-about", path: "/en/about" },
  { name: "english-news", path: "/en/news/2026-05-06-social-gathering-ticket" },
  { name: "english-job-board", path: "/en/job-board", maxDiffRatio: 0.004 },
  {
    name: "language-switch",
    path: "/",
    maxDiffRatio: 0.004,
    action: async (page) => {
      await page.getByRole("button", { name: "EN" }).first().click();
      await expect(page).toHaveURL(/\/en(?:\/)?$/);
      await expect(page.getByRole("button", { name: "EN" }).first()).toHaveAttribute(
        "aria-pressed",
        "true",
      );
    },
  },
];

test.describe("frontend-phpcon-do-website visual parity", () => {
  test.describe.configure({ mode: "serial", timeout: FRONTEND_PHPCON_VRT_TIMEOUT });

  for (const mode of modes) {
    test.describe(mode, () => {
      const apps = createFrontendPhpconVisualParityApps(mode);
      const servers: Array<ReturnType<typeof startDevServer>> = [];

      test.beforeAll(async () => {
        test.setTimeout(FRONTEND_PHPCON_VRT_TIMEOUT);
        servers.push(await startApp(apps.reference));
        servers.push(await startApp(apps.candidate));
      });

      test.afterAll(async () => {
        for (const server of servers) {
          killProcess(server);
        }
      });

      for (const route of routes) {
        test(route.name, async ({ browser }) => {
          await compareRoute(browser, apps, mode, route);
        });
      }
    });
  }
});

async function startApp(app: AppConfig): Promise<ReturnType<typeof startDevServer>> {
  if (app.setup) app.setup();
  await ensurePortFree(app.port);

  const server = startDevServer(app);
  await waitForServerReady(server, app.port, app.readyPattern, app.startupTimeout, app.readyDelay);
  await waitForHttpReady(app.url, app.port);
  return server;
}

async function compareRoute(
  browser: Browser,
  apps: ReturnType<typeof createFrontendPhpconVisualParityApps>,
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
    await Promise.all([
      openRoute(referencePage, apps.reference.url, route),
      openRoute(candidatePage, apps.candidate.url, route),
    ]);

    if (route.action) {
      await Promise.all([route.action(referencePage), route.action(candidatePage)]);
    }

    await Promise.all([
      prepareFrontendPhpconVisualState(referencePage),
      prepareFrontendPhpconVisualState(candidatePage),
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
    localStorage.setItem("nuxt-color-mode", "light");
  });
}

async function prepareFrontendPhpconVisualState(page: Page): Promise<void> {
  await page.evaluate(async () => {
    const images = Array.from(document.images);
    for (const image of images) {
      image.loading = "eager";
    }

    await Promise.all(
      images.map((image) =>
        image.complete
          ? Promise.resolve()
          : new Promise<void>((resolve) => {
              image.addEventListener("load", () => resolve(), { once: true });
              image.addEventListener("error", () => resolve(), { once: true });
            }),
      ),
    );

    await Promise.all(images.map((image) => image.decode?.().catch(() => undefined)));
  });
  await prepareStableVisualState(page);
}

async function openMobileMenu(page: Page): Promise<void> {
  const menu = page.locator("#mobile-menu");
  const button = page.locator('button[aria-controls="mobile-menu"]');
  await expect(button).toBeVisible({ timeout: 10_000 });

  for (let attempt = 0; attempt < 3; attempt += 1) {
    await button.click();
    try {
      await expect(menu).toBeVisible({ timeout: 3_000 });
      return;
    } catch (error) {
      if (attempt === 2) {
        throw error;
      }
      await page.waitForTimeout(500);
    }
  }
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
