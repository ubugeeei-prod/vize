import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Page } from "playwright";
import { createPropsBrowserServer, preparePropsContext } from "./preview-props.browser-server";

async function addObject(page: Page, name: string, value: unknown) {
  await page.getByRole("button", { name: "Add Prop", exact: true }).click();
  await page.getByRole("textbox", { name: "Prop name", exact: true }).fill(name);
  await page
    .getByRole("combobox", { name: "Prop control type", exact: true })
    .selectOption("object");
  await page
    .getByRole("textbox", { name: "Default value", exact: true })
    .fill(JSON.stringify(value));
  await page.getByRole("button", { name: "Add", exact: true }).click();
}

export async function registerPropEditorCancellationTest() {
  await test(
    "removed prop editor cancels a delayed real Monaco initialization",
    {
      skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1",
      timeout: 60_000,
    },
    async () => {
      const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
      const output =
        process.env.MUSEA_CODE_PROOF_DIR || path.join(repository, "artifacts/musea-code");
      const fixture = JSON.parse(
        await readFile(
          path.join(repository, "tests/_fixtures/differential/musea/prop-editor-cancellation.json"),
          "utf8",
        ),
      ) as {
        values: Record<string, unknown>;
        cancelled: string;
        replacement: string;
        objectDefault: unknown;
      };
      const { values } = fixture;
      const server = await createPropsBrowserServer(output, values);
      let browser: Awaited<ReturnType<typeof chromium.launch>> | undefined;
      const errors: string[] = [];
      const observation: Record<string, unknown> = { panel: "prop-editor-cancellation", fixture };
      let release!: () => void;
      const gate = new Promise<void>((resolve) => {
        release = resolve;
      });
      let requested!: () => void;
      const requestGate = new Promise<void>((resolve) => {
        requested = resolve;
      });
      try {
        browser = await chromium.launch();
        await server.listen();
        const address = server.httpServer!.address();
        assert.ok(address && typeof address !== "string");
        const context = await browser.newContext();
        await preparePropsContext(context, values);
        const page = await context.newPage();
        page.on("pageerror", (error) => errors.push(String(error)));
        const scripts: string[] = [];
        page.on("request", (request) => {
          if (request.resourceType() === "script") scripts.push(request.url());
        });
        await page.route(/\/deps\/monaco-editor\.js(?:\?|$)/, async (route) => {
          const response = await route.fetch();
          const bytes = await response.body();
          observation.delayedModule = {
            url: route.request().url(),
            sha256: createHash("sha256").update(bytes).digest("hex"),
          };
          requested();
          await gate;
          await route.fulfill({ response });
        });
        await page.goto(`http://127.0.0.1:${address.port}/__musea__/`);
        await page.locator(".art-item").first().click();
        await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
        await addObject(page, fixture.cancelled, fixture.objectDefault);
        await requestGate;
        await page
          .locator(".props-control-row")
          .filter({ hasText: fixture.cancelled })
          .getByTitle("Remove prop", { exact: true })
          .click();
        assert.equal(
          await page.locator(".props-control-row").filter({ hasText: fixture.cancelled }).count(),
          0,
        );
        observation.removedBeforeModuleResolved = true;
        release();
        await addObject(page, fixture.replacement, fixture.objectDefault);
        await page.locator(".props-control-row .monaco-editor").waitFor();
        const expected = { ...values, [fixture.replacement]: fixture.objectDefault };
        await page.waitForFunction((json) => {
          const frame =
            document.querySelector<HTMLIFrameElement>(".props-preview iframe")?.contentWindow;
          const state = frame && Reflect.get(frame, "__propsBrowserStore");
          return state && JSON.stringify(state) === json;
        }, JSON.stringify(expected));
        observation.currentValues = JSON.parse(
          (await page.locator(".props-json-code code").textContent())!,
        );
        observation.liveEditors = await page.locator(".props-control-row .monaco-editor").count();
        observation.scripts = scripts;
        assert.deepEqual(observation.currentValues, expected);
        assert.equal(observation.liveEditors, 1);
        assert.deepEqual(errors, []);
      } finally {
        try {
          release();
          await mkdir(output, { recursive: true });
          const receiptPath = path.join(output, "observations.json");
          let receipt: { observations?: unknown[] } = {};
          try {
            receipt = JSON.parse(await readFile(receiptPath, "utf8"));
          } catch {
            /* first failure */
          }
          receipt.observations = [...(receipt.observations || []), { ...observation, errors }];
          await writeFile(receiptPath, JSON.stringify(receipt, null, 2));
        } finally {
          try {
            await browser?.close();
          } finally {
            await server.close();
          }
        }
      }
    },
  );
}
