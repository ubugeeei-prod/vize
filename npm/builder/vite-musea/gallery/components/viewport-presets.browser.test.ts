import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Page } from "playwright";
import { VIEWPORT_PRESETS } from "../composables/useAddons.ts";
import {
  basePath,
  createFixtureServer,
  prepareContext,
} from "./global-toolbar.browser-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const output =
  process.env.MUSEA_VIEWPORT_PROOF_DIR || path.join(repository, "artifacts/musea-viewports");

async function geometry(page: Page, end = false) {
  return page.evaluate((scrollToEnd) => {
    const frames = [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")];
    return {
      viewport: [innerWidth, innerHeight],
      documentWidth: document.documentElement.scrollWidth,
      frames: frames.map((frame) => {
        const preview = frame.parentElement!;
        preview.scrollLeft = scrollToEnd ? preview.scrollWidth : 0;
        preview.scrollTop = scrollToEnd ? preview.scrollHeight : 0;
        const area = preview.getBoundingClientRect();
        const rect = frame.getBoundingClientRect();
        return {
          viewport: [frame.contentWindow!.innerWidth, frame.contentWindow!.innerHeight],
          area: [preview.clientWidth, preview.clientHeight],
          scroll: [preview.scrollWidth, preview.scrollHeight],
          start: [rect.left - area.left, rect.top - area.top],
          end: [rect.right - area.right, rect.bottom - area.bottom],
        };
      }),
    };
  }, end);
}

async function checkFixedViewport(page: Page, width: number, height: number) {
  await page.locator(".variant-card").first().frameLocator("iframe").getByRole("status").waitFor();
  const start = await geometry(page);
  assert.equal(start.documentWidth, start.viewport[0], JSON.stringify(start));
  assert.equal(start.frames.length, 2);
  for (const frame of start.frames) {
    assert.deepEqual(frame.viewport, [width, height], JSON.stringify(start));
    assert.ok(
      frame.start.every((offset) => offset >= -1),
      JSON.stringify(start),
    );
    assert.ok(frame.scroll[0] >= width && frame.scroll[1] >= height, JSON.stringify(start));
  }
  const end = await geometry(page, true);
  for (const frame of end.frames) {
    assert.deepEqual(frame.viewport, [width, height], JSON.stringify(end));
    assert.ok(
      frame.end.every((offset) => offset <= 1),
      JSON.stringify(end),
    );
  }
  return { start, end };
}

await test(
  "device presets preserve responsive viewports and every scrollable edge at all gallery sizes",
  { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1", timeout: 120_000 },
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
      const context = await browser.newContext();
      await prepareContext(context);
      const page = await context.newPage();
      page.on("pageerror", (error) => errors.push(String(error)));
      await page.goto(`http://127.0.0.1:${address.port}${basePath}/component/First.art.vue`);
      await page.locator(".variant-card iframe").first().waitFor();
      for (const viewport of [
        { width: 1280, height: 720 },
        { width: 390, height: 844 },
        { width: 844, height: 390 },
        { width: 768, height: 1024 },
        { width: 1024, height: 768 },
      ]) {
        await page.setViewportSize(viewport);
        for (const preset of VIEWPORT_PRESETS.slice(1)) {
          await page.locator(".viewport-btn").click();
          await page
            .getByRole("button", {
              name: `${preset.name} ${preset.width} x ${preset.height}`,
              exact: true,
            })
            .click();
          const width = Number.parseInt(preset.width, 10);
          const height = Number.parseInt(preset.height, 10);
          observations.push({
            preset: preset.name,
            ...(await checkFixedViewport(page, width, height)),
          });
          await page.getByTitle("Rotate", { exact: true }).click();
          observations.push({
            preset: `${preset.name}:rotated`,
            ...(await checkFixedViewport(page, height, width)),
          });
          await page.getByTitle("Rotate", { exact: true }).click();
          await checkFixedViewport(page, width, height);
          if (preset.name === "iPad") {
            await page.getByRole("button", { name: "Props", exact: true }).click();
            await page.getByRole("button", { name: "Variants", exact: true }).click();
            observations.push({
              preset: `${preset.name}:tab-restored`,
              ...(await checkFixedViewport(page, width, height)),
            });
            await geometry(page);
            // Place the preview below the sticky toolbar for useful screenshots
            // even when the gallery itself has only a short landscape viewport.
            await page
              .locator(".variant-preview")
              .first()
              .evaluate((preview) => {
                const scroller = preview.closest(".content")!;
                const toolbar = document.querySelector(".component-sticky-menu")!;
                scroller.scrollTop +=
                  preview.getBoundingClientRect().top -
                  scroller.getBoundingClientRect().top -
                  toolbar.getBoundingClientRect().height -
                  12;
              });
            await page.screenshot({
              path: path.join(output, `${viewport.width}x${viewport.height}-ipad.png`),
            });
          }
        }
        await page.locator(".viewport-btn").click();
        await page.getByRole("button", { name: "Reset", exact: true }).last().click();
        const reset = await geometry(page);
        observations.push({ preset: "Reset", ...reset });
        assert.equal(reset.documentWidth, viewport.width);
        for (const frame of reset.frames) {
          assert.deepEqual(frame.viewport, frame.area, JSON.stringify(reset));
          assert.deepEqual(frame.start, [0, 0], JSON.stringify(reset));
        }
        assert.equal(await page.getByTitle("Rotate", { exact: true }).count(), 0);
      }
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
