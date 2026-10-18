/** Verify generated documentation routes and their real preview assets. */
import assert from "node:assert/strict";
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import path from "node:path";
import { parseArgs } from "node:util";
import type { AddressInfo } from "node:net";

import { verifyTocPointerLayer } from "../../scripts/toc-render-assertions.ts";
import { resolvePuppeteerExecutablePath } from "../../browser-path.js";
import { captureComposablePreviews } from "./capture-composables.ts";
import { previewComposableExamples } from "./build-config.ts";
import { exerciseEmbeddedClipboard } from "./interactions-observed-browser.ts";
import { japaneseComposableGoals } from "../../../npm/ui/scripts/reference-docs/composable-examples.ts";
import { verifyScreenshotResponse } from "./screenshot-file.ts";

const docsRoot = path.resolve(import.meta.dirname, "../..");
const root = path.join(docsRoot, "dist");
const { values } = parseArgs({
  options: {
    site: { type: "string" },
    output: {
      type: "string",
      default: path.resolve(docsRoot, "../docs-render-evidence/component-previews"),
    },
  },
});
mkdirSync(path.resolve(values.output), { recursive: true });
const require = createRequire(path.join(docsRoot, "package.json"));
const mime: Record<string, string> = {
  ".html": "text/html",
  ".js": "text/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
};
const server = createServer((request, response) => {
  const url = new URL(request.url!, "http://localhost");
  let file = path.resolve(root, `.${decodeURIComponent(url.pathname)}`);
  if (!file.startsWith(`${root}${path.sep}`) && file !== root) {
    response.writeHead(404).end();
    return;
  }
  if (existsSync(file) && statSync(file).isDirectory()) {
    if (!url.pathname.endsWith("/")) {
      response.writeHead(301, { location: `${url.pathname}/${url.search}` }).end();
      return;
    }
    file = path.join(file, "index.html");
  }
  if (!existsSync(file) || !statSync(file).isFile()) {
    response.writeHead(404).end();
    return;
  }
  response.writeHead(200, {
    "content-type": mime[path.extname(file)] ?? "application/octet-stream",
  });
  response.end(readFileSync(file));
});
await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
const { chromium } = require("playwright") as typeof import("playwright");
const browser = await chromium.launch({ executablePath: resolvePuppeteerExecutablePath() });
const tocOutput = path.join(path.resolve(values.output), "toc-pointer");
const tocReceipts: {
  width: number;
  controls: Awaited<ReturnType<typeof verifyTocPointerLayer>>;
}[] = [];
try {
  const base =
    values.site?.replace(/\/$/, "") ?? `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
  const checkedLinks = new Set<string>();
  for (const width of [1440, 390]) {
    const page = await browser.newPage({
      viewport: { width, height: 1000 },
      reducedMotion: "reduce",
    });
    try {
      for (const route of [
        "/guide/ui/",
        "/ja/guide/ui/",
        "/ja/getting-started/",
        "/ja/guide/compiler-inspector/",
        "/ja/guide/testing/",
        "/guide/ui/button/",
        "/guide/ui/dialog/",
        "/guide/ui-styles/",
        "/ja/guide/ui-styles/",
        "/guide/musea/",
        "/ja/guide/musea/",
        "/guide/composables/",
        "/ja/guide/composables/",
        ...previewComposableExamples.map((example) => `/guide/composables/${example.name}/`),
      ]) {
        if (route === "/guide/composables/use-clipboard/")
          await page.context().grantPermissions(["clipboard-read", "clipboard-write"], {
            origin: new URL(base).origin,
          });
        const response = await page.goto(`${base}${route}`, { waitUntil: "domcontentloaded" });
        assert.equal(response!.status(), 200, route);
        assert.ok(await page.locator("h1").first().textContent(), `${route}: heading`);
        if (route === "/ja/getting-started/")
          tocReceipts.push({
            width,
            controls: await verifyTocPointerLayer(
              page,
              width === 390 ? "mobile" : "desktop",
              tocOutput,
            ),
          });
        if (route === "/ja/guide/compiler-inspector/" || route === "/ja/guide/testing/") {
          await page.evaluate(() => document.fonts.ready);
          await page.screenshot({
            path: path.join(values.output, `ja-${route.split("/").at(-2)}-${width}.png`),
            fullPage: true,
            animations: "disabled",
          });
        }
        if (route === "/ja/guide/composables/") {
          for (const [name, goal] of Object.entries(japaneseComposableGoals)) {
            const link = page.locator("main").getByRole("link", { name: goal, exact: true });
            assert.equal(await link.count(), 1, `${width}px: Japanese goal ${name}`);
            const target = new URL((await link.getAttribute("href"))!, page.url());
            assert.equal(
              target.pathname.replace(/\/index\.html$/, "/"),
              `/guide/composables/${name}/`,
              name,
            );
          }
          await page.screenshot({
            path: path.join(values.output, `ja-composable-goals-${width}.png`),
          });
          const lastGoal = page.locator("main").getByRole("link", {
            name: japaneseComposableGoals["use-builtin-ai"],
            exact: true,
          });
          await lastGoal.evaluate((link) => (link as HTMLElement).focus({ preventScroll: true }));
          assert.equal(await lastGoal.evaluate((link) => link === document.activeElement), true);
          await lastGoal.evaluate((link) =>
            link.scrollIntoView({ block: "center", behavior: "instant" }),
          );
          await lastGoal.evaluate(async (link) => {
            const deadline = performance.now() + 10_000;
            let previous = "";
            let stableFrames = 0;
            while (performance.now() < deadline) {
              const rect = link.getBoundingClientRect();
              const sample = JSON.stringify([rect.x, rect.y, rect.width, rect.height, scrollY]);
              const inside =
                rect.width > 0 && rect.height > 0 && rect.top >= 0 && rect.bottom <= innerHeight;
              stableFrames = inside && sample === previous ? stableFrames + 1 : 0;
              if (stableFrames >= 2) return;
              previous = sample;
              await new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
            }
            throw new Error(`Device goal did not settle inside the viewport: ${previous}`);
          });
          const bounds = await lastGoal.boundingBox();
          assert.ok(
            bounds && bounds.y >= 0 && bounds.y + bounds.height <= 1000,
            `${width}px: focused device goal must be inside the captured viewport`,
          );
          await page.screenshot({
            path: path.join(values.output, `ja-composable-device-goals-${width}.png`),
          });
        }
        const example = previewComposableExamples.find(
          (item) => route === `/guide/composables/${item.name}/`,
        );
        if (example != null) {
          const source = await page.locator("main pre code").allTextContents();
          assert.ok(
            source.some((packet) => packet.trim() === example.source.trim()),
            `${route}: displayed complete source equals compiled live SFC`,
          );
          const frameElement = page.locator(`iframe[src*="composable=${example.name}"]`);
          assert.equal(await frameElement.count(), 1, `${route}: live preview`);
          await frameElement.scrollIntoViewIfNeeded();
          const frame = page.frameLocator(`iframe[src*="composable=${example.name}"]`);
          await frame.locator(`html[data-preview-ready="${example.name}"]`).waitFor();
          assert.equal(
            await frame.locator("html").getAttribute("data-preview-source-sha256"),
            example.sourceSha256,
          );
          assert.equal(
            await frame.locator("html").getAttribute("data-preview-hydration-retained"),
            "true",
          );
          if (example.name === "use-clipboard") await exerciseEmbeddedClipboard(page, frame);
        }
        assert.equal(
          await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
          true,
          `${route}: ${width}px body overflow`,
        );
        for (const href of await page
          .locator("main a[href], main img[src], main iframe[src]")
          .evaluateAll((elements) =>
            elements.map(
              (element) =>
                (element as HTMLAnchorElement & HTMLImageElement).href ??
                (element as HTMLAnchorElement & HTMLImageElement).src,
            ),
          )) {
          const target = new URL(href, page.url());
          if (target.origin !== base || checkedLinks.has(target.href)) continue;
          target.hash = "";
          checkedLinks.add(target.href);
          const linked = await page.request.get(target.href);
          assert.equal(linked.status(), 200, `${route}: ${target.pathname}`);
          if (
            target.pathname.startsWith("/component-previews/") &&
            target.pathname.endsWith(".png")
          )
            verifyScreenshotResponse(target.pathname, await linked.body());
        }
        const frame = page.frameLocator('iframe[src*="family=button"]');
        if ((await page.locator('iframe[src*="family=button"]').count()) > 0) {
          await page.locator('iframe[src*="family=button"]').scrollIntoViewIfNeeded();
          await frame.getByRole("button", { name: "Save draft" }).click();
          assert.equal(
            await frame.locator("output").textContent(),
            "Saved 1 times",
            `${route}: live preview`,
          );
        }
        if (process.env.VIZE_DOCS_SCREENSHOT_DIR && route === "/guide/ui/") {
          await page.screenshot({
            path: path.join(process.env.VIZE_DOCS_SCREENSHOT_DIR, `ui-hub-${width}.png`),
            fullPage: false,
          });
        }
      }
    } finally {
      await page.close();
    }
  }
  assert.ok(checkedLinks.size > 0, "generated main content must contain local links");
  await captureComposablePreviews(
    browser,
    `${base}/component-previews/app/`,
    path.resolve(values.output),
  );
  process.stdout.write(
    `Verified ${13 + previewComposableExamples.length} documentation routes at desktop/mobile widths and ${checkedLinks.size} local links\n`,
  );
} finally {
  mkdirSync(tocOutput, { recursive: true });
  writeFileSync(
    path.join(tocOutput, "receipt.json"),
    JSON.stringify({ completed: tocReceipts.length === 2, controls: tocReceipts }, null, 2) + "\n",
  );
  await browser.close();
  await new Promise<void>((resolve, reject) =>
    server.close((error) => (error ? reject(error) : resolve())),
  );
}
