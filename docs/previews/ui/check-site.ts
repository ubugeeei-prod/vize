/** Verify generated documentation routes and their real preview assets. */
import assert from "node:assert/strict";
import { existsSync, readFileSync, statSync } from "node:fs";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import path from "node:path";
import type { AddressInfo } from "node:net";

import { resolvePuppeteerExecutablePath } from "../../browser-path.js";

const docsRoot = path.resolve(import.meta.dirname, "../..");
const root = path.join(docsRoot, "dist");
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
try {
  const base = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
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
        "/guide/ui/button/",
        "/guide/ui/dialog/",
        "/guide/ui-styles/",
        "/ja/guide/ui-styles/",
        "/guide/musea/",
        "/ja/guide/musea/",
      ]) {
        const response = await page.goto(`${base}${route}`, { waitUntil: "domcontentloaded" });
        assert.equal(response!.status(), 200, route);
        assert.ok(await page.locator("h1").first().textContent(), `${route}: heading`);
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
  process.stdout.write(
    `Verified 8 generated component-documentation routes at desktop/mobile widths and ${checkedLinks.size} local links\n`,
  );
} finally {
  await browser.close();
  await new Promise<void>((resolve, reject) =>
    server.close((error) => (error ? reject(error) : resolve())),
  );
}
