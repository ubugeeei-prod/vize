import assert from "node:assert/strict";
import { mkdir, readFile, stat, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import path from "node:path";
import { parseArgs } from "node:util";
import { chromium } from "playwright";
import type { Browser } from "playwright";
import { resolvePuppeteerExecutablePath } from "../browser-path.js";
import { verifyRenderedRulePackets } from "./rule-render-assertions.ts";
import { navigationRenderRoutes } from "./navigation-render-routes.ts";
import { capturePageRender } from "./capture-page-render.ts";
import type { PageCapture } from "./capture-page-render.ts";
import { verifyCaptureRenderControls } from "./capture-render-controls.ts";
import {
  verifyIntroductoryNavigation,
  verifySidebarMotion,
  verifyThemeReadability,
  switchDocsTheme,
} from "./theme-render-assertions.ts";
import { verifyCommandTabs } from "./command-tab-render-assertions.ts";
import { verifyLocaleRenderControls } from "./locale-control-render.ts";
import {
  collectRenderReceipts,
  createRenderJobs,
  measureRenderPhase,
  parseRenderWorkers,
  renderError,
  runRenderJobs,
  withRenderContext,
} from "./navigation-render-concurrency.ts";
import type { RenderJobResult, RenderPhaseTiming } from "./navigation-render-concurrency.ts";

type FontUsage = { familyName: string; glyphCount: number };
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
function isFontUsage(value: unknown): value is FontUsage {
  return (
    isRecord(value) && typeof value.familyName === "string" && typeof value.glyphCount === "number"
  );
}
function readPageMetrics() {
  return {
    title: document.title,
    viewport: innerWidth,
    bodyWidth: document.documentElement.scrollWidth,
    images: [...document.images]
      .filter((image) => new URL(image.src).origin === location.origin)
      .map((image) => ({ src: image.src, naturalWidth: image.naturalWidth })),
    goals: [...document.querySelectorAll(".feature-title")].map((element) => element.textContent),
    groups: [...document.querySelectorAll(".sidebar .nav-section")].map((element) => {
      if (!(element instanceof HTMLDetailsElement))
        throw new Error("Navigation section is not details");
      return { title: element.querySelector("summary")?.textContent, open: element.open };
    }),
    tables: [...document.querySelectorAll(".content table")].map((element) => ({
      width: element.clientWidth,
      scrollWidth: element.scrollWidth,
      overflow: getComputedStyle(element).overflowX,
    })),
    links: [...document.querySelectorAll(".content a[href], .feature-card[href]")].map(
      (element) => {
        if (!(element instanceof HTMLAnchorElement))
          throw new Error("Navigation target is not an anchor");
        return element.href;
      },
    ),
  };
}
type RouteReceipt = ReturnType<typeof readPageMetrics> & {
  codeContrast: Awaited<ReturnType<typeof verifyThemeReadability>>;
  route: string;
  device: string;
  screenshot: string;
  capture: PageCapture;
  themeScreenshots: {
    theme: string;
    screenshot: string;
    capture: PageCapture;
    controlSelector: string;
    codeContrast: Awaited<ReturnType<typeof verifyThemeReadability>>;
  }[];
  pageErrors: string[];
  japaneseFonts: FontUsage[];
  rulePackets: Awaited<ReturnType<typeof verifyRenderedRulePackets>>;
};

const { values } = parseArgs({
  options: {
    dir: { type: "string", default: "docs/dist" },
    output: { type: "string", default: "docs-render-evidence" },
    routes: { type: "string" },
    workers: { type: "string" },
  },
});
const dist = path.resolve(values.dir);
const output = path.resolve(values.output);
const routes = values.routes?.split(",") ?? navigationRenderRoutes;
const workers = parseRenderWorkers(values.workers);
const jobs = createRenderJobs(routes);
const startedAt = new Date().toISOString();
const start = performance.now();
const phases: RenderPhaseTiming[] = [];
const types: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".svg": "image/svg+xml",
  ".png": "image/png",
};
const server = createServer(async (request, response) => {
  try {
    assert(request.url !== undefined, "Static request has no URL");
    const pathname = decodeURIComponent(new URL(request.url, "http://localhost").pathname);
    let file = path.resolve(dist, `.${pathname}`);
    if (!file.startsWith(`${dist}${path.sep}`) && file !== dist) throw new Error("Outside site");
    if ((await stat(file)).isDirectory()) file = path.join(file, "index.html");
    response.writeHead(200, {
      "content-type": types[path.extname(file)] ?? "application/octet-stream",
    });
    response.end(await readFile(file));
  } catch {
    response.writeHead(404);
    response.end("Not found");
  }
});
await mkdir(output, { recursive: true });
let browser: Browser | undefined;
let results: RenderJobResult<RouteReceipt>[] = [];
let reports: RouteReceipt[] = [];
let failure: unknown;
let failed = false;
const linkedPages = new Map<string, Promise<string>>();
try {
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  assert(address && typeof address !== "string", "Static server has no TCP address");
  const origin = `http://127.0.0.1:${address.port}`;
  const activeBrowser = await measureRenderPhase(phases, "browser-launch", () =>
    chromium.launch({
      executablePath: resolvePuppeteerExecutablePath(),
      headless: true,
    }),
  );
  browser = activeBrowser;
  await measureRenderPhase(phases, "capture-controls", () =>
    verifyCaptureRenderControls(activeBrowser, origin, output),
  );
  await verifyLocaleRenderControls(activeBrowser, origin, output);
  results = await runRenderJobs(jobs, workers, async (job, phase) =>
    withRenderContext(
      () =>
        phase("context", () =>
          activeBrowser.newContext({ viewport: job.viewport, reducedMotion: "reduce" }),
        ),
      async (context) => {
        const { route, device, viewport, screenshot } = job;
        const page = await context.newPage();
        const pageErrors: string[] = [];
        page.on("pageerror", (error) => pageErrors.push(String(error)));
        const url = `${origin}${route.replace(/\/$/, "")}/index.html`;
        await phase("navigation", async () => {
          const response = await page.goto(url, { waitUntil: "networkidle" });
          assert(response, `${route}: navigation returned no response`);
          assert.equal(response.status(), 200, route);
        });
        await phase("assets", async () => {
          await page.locator("h1").first().waitFor();
          await page.evaluate(() => document.fonts.ready);
          await page.waitForFunction(() =>
            [...document.images].every(
              (image) =>
                new URL(image.src).origin !== location.origin ||
                (image.complete && image.naturalWidth > 0),
            ),
          );
        });
        const japaneseFonts = await phase("japanese-fonts", async () => {
          let japaneseFonts: FontUsage[] = [];
          if (route === "/ja/" || route.startsWith("/ja/")) {
            const sample = await page.evaluate(() => {
              const walker = document.createTreeWalker(
                document.querySelector(".content") ?? document.body,
                NodeFilter.SHOW_TEXT,
              );
              for (let node = walker.nextNode(); node; node = walker.nextNode()) {
                if (
                  /[\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Han}]/u.test(
                    node.textContent ?? "",
                  )
                ) {
                  const element = node.parentElement;
                  if (element?.checkVisibility()) {
                    element.setAttribute("data-render-japanese-font", "");
                    return node.textContent;
                  }
                }
              }
              return null;
            });
            assert(sample, `${route}: no visible Japanese text`);
            const session = await page.context().newCDPSession(page);
            await session.send("DOM.enable");
            await session.send("CSS.enable");
            const { root } = await session.send("DOM.getDocument");
            const { nodeId } = await session.send("DOM.querySelector", {
              nodeId: root.nodeId,
              selector: "[data-render-japanese-font]",
            });
            const fontPacket: unknown = await session.send("CSS.getPlatformFontsForNode", {
              nodeId,
            });
            assert(
              isRecord(fontPacket) &&
                Array.isArray(fontPacket.fonts) &&
                fontPacket.fonts.every(isFontUsage),
              "Complete typed platform font packet",
            );
            japaneseFonts = fontPacket.fonts;
            assert(
              japaneseFonts.some((font) => font.glyphCount > 0),
              `${route}: no rendered glyphs`,
            );
            if (process.platform === "linux") {
              assert(
                japaneseFonts.some(
                  (font) => /Noto Sans CJK/.test(font.familyName) && font.glyphCount > 0,
                ),
                `${route}: Japanese text did not use installed CJK fonts`,
              );
            }
            await session.detach();
          }
          return japaneseFonts;
        });
        const metrics = await phase("metrics", () => page.evaluate(readPageMetrics));
        await phase("navigation-assertions", async () => {
          await verifyIntroductoryNavigation(page, route);
          await verifySidebarMotion(page, device);
        });
        const codeContrast = await phase("theme-readability", () => verifyThemeReadability(page));
        const rulePackets = await phase("rule-packets", () =>
          verifyRenderedRulePackets(page, route),
        );
        await phase("command-tabs", () => verifyCommandTabs(page, route));
        await phase("layout", async () => {
          assert(metrics.bodyWidth <= viewport.width + 1, `${route}: page overflows viewport`);
          assert.equal(pageErrors.length, 0, pageErrors.join("\n"));
          const sidebar = page.locator(".sidebar");
          if (await page.locator("body.entry-page").count()) {
            assert.equal(await sidebar.isVisible(), false, "Entry page sidebar must be hidden");
            const cards = page.locator(".feature-card");
            if (["/", "/ja", "/ja/"].includes(route)) {
              assert.equal(await cards.count(), 6, "English/Japanese entry pages expose six goals");
            }
            for (const card of await cards.all()) {
              assert.equal(await card.isVisible(), true);
              await card.click({ trial: true });
            }
            await page.evaluate(() => scrollTo(0, 0));
          } else if (await sidebar.count()) {
            if (device === "mobile") {
              assert.equal(await sidebar.isVisible(), false, "Closed menu must be hidden");
              await page.locator("[data-mobile-menu]").click();
              assert.equal(await sidebar.isVisible(), true);
              await page.locator("[data-mobile-menu]").click();
              await sidebar.waitFor({ state: "hidden" });
              assert.equal(await sidebar.isVisible(), false);
            } else {
              const collapsed = page.locator(".sidebar details:not([open]) > summary").first();
              if (await collapsed.count()) {
                const summary = await collapsed.elementHandle();
                assert(summary, "Collapsed navigation summary disappeared");
                await collapsed.focus();
                await page.keyboard.press("Enter");
                await page.waitForFunction(
                  (element) =>
                    element.parentElement instanceof HTMLDetailsElement &&
                    element.parentElement.open,
                  summary,
                );
                assert.equal(
                  await summary.evaluate((element) =>
                    element.parentElement instanceof HTMLDetailsElement
                      ? element.parentElement.open
                      : undefined,
                  ),
                  true,
                );
                await page.keyboard.press("Enter");
                await page.waitForFunction(
                  (element) =>
                    element.parentElement instanceof HTMLDetailsElement &&
                    !element.parentElement.open,
                  summary,
                );
                assert.equal(
                  await summary.evaluate((element) =>
                    element.parentElement instanceof HTMLDetailsElement
                      ? element.parentElement.open
                      : undefined,
                  ),
                  false,
                );
              }
            }
          }
        });
        await phase("links", async () => {
          for (const href of new Set(metrics.links)) {
            const target = new URL(href);
            if (target.origin !== origin) continue;
            const linkedUrl = `${target.origin}${target.pathname}`;
            if (!linkedPages.has(linkedUrl)) {
              linkedPages.set(
                linkedUrl,
                (async () => {
                  const linked = await page.request.get(linkedUrl);
                  assert.equal(linked.status(), 200, `${route}: broken link ${href}`);
                  return linked.text();
                })(),
              );
            }
            const html = await linkedPages.get(linkedUrl);
            assert(html !== undefined, `${route}: missing captured linked page ${linkedUrl}`);
            if (target.hash) {
              const id = decodeURIComponent(target.hash.slice(1));
              assert(
                html.includes(`id="${id}"`) || html.includes(`name="${id}"`),
                `${route}: missing anchor ${href}`,
              );
            }
          }
        });
        const initialTheme = await page.locator("html").getAttribute("data-theme");
        const capture = await phase("capture", () => capturePageRender(page, output, screenshot));
        const themeScreenshots = await phase("theme-captures", async () => {
          const themeScreenshots: RouteReceipt["themeScreenshots"] = [];
          if (
            /^\/(?:ja\/)?rules\//.test(route) ||
            /\/(?:guide\/content-mapper|getting-started)$/.test(route) ||
            ["/", "/ja/"].includes(route)
          ) {
            const controlSelector = device === "mobile" ? "[data-mobile-theme]" : ".theme-toggle";
            for (const theme of ["light", "dark"]) {
              await switchDocsTheme(page, theme, device);
              assert.equal(await page.locator("html").getAttribute("data-theme"), theme);
              assert.equal(await page.evaluate(() => localStorage.getItem("theme")), theme);
              const codeContrast = await verifyThemeReadability(page);
              const file = screenshot.replace(/\.png$/, `-${theme}.png`);
              const themeCapture =
                theme === initialTheme ? capture : await capturePageRender(page, output, file);
              themeScreenshots.push({
                theme,
                screenshot: theme === initialTheme ? screenshot : file,
                capture: themeCapture,
                controlSelector,
                codeContrast,
              });
            }
          }
          return themeScreenshots;
        });
        return {
          route,
          device,
          screenshot,
          capture,
          themeScreenshots,
          pageErrors,
          japaneseFonts,
          rulePackets,
          codeContrast,
          ...metrics,
        };
      },
      (close) => phase("context-close", close),
    ),
  );
  reports = collectRenderReceipts(jobs, results);
  const failures = results.filter(({ timing }) => timing.status === "failed");
  assert.equal(
    failures.length,
    0,
    failures.map(({ timing }) => `${timing.device} ${timing.route}: ${timing.error}`).join("\n"),
  );
  assert.equal(reports.length, jobs.length, "All requested route/device receipts must pass");
} catch (error) {
  failed = true;
  failure = error;
} finally {
  // Release browser/server even if assertion, capture or evidence writing fails.
  for (const [phase, close] of [
    [
      "browser-close",
      async () => {
        await browser?.close();
      },
    ],
    [
      "server-close",
      async () => {
        if (server.listening)
          await new Promise<void>((resolve, reject) =>
            server.close((error) => (error ? reject(error) : resolve())),
          );
      },
    ],
  ] as const) {
    try {
      await measureRenderPhase(phases, phase, close);
    } catch (error) {
      failure = failed
        ? new AggregateError([failure, error], "Render verification cleanup failed")
        : error;
      failed = true;
    }
  }
  const status = failed ? "failed" : "passed";
  const requestedJobs = jobs.map(({ index, route, device }) => ({ index, route, device }));
  await writeFile(
    path.join(output, "receipt.json"),
    JSON.stringify(
      { dist, status, workers, expectedCount: jobs.length, requestedJobs, reports },
      null,
      2,
    ) + "\n",
  );
  await writeFile(
    path.join(output, "timings.json"),
    JSON.stringify(
      {
        schemaVersion: 1,
        dist,
        workers,
        routes,
        requestedJobs,
        startedAt,
        durationMs: performance.now() - start,
        status,
        expectedCount: jobs.length,
        passedCount: reports.length,
        failedCount: results.filter(({ timing }) => timing.status === "failed").length,
        incompleteCount: jobs.length - results.length,
        phases,
        jobs: jobs.map(
          (job) =>
            results.find((result) => result.job.index === job.index)?.timing ?? {
              index: job.index,
              route: job.route,
              device: job.device,
              status: "pending",
              phases: [],
            },
        ),
        ...(failed ? { error: renderError(failure) } : {}),
      },
      null,
      2,
    ) + "\n",
  );
}
if (failed) throw failure;
console.log(
  `Verified ${reports.length} real rendered pages with ${workers} workers; desktop/mobile screenshots and timings in ${output}`,
);
