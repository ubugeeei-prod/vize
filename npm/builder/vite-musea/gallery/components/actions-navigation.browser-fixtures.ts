import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { chromium, type Page } from "playwright";
import { createFixtureServer, prepareContext } from "./global-toolbar.browser-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const output = process.env.MUSEA_CODE_PROOF_DIR || path.join(repository, "artifacts/musea-code");
const expectedEvents = ["click", "mouseup", "pointerup", "mousedown", "pointerdown"];

async function listeners(page: Page) {
  return page.evaluate(() => ({
    count: Reflect.get(window, "__toolbarMessageListenerCount"),
    names: Reflect.get(window, "__toolbarMessageListenerNames"),
  }));
}

async function clickDefault(page: Page) {
  await page.locator(".variant-toc-item").filter({ hasText: "Default" }).first().click();
  await page.frameLocator(".variant-card iframe").first().locator("output").click();
  await page.locator(".action-count-badge").filter({ hasText: "5" }).waitFor();
}

async function eventNames(page: Page) {
  if ((await page.locator(".actions-footer-toggle").getAttribute("aria-expanded")) !== "true") {
    await page.locator(".actions-footer-toggle").click();
  }
  return page.locator(".action-item .action-type").allTextContents();
}

export async function registerActionsNavigationTest() {
  await test(
    "captured actions stay with their Art page across home navigation and repeated mounts",
    {
      skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1",
    },
    async () => {
      const observations: unknown[] = [];
      const errors: string[] = [];
      const server = await createFixtureServer(path.join(output, "actions-navigation"));
      const browser = await chromium.launch();
      try {
        await server.listen();
        const address = server.httpServer!.address();
        assert.ok(address && typeof address !== "string");
        const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
        await prepareContext(context);
        const page = await context.newPage();
        page.on("pageerror", (error) => errors.push(String(error)));
        await page.goto(`http://127.0.0.1:${address.port}/__musea__/`);
        await page.locator(".art-item").filter({ hasText: "First" }).first().click();
        await page.getByRole("heading", { name: "First", exact: true }).waitFor();
        const initialListeners = await listeners(page);
        await clickDefault(page);
        assert.deepEqual(await eventNames(page), expectedEvents);
        observations.push({
          phase: "first-art-events",
          events: await eventNames(page),
          listeners: initialListeners,
        });

        // Ordinary UI changes within the same Art must retain its captured events.
        await page.getByRole("combobox", { name: "Theme", exact: true }).selectOption("1");
        await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
        await page.locator(".props-panel").waitFor();
        await page.locator(".props-slot-editor .monaco-editor").waitFor();
        assert.deepEqual(await eventNames(page), expectedEvents);
        await page.locator(".tab-btn").filter({ hasText: "Variants" }).click();
        await page.locator(".variant-card iframe").first().waitFor();
        await page.locator(".variant-toc-item").filter({ hasText: "Default" }).first().click();
        assert.deepEqual(await eventNames(page), expectedEvents);

        // Compare repeated mounts after the existing lazy editor has finished
        // initializing its app-lifetime listeners.
        const steadyListeners = await listeners(page);

        for (const [index, title] of ["Second", "First", "Second", "First"].entries()) {
          const started = performance.now();
          await page.getByRole("link", { name: "Home", exact: true }).click();
          await page.locator(".art-item").filter({ hasText: title }).first().click();
          await page.getByRole("heading", { name: title, exact: true }).waitFor();
          const events = await eventNames(page);
          assert.deepEqual(
            events,
            [],
            `previous Art events leaked into ${title} after home navigation`,
          );
          assert.equal(await page.locator(".action-count-badge").count(), 0);
          const activeListeners = await listeners(page);
          assert.deepEqual(
            activeListeners,
            steadyListeners,
            "component remounts must not accumulate message listeners",
          );
          await clickDefault(page);
          assert.deepEqual(
            await eventNames(page),
            expectedEvents,
            "each DOM event must appear exactly once after remounting",
          );
          observations.push({
            phase: "art-remount",
            index,
            title,
            elapsedMs: performance.now() - started,
            listeners: activeListeners,
            events: await eventNames(page),
          });
        }
        assert.deepEqual(errors, []);
      } finally {
        await mkdir(output, { recursive: true });
        await writeFile(
          path.join(output, "actions-navigation.json"),
          JSON.stringify({ observations, errors }, null, 2),
        );
        const receiptPath = path.join(output, "observations.json");
        let receipt: { observations?: unknown[] } = {};
        try {
          receipt = JSON.parse(await readFile(receiptPath, "utf8"));
        } catch {
          // A focused or earlier failing browser run may not have a receipt yet.
        }
        receipt.observations = [
          ...(receipt.observations ?? []),
          { panel: "actions-navigation", observations, errors },
        ];
        await writeFile(receiptPath, JSON.stringify(receipt, null, 2));
        await browser.close();
        await server.close();
      }
    },
  );
}
