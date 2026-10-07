import assert from "node:assert/strict";
import { mkdir, readFile, stat, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import path from "node:path";
import { parseArgs } from "node:util";
import { chromium } from "playwright";
import { resolvePuppeteerExecutablePath } from "../browser-path.js";

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
];
const routes = values.routes?.split(",") ?? pages.flatMap((route) => [route, `/ja${route}`]);
const types = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".svg": "image/svg+xml",
  ".png": "image/png",
};
const server = createServer(async (request, response) => {
  try {
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
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const origin = `http://127.0.0.1:${server.address().port}`;
const reports = [];
let browser;
try {
  browser = await chromium.launch({
    executablePath: resolvePuppeteerExecutablePath(),
    headless: true,
  });
  for (const { name: device, viewport } of [
    { name: "desktop", viewport: { width: 1440, height: 960 } },
    { name: "mobile", viewport: { width: 390, height: 844 } },
  ]) {
    for (const route of routes) {
      const page = await browser.newPage({ viewport, reducedMotion: "reduce" });
      const pageErrors = [];
      page.on("pageerror", (error) => pageErrors.push(String(error)));
      const url = `${origin}${route.replace(/\/$/, "")}/index.html`;
      const response = await page.goto(url, { waitUntil: "networkidle" });
      assert.equal(response.status(), 200, route);
      await page.locator("h1").first().waitFor();
      await page.evaluate(() => document.fonts.ready);
      let japaneseFonts = [];
      if (route === "/ja/" || route.startsWith("/ja/")) {
        const sample = await page.evaluate(() => {
          const walker = document.createTreeWalker(
            document.querySelector(".content") ?? document.body,
            NodeFilter.SHOW_TEXT,
          );
          for (let node = walker.nextNode(); node; node = walker.nextNode()) {
            if (/[\p{Script=Hiragana}\p{Script=Katakana}\p{Script=Han}]/u.test(node.textContent)) {
              const element = node.parentElement;
              if (element.checkVisibility()) {
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
        ({ fonts: japaneseFonts } = await session.send("CSS.getPlatformFontsForNode", { nodeId }));
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
      const metrics = await page.evaluate(() => ({
        title: document.title,
        viewport: innerWidth,
        bodyWidth: document.documentElement.scrollWidth,
        goals: [...document.querySelectorAll(".feature-title")].map(
          (element) => element.textContent,
        ),
        groups: [...document.querySelectorAll(".sidebar .nav-section")].map((element) => ({
          title: element.querySelector("summary")?.textContent,
          open: element.open,
        })),
        tables: [...document.querySelectorAll(".content table")].map((element) => ({
          width: element.clientWidth,
          scrollWidth: element.scrollWidth,
          overflow: getComputedStyle(element).overflowX,
        })),
        links: [...document.querySelectorAll(".content a[href], .feature-card[href]")].map(
          (element) => element.href,
        ),
      }));
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
          assert.equal(await sidebar.isVisible(), false);
        } else {
          const collapsed = page.locator(".sidebar details:not([open]) > summary").first();
          if (await collapsed.count()) {
            const summary = await collapsed.elementHandle();
            await collapsed.focus();
            await page.keyboard.press("Enter");
            await page.waitForFunction((element) => element.parentElement.open, summary);
            assert.equal(await summary.evaluate((element) => element.parentElement.open), true);
            await page.keyboard.press("Enter");
            await page.waitForFunction((element) => !element.parentElement.open, summary);
            assert.equal(await summary.evaluate((element) => element.parentElement.open), false);
          }
        }
      }
      for (const href of new Set(metrics.links)) {
        const target = new URL(href);
        if (target.origin !== origin) continue;
        const linked = await page.request.get(`${target.origin}${target.pathname}`);
        assert.equal(linked.status(), 200, `${route}: broken link ${href}`);
        if (target.hash) {
          const html = await linked.text();
          const id = decodeURIComponent(target.hash.slice(1));
          assert(
            html.includes(`id="${id}"`) || html.includes(`name="${id}"`),
            `${route}: missing anchor ${href}`,
          );
        }
      }
      const name = route.replace(/^\//, "").replaceAll("/", "-") || "home";
      const screenshot = `${name}-${device}.png`;
      await page.screenshot({ path: path.join(output, screenshot), fullPage: true });
      reports.push({ route, device, screenshot, pageErrors, japaneseFonts, ...metrics });
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
  await new Promise((resolve) => server.close(resolve));
}
