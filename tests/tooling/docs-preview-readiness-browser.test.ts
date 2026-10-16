import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { createServer } from "node:http";
import { createRequire } from "node:module";
import path from "node:path";
import { setImmediate } from "node:timers/promises";
import { test } from "node:test";
import { resolvePuppeteerExecutablePath } from "../../docs/browser-path.js";
import { createPreviewReadinessWaiter } from "../../docs/previews/ui/wait-for-preview.ts";

const fromDocs = createRequire(new URL("../../docs/package.json", import.meta.url));
const { chromium } = fromDocs("playwright");
const fromPlaywright = createRequire(fromDocs.resolve("playwright/package.json"));

async function fontFixture(): Promise<Buffer> {
  // The pinned Playwright distribution already supplies a genuine portable
  // webfont. Discover its hashed name instead of copying a third-party binary
  // or depending on whichever fonts happen to be installed on the runner.
  const assets = path.join(
    path.dirname(fromPlaywright.resolve("playwright-core/package.json")),
    "lib/vite/recorder/assets",
  );
  const fonts = (await readdir(assets)).filter((name) => /^codicon.*\.ttf$/.test(name));
  assert.equal(fonts.length, 1, "Playwright must supply exactly one real codicon font fixture");
  const font = await readFile(path.join(assets, fonts[0]!));
  assert.equal(font.readUInt32BE(0), 0x00010000, "fixture is an actual TrueType font");
  return font;
}

function fixture(resource: string): string {
  const font = ["held", "missing-font", "corrupt-font"].includes(resource);
  const image = ["held", "missing-image", "corrupt-image"].includes(resource);
  return `<!doctype html><html><head><meta charset="utf-8"><title>Preview resources</title>
    ${
      font
        ? `<style>@font-face { font-family: Readiness; src: url('/${resource}.ttf') format('truetype'); }
      .font { font-family: Readiness; font-size: 32px; }</style>`
        : ""
    }
    ${resource === "missing-css" ? '<link rel="stylesheet" href="/missing.css">' : ""}
    ${resource === "missing-js" ? '<script defer src="/missing.js"></script>' : ""}
    </head><body><div class="preview-shell"><section class="preview-stage">
    <span class="font">&#xea60;</span><span hidden>Hidden preview content</span>
    ${image ? `<img src="/${resource}.svg" alt="Resource fixture">` : ""}
    <output>Ready for interaction</output></section></div></body></html>`;
}

test(
  "preview readiness waits for actual resources and reports resource failures",
  {
    timeout: 45_000,
  },
  async (t) => {
    const font = await fontFixture();
    const imageArrived = Promise.withResolvers<void>();
    const fontArrived = Promise.withResolvers<void>();
    const releaseImage = Promise.withResolvers<void>();
    const releaseFont = Promise.withResolvers<void>();
    const server = createServer(async (request, response) => {
      const resource = new URL(request.url!, "http://localhost").pathname;
      if (resource === "/held.svg") {
        imageArrived.resolve();
        await releaseImage.promise;
        response.writeHead(200, { "Content-Type": "image/svg+xml" });
        response.end(
          '<svg xmlns="http://www.w3.org/2000/svg" width="37" height="19"><rect width="37" height="19" fill="blue"/></svg>',
        );
      } else if (resource === "/held.ttf") {
        fontArrived.resolve();
        await releaseFont.promise;
        response.writeHead(200, { "Content-Type": "font/ttf" });
        response.end(font);
      } else if (resource === "/corrupt-image.svg" || resource === "/corrupt-font.ttf") {
        response.writeHead(200, {
          "Content-Type": resource.endsWith(".svg") ? "image/svg+xml" : "font/ttf",
        });
        response.end("This response is not a decodable image or font.");
      } else if (resource.includes(".") || resource === "/favicon.ico") {
        response.writeHead(404);
        response.end("Missing resource");
      } else {
        response.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
        response.end(fixture(resource.slice(1)));
      }
    });
    const browser = await chromium.launch({ executablePath: resolvePuppeteerExecutablePath() });
    try {
      await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
      const address = server.address();
      assert(address && typeof address !== "string");
      const origin = `http://127.0.0.1:${address.port}`;
      await t.test(
        "held image and font settle before geometry without advancing the frozen clock",
        async () => {
          const page = await browser.newPage();
          try {
            page.setDefaultTimeout(5_000);
            page.setDefaultNavigationTimeout(5_000);
            await page.clock.install({ time: 0 });
            await page.clock.pauseAt(1000);
            const waitForResources = createPreviewReadinessWaiter(page);
            await page.goto(`${origin}/held`, { waitUntil: "domcontentloaded" });
            await Promise.all([imageArrived.promise, fontArrived.promise]);
            let settled = false;
            const waiting = waitForResources().finally(() => {
              settled = true;
            });
            void waiting.catch(() => {});
            // The server has received both real requests, but neither response can
            // complete. A browser round trip and host turn give an early-returning
            // waiter a chance to expose itself without a timing-based sleep.
            assert.deepEqual(
              await page.evaluate(() => ({
                imageComplete: document.images[0]!.complete,
                fontStatus: document.fonts.status,
              })),
              { imageComplete: false, fontStatus: "loading" },
            );
            await setImmediate();
            assert.equal(settled, false, "both real responses are still held");

            const imageResponse = page.waitForResponse(`${origin}/held.svg`);
            releaseImage.resolve();
            await (await imageResponse).finished();
            await page.evaluate(() => document.images[0]!.decode());
            await setImmediate();
            assert.equal(
              settled,
              false,
              "the webfont response remains held after image completion",
            );

            releaseFont.resolve();
            await waiting;
            const actual = await page.evaluate(() => ({
              width: document.images[0]!.naturalWidth,
              height: document.images[0]!.naturalHeight,
              fonts: [...document.fonts].map((face) => ({
                family: face.family,
                status: face.status,
              })),
              now: Date.now(),
              hidden: document.querySelector("[hidden]")!.getBoundingClientRect().width,
            }));
            assert.deepEqual(actual, {
              width: 37,
              height: 19,
              fonts: [{ family: "Readiness", status: "loaded" }],
              now: 1000,
              hidden: 0,
            });
          } finally {
            releaseImage.resolve();
            releaseFont.resolve();
            await page.close();
          }
        },
      );

      for (const resource of [
        "missing-image",
        "corrupt-image",
        "missing-font",
        "corrupt-font",
        "missing-css",
        "missing-js",
      ]) {
        await t.test(`${resource} rejects rather than accepting fallback pixels`, async () => {
          const page = await browser.newPage();
          try {
            page.setDefaultTimeout(5_000);
            page.setDefaultNavigationTimeout(5_000);
            await page.clock.install({ time: 0 });
            await page.clock.pauseAt(1000);
            const waitForResources = createPreviewReadinessWaiter(page);
            await page.goto(`${origin}/${resource}`, { waitUntil: "domcontentloaded" });
            await assert.rejects(waitForResources(), (error: Error) => {
              assert.notEqual(
                error.name,
                "TimeoutError",
                "a stuck waiter is not a resource rejection",
              );
              assert.doesNotMatch(error.message, /did not finish|timed out|timeout/i);
              assert.match(error.message, /preview.*(?:resource|font)|image.*decod/i);
              return true;
            });
          } finally {
            await page.close();
          }
        });
      }
    } finally {
      releaseImage.resolve();
      releaseFont.resolve();
      await browser.close();
      if (server.listening) {
        server.closeAllConnections();
        await new Promise<void>((resolve, reject) =>
          server.close((error) => (error ? reject(error) : resolve())),
        );
      }
    }
  },
);
