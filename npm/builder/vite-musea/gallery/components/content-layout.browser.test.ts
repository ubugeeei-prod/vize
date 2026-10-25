import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Page } from "playwright";
import {
  basePath,
  createFixtureServer,
  prepareContext,
} from "./global-toolbar.browser-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const output =
  process.env.MUSEA_CONTENT_PROOF_DIR || path.join(repository, "artifacts/musea-content-layout");

async function geometry(page: Page) {
  return page.evaluate(() => {
    const content = document.querySelector(".content")!;
    const grid = document.querySelector(".variants-view")!;
    const section = document.querySelector(".variant-section")!.getBoundingClientRect();
    const preview = document.querySelector(".variant-preview")!;
    const frame = preview.querySelector("iframe")!;
    const bounds = preview.getBoundingClientRect();
    const toc = document.querySelector(".variant-toc-column")!.getBoundingClientRect();
    return {
      viewport: [innerWidth, innerHeight],
      documentWidth: document.documentElement.scrollWidth,
      content: [content.clientWidth, content.scrollWidth],
      columns: getComputedStyle(grid).gridTemplateColumns.split(" "),
      gridWidth: grid.clientWidth,
      preview: [preview.clientWidth, preview.clientHeight],
      frame: [frame.clientWidth, frame.clientHeight],
      sectionTop: section.top,
      previewRight: bounds.right,
      toc: { left: toc.left, bottom: toc.bottom },
    };
  });
}

await test(
  "component previews follow available gallery width and keep every tab reachable",
  { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1", timeout: 90_000 },
  async () => {
    const server = await createFixtureServer(output);
    const browser = await chromium.launch();
    const observations: unknown[] = [];
    const errors: string[] = [];
    try {
      await mkdir(output, { recursive: true });
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
      await prepareContext(context);
      const page = await context.newPage();
      page.on("pageerror", (error) => errors.push(String(error)));
      await page.goto(`http://127.0.0.1:${address.port}${basePath}/component/First.art.vue`);
      await page
        .locator(".variant-card")
        .first()
        .frameLocator("iframe")
        .getByRole("status")
        .waitFor();
      async function observe(phase: string, width: number, height: number, columns: number) {
        await page.setViewportSize({ width, height });
        await page.waitForFunction(
          (value) => Boolean(document.querySelector(".sidebar-wrapper")) === value > 768,
          width,
        );
        const state = await geometry(page);
        observations.push({ phase, ...state });
        assert.equal(state.documentWidth, width, JSON.stringify(state));
        assert.equal(state.content[0], state.content[1], JSON.stringify(state));
        assert.equal(state.columns.length, columns, JSON.stringify(state));
        assert.ok(state.frame[0] > 0, JSON.stringify(state));
        assert.deepEqual(state.frame, state.preview, JSON.stringify(state));
        if (columns === 1) {
          assert.ok(state.preview[0] >= state.gridWidth - 3, JSON.stringify(state));
          assert.ok(state.toc.bottom <= state.sectionTop + 1, JSON.stringify(state));
        } else {
          assert.ok(state.previewRight <= state.toc.left, JSON.stringify(state));
        }
        const tabs = page.locator(".component-tabs .tab-btn");
        assert.equal(await tabs.count(), 5);
        for (const tab of await tabs.all()) {
          await tab.scrollIntoViewIfNeeded();
          const reachable = await tab.evaluate((button) => {
            const bounds = button.getBoundingClientRect();
            const strip = button.parentElement!.getBoundingClientRect();
            return bounds.left >= strip.left - 1 && bounds.right <= strip.right + 1;
          });
          assert.equal(reachable, true, `${phase}: ${await tab.textContent()}`);
        }
        await tabs.first().scrollIntoViewIfNeeded();
        await page.locator(".content").evaluate((content) => {
          content.scrollTop = 0;
        });
        await page.screenshot({ path: path.join(output, `${phase}.png`) });
      }
      await observe("wide", 1440, 900, 2);
      await observe("content-boundary", 1200, 900, 1);
      await observe("wide-boundary", 1201, 900, 2);
      await page.setViewportSize({ width: 1440, height: 900 });
      const handle = page.getByRole("button", {
        name: "Resize sidebar with left and right arrow keys",
        exact: true,
      });
      for (let index = 0; index < 20; index++) await handle.press("Shift+ArrowRight");
      await observe("wide-with-narrow-content", 1440, 900, 1);
      await observe("ipad-landscape", 1366, 1024, 1);
      await observe("ipad-portrait", 1024, 1366, 1);
      await observe("tablet-landscape", 1024, 768, 1);
      await observe("phone", 390, 844, 1);
      await observe("compact-boundary", 769, 1024, 1);
      await page.setViewportSize({ width: 1440, height: 900 });
      for (let index = 0; index < 20; index++) await handle.press("Shift+ArrowLeft");
      await observe("wide-restored", 1440, 900, 2);
      assert.deepEqual(errors, []);
    } finally {
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify({ observations, errors }, null, 2),
      );
      await browser.close();
      await server.close();
    }
  },
);
