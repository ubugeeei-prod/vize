import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
import test from "node:test";
import { chromium, type Page } from "playwright";
import { createPropsPermalink, MUSEA_PROPS_QUERY } from "../utils/propsPermalink.ts";
import { getPaletteStateStorageKey } from "../composables/paletteState.ts";
import {
  art,
  base,
  defaults,
  createPermalinkServer,
  preparePermalinkContext,
} from "./props-permalink.browser-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const output =
  process.env.MUSEA_PROPS_PROOF_DIR || path.join(repository, "output/playwright/musea-props");

async function preview(page: Page, expected: unknown) {
  const text = JSON.stringify(expected);
  await page
    .frameLocator(".props-preview iframe")
    .locator("output")
    .filter({ hasText: text })
    .waitFor();
  const value = await page.frameLocator(".props-preview iframe").locator("output").textContent();
  assert.deepEqual(JSON.parse(value!), expected);
  return JSON.parse(value!);
}

await test(
  "props links reproduce edited controls and slots in an empty browser, reload and reset safely",
  {
    skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1",
  },
  async () => {
    const observations: unknown[] = [];
    const errors: string[] = [];
    const server = await createPermalinkServer(output);
    const browser = await chromium.launch();
    try {
      await mkdir(output, { recursive: true });
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      const origin = `http://127.0.0.1:${address.port}`;
      const context = await browser.newContext({
        permissions: ["clipboard-read", "clipboard-write"],
        viewport: { width: 1440, height: 1000 },
      });
      await preparePermalinkContext(context);
      const page = await context.newPage();
      let paletteRequests = 0;
      page.on("request", (request) => {
        if (new URL(request.url()).pathname.endsWith("/palette")) paletteRequests++;
      });
      page.on("pageerror", (error) => errors.push(String(error)));
      await page.goto(`${origin}${base}/?other=retained`);
      await page.locator(".art-item").first().click();
      await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
      await preview(page, { props: defaults, attrs: {} });
      await page
        .getByRole("textbox", { name: "label", exact: true })
        .fill("こんにちは 🎨 & # ? = + %");
      await page
        .locator(".props-control-row")
        .filter({ hasText: "tone" })
        .getByTitle("Remove prop", { exact: true })
        .click();
      await page.getByRole("button", { name: "Add Prop", exact: true }).click();
      await page.getByRole("textbox", { name: "Prop name", exact: true }).fill("data-state");
      await page.getByRole("textbox", { name: "Default value", exact: true }).fill("custom 🎨");
      await page.getByRole("button", { name: "Add", exact: true }).click();
      const expected = {
        props: { label: "こんにちは 🎨 & # ? = + %", config: defaults.config },
        attrs: { "data-state": "custom 🎨" },
      };
      observations.push({ phase: "edited", rendered: await preview(page, expected) });
      const requestsBeforeGlobals = paletteRequests;
      assert.equal(requestsBeforeGlobals, 1);
      await page.getByRole("combobox", { name: "Theme", exact: true }).selectOption("1");
      await page.waitForURL(
        (url) => url.searchParams.get("museaGlobals") === JSON.stringify({ theme: "dark" }),
      );
      await page.evaluate(
        () =>
          new Promise<void>((resolve) =>
            requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
          ),
      );
      assert.equal(
        paletteRequests,
        requestsBeforeGlobals,
        "unrelated query changes must not reload the palette or discard unsaved edits",
      );
      observations.push({
        phase: "globals-change",
        rendered: await preview(page, expected),
        paletteRequests,
      });
      await page.getByRole("button", { name: "Copy link", exact: true }).click();
      await page.getByRole("status").filter({ hasText: "Link copied" }).waitFor();
      const copied = await page.evaluate(() => navigator.clipboard.readText());
      const copiedUrl = new URL(copied);
      assert.equal(copiedUrl.pathname.startsWith(`${base}/component/`), true);
      assert.ok(copiedUrl.searchParams.has(MUSEA_PROPS_QUERY));
      assert.equal(copiedUrl.searchParams.get("museaGlobals"), JSON.stringify({ theme: "dark" }));

      for (const width of [360, 390, 768]) {
        await page.setViewportSize({ width, height: 1000 });
        await page.locator(".props-header").scrollIntoViewIfNeeded();
        const header = await page.locator(".props-header").evaluate((element) => ({
          width: element.clientWidth,
          scrollWidth: element.scrollWidth,
          buttons: [...element.querySelectorAll("button")].map((button) => {
            const rect = button.getBoundingClientRect();
            return {
              text: button.textContent?.trim(),
              left: rect.left,
              right: rect.right,
              width: rect.width,
            };
          }),
        }));
        assert.ok(header.width >= header.scrollWidth, JSON.stringify({ width, header }));
        assert.ok(
          header.buttons.every(
            (button) => button.width > 0 && button.left >= 0 && button.right <= width,
          ),
          JSON.stringify({ width, header }),
        );
        observations.push({ phase: "narrow-props-header", viewport: width, ...header });
        await page.screenshot({
          path: path.join(output, `props-header-${width}.png`),
          fullPage: true,
        });
      }
      await page.setViewportSize({ width: 1440, height: 1000 });

      // Slot content is rendered literally by the real generated Vue preview.
      const snapshot = JSON.parse(copiedUrl.searchParams.get(MUSEA_PROPS_QUERY)!);
      snapshot.slots = { default: "<img src=x onerror=alert(1)> {{ neverEvaluate() }} 🎨" };
      copiedUrl.searchParams.set("other", "retained");
      copiedUrl.hash = "#variant-retained";
      const link = createPropsPermalink(copiedUrl.href, art.path, snapshot);
      const fresh = await browser.newContext({
        permissions: ["clipboard-read", "clipboard-write"],
        viewport: { width: 1440, height: 1000 },
      });
      await preparePermalinkContext(fresh);
      const shared = await fresh.newPage();
      shared.on("pageerror", (error) => errors.push(String(error)));
      let dialogCount = 0;
      shared.on("dialog", async (dialog) => {
        dialogCount++;
        await dialog.dismiss();
      });
      // Conflicting local state must not win over an explicit shared snapshot.
      await shared.addInitScript(
        ({ key }) => {
          if (localStorage.getItem("musea:permalink-fixture-initialized")) return;
          localStorage.setItem("musea:permalink-fixture-initialized", "true");
          localStorage.setItem(
            key,
            JSON.stringify({
              version: 1,
              values: { label: "wrong persisted label" },
              customProps: [],
              deletedPaletteProps: [],
            }),
          );
        },
        { key: getPaletteStateStorageKey(art.path) },
      );
      await shared.goto(copied);
      observations.push({ phase: "copied-link", rendered: await preview(shared, expected) });
      await shared.goto(link);
      observations.push({ phase: "fresh-browser", rendered: await preview(shared, expected) });
      const slot = shared.frameLocator(".props-preview iframe").locator("#slot-content");
      await slot.filter({ hasText: snapshot.slots.default }).waitFor();
      assert.equal(await slot.textContent(), snapshot.slots.default);
      assert.equal(await slot.locator("img").count(), 0);
      assert.equal(
        await shared.getByRole("textbox", { name: "data-state", exact: true }).inputValue(),
        "custom 🎨",
      );
      await shared.reload();
      observations.push({ phase: "reloaded", rendered: await preview(shared, expected) });
      assert.equal(
        await shared.frameLocator(".props-preview iframe").locator("#slot-content").textContent(),
        snapshot.slots.default,
      );
      await shared.locator(".props-reset").click();
      observations.push({
        phase: "reset",
        rendered: await preview(shared, { props: defaults, attrs: {} }),
      });
      await shared.waitForURL((url) => !url.searchParams.has(MUSEA_PROPS_QUERY));
      assert.equal(new URL(shared.url()).searchParams.get("other"), "retained");
      assert.equal(new URL(shared.url()).hash, "#variant-retained");
      assert.equal(
        await shared.frameLocator(".props-preview iframe").locator("#slot-content").textContent(),
        "",
      );
      assert.equal(
        await shared.locator(".props-control-row").filter({ hasText: "data-state" }).count(),
        0,
      );
      await shared.screenshot({
        path: path.join(output, "shared-props-reset.png"),
        fullPage: true,
      });

      await shared.reload();
      await shared.locator(".tab-btn").filter({ hasText: "Props" }).click();
      observations.push({
        phase: "reset-reloaded",
        rendered: await preview(shared, { props: defaults, attrs: {} }),
      });

      const invalid = new URL(link);
      invalid.searchParams.set(MUSEA_PROPS_QUERY, "{broken");
      await shared.goto(invalid.href);
      await shared.getByRole("alert").waitFor();
      assert.equal(await shared.getByRole("alert").count(), 1);
      observations.push({
        phase: "invalid-url",
        rendered: await preview(shared, { props: defaults, attrs: {} }),
        alert: await shared.getByRole("alert").textContent(),
      });
      assert.equal(dialogCount, 0);
      assert.deepEqual(errors, []);
    } finally {
      await mkdir(output, { recursive: true });
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify({ observations, errors }, null, 2),
      );
      await browser.close();
      await server.close();
    }
  },
);
