import assert from "node:assert/strict";
import { mkdir, readFile, stat, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import path from "node:path";
import { parseArgs } from "node:util";
import { chromium } from "playwright";
import type { Browser } from "playwright";
import { resolvePuppeteerExecutablePath } from "../browser-path.js";
import { ruleRenderRoutes, verifyRenderedRulePackets } from "./rule-render-assertions.ts";
import { capturePageRender } from "./capture-page-render.ts";
import type { PageCapture } from "./capture-page-render.ts";
import { verifyCaptureRenderControls } from "./capture-render-controls.ts";
import {
  verifyIntroductoryNavigation,
  verifySidebarMotion,
  verifyThemeReadability,
} from "./theme-render-assertions.ts";

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
  },
});
const dist = path.resolve(values.dir);
const output = path.resolve(values.output);
const pages = [
  "/",
  "/getting-started",
  "/guide/configuration",
  "/guide/migration",
  "/guide/vite-plus",
  "/guide/vite-plugin",
  "/guide/configuration-reference",
  "/guide/compiler-configuration-reference",
  ...ruleRenderRoutes,
];
const routes = values.routes?.split(",") ?? [
  ...pages.flatMap((route) => [route, `/ja${route}`]),
  ...["", "/ja", "/zh-CN", "/pt-BR", "/fr"].flatMap((locale) => [
    `${locale}/philosophy`,
    `${locale}/guide/content-mapper`,
  ]),
];
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
await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
const address = server.address();
assert(address && typeof address !== "string", "Static server has no TCP address");
const origin = `http://127.0.0.1:${address.port}`;
const reports: RouteReceipt[] = [];
const linkedPages = new Map<string, string>();
let browser: Browser | undefined;
try {
  browser = await chromium.launch({
    executablePath: resolvePuppeteerExecutablePath(),
    headless: true,
  });
  await verifyCaptureRenderControls(browser, origin, output);
  for (const { name: device, viewport } of [
    { name: "desktop", viewport: { width: 1440, height: 960 } },
    { name: "mobile", viewport: { width: 390, height: 844 } },
  ]) {
    for (const route of routes) {
      const page = await browser.newPage({ viewport, reducedMotion: "reduce" });
      const pageErrors: string[] = [];
      page.on("pageerror", (error) => pageErrors.push(String(error)));
      const url = `${origin}${route.replace(/\/$/, "")}/index.html`;
      const response = await page.goto(url, { waitUntil: "networkidle" });
      assert(response, `${route}: navigation returned no response`);
      assert.equal(response.status(), 200, route);
      await page.locator("h1").first().waitFor();
      await page.evaluate(() => document.fonts.ready);
      await page.waitForFunction(() =>
        [...document.images].every(
          (image) =>
            new URL(image.src).origin !== location.origin ||
            (image.complete && image.naturalWidth > 0),
        ),
      );
      let japaneseFonts: FontUsage[] = [];
      if (route === "/ja/" || route.startsWith("/ja/")) {
        const sample = await page.evaluate(() => {
          const walker = document.createTreeWalker(
            document.querySelector(".content") ?? document.body,
            NodeFilter.SHOW_TEXT,
          );
          for (let node = walker.nextNode(); node; node = walker.nextNode()) {
            if (
              /[\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Han}]/u.test(node.textContent ?? "")
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
        const fontPacket: unknown = await session.send("CSS.getPlatformFontsForNode", { nodeId });
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
      const metrics = await page.evaluate(readPageMetrics);
      await verifyIntroductoryNavigation(page, route);
      await verifySidebarMotion(page, device);
      const codeContrast = await verifyThemeReadability(page);
      const rulePackets = await verifyRenderedRulePackets(page, route);
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
                element.parentElement instanceof HTMLDetailsElement && element.parentElement.open,
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
                element.parentElement instanceof HTMLDetailsElement && !element.parentElement.open,
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
      for (const href of new Set(metrics.links)) {
        const target = new URL(href);
        if (target.origin !== origin) continue;
        const linkedUrl = `${target.origin}${target.pathname}`;
        if (!linkedPages.has(linkedUrl)) {
          const linked = await page.request.get(linkedUrl);
          assert.equal(linked.status(), 200, `${route}: broken link ${href}`);
          linkedPages.set(linkedUrl, await linked.text());
        }
        if (target.hash) {
          const html = linkedPages.get(linkedUrl);
          assert(html !== undefined, `${route}: missing captured linked page ${linkedUrl}`);
          const id = decodeURIComponent(target.hash.slice(1));
          assert(
            html.includes(`id="${id}"`) || html.includes(`name="${id}"`),
            `${route}: missing anchor ${href}`,
          );
        }
      }
      const name = route.replace(/^\//, "").replaceAll("/", "-") || "home";
      const screenshot = `${name}-${device}.png`;
      const initialTheme = await page.locator("html").getAttribute("data-theme");
      const capture = await capturePageRender(page, output, screenshot);
      const themeScreenshots: RouteReceipt["themeScreenshots"] = [];
      if (
        /^\/(?:ja\/)?rules\//.test(route) ||
        /\/(?:guide\/content-mapper|getting-started)$/.test(route) ||
        ["/", "/ja/"].includes(route)
      ) {
        const controlSelector = device === "mobile" ? "[data-mobile-theme]" : ".theme-toggle";
        for (const theme of ["light", "dark"]) {
          if ((await page.locator("html").getAttribute("data-theme")) !== theme)
            await page.locator(controlSelector).click();
          assert.equal(await page.locator("html").getAttribute("data-theme"), theme);
          assert.equal(await page.evaluate(() => localStorage.getItem("theme")), theme);
          const codeContrast = await verifyThemeReadability(page);
          const file = `${name}-${device}-${theme}.png`;
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
      reports.push({
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
      });
      await page.close();
    }
  }
  console.log(
    `Verified ${reports.length} real rendered pages; desktop/mobile screenshots in ${output}`,
  );
} finally {
  await writeFile(
    path.join(output, "receipt.json"),
    JSON.stringify({ dist, reports }, null, 2) + "\n",
  );
  await browser?.close();
  await new Promise<void>((resolve, reject) =>
    server.close((error) => (error ? reject(error) : resolve())),
  );
}
