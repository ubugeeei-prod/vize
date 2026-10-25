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
  process.env.MUSEA_SIDEBAR_PROOF_DIR || path.join(repository, "artifacts/musea-sidebar-rotation");

async function geometry(page: Page) {
  return page.evaluate(() => ({
    viewport: [innerWidth, innerHeight],
    documentWidth: document.documentElement.scrollWidth,
    sidebar: document.querySelector(".sidebar-wrapper")?.getBoundingClientRect().width ?? 0,
    content: document.querySelector(".content")!.getBoundingClientRect().width,
    stored: localStorage.getItem("musea-sidebar-width"),
    resizing: document.body.classList.contains("musea-sidebar-resizing"),
    cursor: document.body.style.cursor,
    selection: document.body.style.userSelect,
  }));
}

await test(
  "sidebar keeps gallery content visible across tablet rotations and resizes from its displayed width",
  { skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1", timeout: 60_000 },
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
      const context = await browser.newContext({ viewport: { width: 1366, height: 1024 } });
      await prepareContext(context);
      const page = await context.newPage();
      page.on("pageerror", (error) => errors.push(String(error)));
      await page.goto(`http://127.0.0.1:${address.port}${basePath}/`);
      const handle = page.getByRole("button", {
        name: "Resize sidebar with left and right arrow keys",
        exact: true,
      });
      await handle.focus();
      for (let index = 0; index < 20; index++) await handle.press("Shift+ArrowRight");
      assert.equal((await geometry(page)).sidebar, 1040);
      for (const viewport of [
        { width: 1366, height: 1024 },
        { width: 1024, height: 1366 },
        { width: 768, height: 1024 },
        { width: 1024, height: 768 },
        { width: 769, height: 1024 },
        { width: 1366, height: 1024 },
      ]) {
        await page.setViewportSize(viewport);
        await page.waitForFunction(
          (width) => Boolean(document.querySelector(".sidebar-wrapper")) === width > 768,
          viewport.width,
        );
        const state = await geometry(page);
        observations.push({ phase: "rotation", ...state });
        const sidebar = viewport.width <= 768 ? 0 : Math.min(1040, viewport.width - 320);
        assert.equal(state.sidebar, sidebar, JSON.stringify(state));
        assert.equal(state.content, viewport.width - sidebar, JSON.stringify(state));
        assert.equal(state.documentWidth, viewport.width, JSON.stringify(state));
        assert.equal(state.stored, "1040", "rotation preserves the user's preferred width");
        await page.screenshot({
          path: path.join(output, `${viewport.width}x${viewport.height}.png`),
        });
      }

      await page.setViewportSize({ width: 1024, height: 1366 });
      await page.getByTitle("Collapse sidebar (⌘B)", { exact: true }).click();
      assert.equal((await geometry(page)).sidebar, 40);
      await page.setViewportSize({ width: 1366, height: 1024 });
      assert.equal((await geometry(page)).sidebar, 40);
      await page.getByTitle("Expand sidebar (⌘B)", { exact: true }).click();
      assert.equal((await geometry(page)).sidebar, 1040);
      observations.push({ phase: "collapse-expand-preference", ...(await geometry(page)) });

      await page.setViewportSize({ width: 1024, height: 1366 });
      await handle.press("ArrowLeft");
      let state = await geometry(page);
      observations.push({ phase: "keyboard-after-clamp", ...state });
      assert.equal(state.sidebar, 694, JSON.stringify(state));
      assert.equal(state.stored, "694");
      await handle.press("Shift+ArrowRight");
      assert.equal((await geometry(page)).sidebar, 704);

      await page.setViewportSize({ width: 1366, height: 1024 });
      for (let index = 0; index < 20; index++) await handle.press("Shift+ArrowRight");
      await page.setViewportSize({ width: 1024, height: 1366 });
      const bounds = await handle.boundingBox();
      assert.ok(bounds);
      // The outer half of this handle is clipped by the sidebar's overflow.
      const x = bounds.x + 1;
      const y = bounds.y + bounds.height / 2;
      assert.equal(
        await page.evaluate(
          ([x, y]) => document.elementFromPoint(x, y)?.classList.contains("sidebar-resize-handle"),
          [x, y],
        ),
        true,
      );
      await page.mouse.move(x, y);
      await page.mouse.down();
      await page.mouse.move(x - 80, y);
      await page.mouse.up();
      state = await geometry(page);
      observations.push({ phase: "pointer-after-clamp", ...state });
      assert.equal(state.sidebar, 624, JSON.stringify(state));
      assert.equal(state.stored, "624");
      assert.equal(state.resizing, false);
      assert.equal(state.cursor, "");
      assert.equal(state.selection, "");
      await page.reload();
      await handle.waitFor();
      state = await geometry(page);
      observations.push({ phase: "reload", ...state });
      assert.equal(state.sidebar, 624, JSON.stringify(state));
      assert.equal(state.content, 400);
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
