import { test, expect, type Browser, type Page } from "@playwright/test";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { createVuefesVisualParityApps, type AppConfig } from "../../_helpers/apps";
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
  viewport?: { height: number; width: number };
}

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const OUTPUT_DIR =
  process.env.VIZE_VUEFES_VRT_OUTPUT_DIR ??
  path.resolve(__dirname, "../../../__agent_only/vuefes-vrt/artifacts");
const DEFAULT_VIEWPORT = { width: 1280, height: 720 };
const MOBILE_VIEWPORT = { width: 390, height: 844 };
const apps = createVuefesVisualParityApps();

const routes: VisualRoute[] = [
  { name: "home", path: "/" },
  { name: "home-mobile", path: "/", viewport: MOBILE_VIEWPORT },
  { name: "home-en", path: "/en" },
  { name: "photo", path: "/photo" },
  { name: "timetable", path: "/timetable", maxDiffRatio: 0.004 },
  { name: "speaker", path: "/speaker" },
  { name: "speaker-detail", path: "/speaker/yyx990803" },
  { name: "event", path: "/event", maxDiffRatio: 0.007 },
  { name: "store", path: "/store" },
  { name: "ticket", path: "/ticket" },
  { name: "sponsors", path: "/sponsors" },
  { name: "sponsor-detail", path: "/sponsors/bengo4", maxDiffRatio: 0.052 },
  { name: "related-events", path: "/related-events" },
  { name: "privacy-policy", path: "/privacy-policy" },
  { name: "code-of-conduct", path: "/code-of-conduct" },
  { name: "tokusho", path: "/tokusho" },
  { name: "english-speaker-detail", path: "/en/speaker/danielroe" },
];

test.describe("vuefes-2025 visual parity", () => {
  test.describe.configure({ mode: "serial" });

  const servers: Array<ReturnType<typeof startDevServer>> = [];

  test.beforeAll(async () => {
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
      await compareRoute(browser, route);
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
    await Promise.all([
      openRoute(referencePage, apps.reference.url, route),
      openRoute(candidatePage, apps.candidate.url, route),
    ]);

    await Promise.all([prepareVuefesPage(referencePage), prepareVuefesPage(candidatePage)]);

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

async function prepareVuefesPage(page: Page): Promise<void> {
  await retryAfterReload(() => stabilizeVuefesTheme(page));
  await retryAfterReload(() => waitForVuefesLayoutReady(page));
  await retryAfterReload(() => loadVuefesLazyImages(page));
  await retryAfterReload(() => prepareStableVisualState(page));
  await retryAfterReload(() => waitForVuefesLayoutReady(page));
  await retryAfterReload(() => waitForVuefesImagesReady(page));
  await page.waitForTimeout(500);
}

async function retryAfterReload(operation: () => Promise<void>): Promise<void> {
  let lastError: unknown;

  for (let attempt = 0; attempt < 3; attempt++) {
    try {
      await operation();
      return;
    } catch (error) {
      lastError = error;
      if (!isNavigationRace(error)) {
        throw error;
      }
      await new Promise((resolve) => setTimeout(resolve, 500));
    }
  }

  throw lastError;
}

function isNavigationRace(error: unknown): boolean {
  const message = String(error);
  return (
    message.includes("Execution context was destroyed") ||
    message.includes("Target page, context or browser has been closed")
  );
}

async function stabilizeVuefesTheme(page: Page): Promise<void> {
  await page.evaluate(() => {
    document.body.classList.remove("theme-purple", "theme-orange", "theme-navy");
    document.body.classList.add("theme-primary");
  });
}

async function waitForVuefesLayoutReady(page: Page): Promise<void> {
  await expect
    .poll(
      () =>
        page.evaluate(() => {
          const header = document.querySelector(".header");
          const logo = document.querySelector(".logo-image");
          if (!(header instanceof HTMLElement) || !(logo instanceof Element)) {
            return "missing-shell";
          }

          const headerStyle = getComputedStyle(header);
          const logoRect = logo.getBoundingClientRect();
          const scrollHeight = document.documentElement.scrollHeight;

          if (headerStyle.position !== "sticky") {
            return `header-position:${headerStyle.position}`;
          }
          if (logoRect.width < 100 || logoRect.width > 400) {
            return `logo-width:${logoRect.width}`;
          }
          if (scrollHeight < window.innerHeight || scrollHeight > 40_000) {
            return `scroll-height:${scrollHeight}`;
          }

          return "ready";
        }),
      { timeout: 30_000 },
    )
    .toBe("ready");
}

async function loadVuefesLazyImages(page: Page): Promise<void> {
  await page.evaluate(async () => {
    for (const image of document.images) {
      image.loading = "eager";
    }

    const maxY = Math.max(document.documentElement.scrollHeight, document.body.scrollHeight);
    for (let y = 0; y <= maxY; y += Math.max(window.innerHeight, 600)) {
      window.scrollTo(0, y);
      await new Promise((resolve) => setTimeout(resolve, 50));
    }
    window.scrollTo(0, 0);
  });
  await waitForVuefesImagesReady(page);
}

async function waitForVuefesImagesReady(page: Page): Promise<void> {
  await page.evaluate(async () => {
    await Promise.all(
      Array.from(document.images, (image) => {
        if (image.complete && image.naturalWidth > 0) {
          return image.decode().catch(() => undefined);
        }

        return new Promise<void>((resolve) => {
          const done = () => resolve();
          image.addEventListener("load", done, { once: true });
          image.addEventListener("error", done, { once: true });
          setTimeout(done, 5000);
        });
      }),
    );
    window.scrollTo(0, 0);
  });
}
