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
import {
  DEFAULT_VIEWPORT,
  FRONTEND_PHPCON_VRT_TIMEOUT,
  frontendPhpconVisualModes,
  frontendPhpconVisualRoutes,
  maxDiffPixelsForFrontendPhpconMode,
  type FrontendPhpconVisualMode,
  type FrontendPhpconVisualRouteConfig,
} from "./frontend-phpcon-routes";

interface VisualRoute extends FrontendPhpconVisualRouteConfig {
  action?: (page: Page) => Promise<void>;
}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR =
  process.env.VIZE_FRONTEND_PHPCON_VRT_OUTPUT_DIR ??
  path.resolve(__dirname, "../../../.vize/artifacts/frontend-phpcon-vrt/artifacts");
const routes: VisualRoute[] = frontendPhpconVisualRoutes.map((route) => ({
  ...route,
  action: actionForRoute(route.name),
}));

test.describe("frontend-phpcon-do-website visual parity", () => {
  test.describe.configure({ mode: "serial", timeout: FRONTEND_PHPCON_VRT_TIMEOUT });

  for (const mode of frontendPhpconVisualModes) {
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
  mode: FrontendPhpconVisualMode,
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
      maxDiffPixels: maxDiffPixelsForFrontendPhpconMode(route, mode),
      maxDiffRatio: route.maxDiffRatio,
      name: `${mode}-${route.name}`,
      outputDir: OUTPUT_DIR,
    });
  } finally {
    await context.close();
  }
}

function actionForRoute(routeName: string): VisualRoute["action"] {
  if (routeName === "language-switch") {
    return async (page) => {
      await page.getByRole("button", { name: "EN" }).first().click();
      await expect(page).toHaveURL(/\/en(?:\/)?$/);
      await expect(page.getByRole("button", { name: "EN" }).first()).toHaveAttribute(
        "aria-pressed",
        "true",
      );
    };
  }

  if (routeName === "mobile-menu") {
    return async (page) => {
      await openMobileMenu(page);
      await page.waitForTimeout(1200);
    };
  }

  return undefined;
}

async function setupPage(page: Page): Promise<void> {
  await mockViteHmrSocket(page);
  await installVisualStabilityHooks(page);
  await page.addInitScript(() => {
    localStorage.setItem("nuxt-color-mode", "light");
  });
}

async function mockViteHmrSocket(page: Page): Promise<void> {
  // Visual parity only needs the initial render. Mocking HMR avoids the
  // fixture's duplicate WebSocket upgrade listeners crashing the dev server.
  await page.routeWebSocket(/\/_nuxt\//, () => {});
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
