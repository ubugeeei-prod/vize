import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Page } from "playwright";
import { createPropsBrowserServer, preparePropsContext } from "./preview-props.browser-server";
import { assertUsageProps } from "../../src/usage-code.test-helpers";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));

async function snapshot(page: Page) {
  return page.locator(".props-preview iframe").evaluate((element) => {
    const frame = (element as HTMLIFrameElement).contentWindow!;
    const store = Reflect.get(frame, "__propsBrowserStore");
    return {
      store: JSON.stringify(store),
      keys: Object.keys(store),
      unchangedPrototype:
        Object.getPrototypeOf(store) === Reflect.get(frame, "__propsBrowserPrototype"),
      ownProto: Object.hasOwn(store, "__proto__"),
      ownConstructor: Object.hasOwn(store, "constructor"),
      ownHasOwnProperty: Object.hasOwn(store, "hasOwnProperty"),
      rendered: frame.document.querySelector("output")!.textContent,
    };
  });
}

async function waitPreview(page: Page, values: Record<string, unknown>) {
  await page.waitForFunction((expected) => {
    const frame = document.querySelector<HTMLIFrameElement>(".props-preview iframe")?.contentWindow;
    const store = frame && Reflect.get(frame, "__propsBrowserStore");
    return store && JSON.stringify(store) === expected;
  }, JSON.stringify(values));
  return snapshot(page);
}

async function checkCopy(page: Page, expected: Record<string, unknown>) {
  await page.locator(".props-copy-btn").click();
  const copied = await page.evaluate(() => navigator.clipboard.readText());
  assert.equal(copied, await page.locator(".props-usage-code code").textContent());
  const template = copied.match(/<template>\n([\s\S]*)\n<\/template>$/)?.[1] ?? copied;
  await assertUsageProps(template, expected, "PropsProbe", {});
  return copied;
}

export async function registerPreviewPropsBrowserTest() {
  await test(
    "generated preview preserves own prop keys through editor, copy, persistence and reset",
    {
      skip: process.env.VIZE_MUSEA_BROWSER_TESTS !== "1",
    },
    async () => {
      const fixture = JSON.parse(
        await readFile(
          path.join(repository, "tests/_fixtures/differential/musea/preview-prop-keys.json"),
          "utf8",
        ),
      ) as {
        values: Record<string, unknown>;
        editedLabel: string;
        custom: unknown;
      };
      const output =
        process.env.MUSEA_CODE_PROOF_DIR || path.join(repository, "artifacts/musea-code");
      const server = await createPropsBrowserServer(output, fixture.values);
      const browser = await chromium.launch();
      const observation: Record<string, unknown> = {
        panel: "generated-preview-prop-keys",
        fixture,
        snapshots: [],
      };
      const states = observation.snapshots as unknown[];
      const errors: string[] = [];
      try {
        await server.listen();
        const address = server.httpServer!.address();
        assert.ok(address && typeof address !== "string");
        const context = await browser.newContext({
          permissions: ["clipboard-read", "clipboard-write"],
          viewport: { width: 1280, height: 900 },
        });
        await preparePropsContext(context, fixture.values);
        const page = await context.newPage();
        page.on("pageerror", (error) => errors.push(String(error)));
        await page.goto(`http://127.0.0.1:${address.port}/__musea__/`);
        await page.locator(".art-item").first().click();
        await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
        await page.frameLocator(".props-preview iframe").locator("output").waitFor();
        await page
          .frameLocator(".props-preview iframe")
          .locator("output")
          .filter({ hasText: "Authored label" })
          .waitFor();
        // Save the original failure before assertions, including the real store and DOM.
        const initial = await snapshot(page);
        states.push({ phase: "authored", ...initial });
        assert.deepEqual(JSON.parse(initial.store), fixture.values);
        assert.equal(initial.unchangedPrototype, true);
        assert.equal(initial.ownProto, true);
        assert.equal(initial.ownConstructor, true);
        assert.equal(initial.ownHasOwnProperty, true);
        await page.locator(".props-control-row .monaco-editor").nth(1).waitFor();

        await page.getByRole("textbox", { name: "label", exact: true }).fill(fixture.editedLabel);
        await page.getByRole("button", { name: "Add Prop", exact: true }).click();
        await page.getByRole("textbox", { name: "Prop name", exact: true }).fill("custom");
        await page
          .getByRole("combobox", { name: "Prop control type", exact: true })
          .selectOption("object");
        await page
          .getByRole("textbox", { name: "Default value", exact: true })
          .fill(JSON.stringify(fixture.custom));
        await page.getByRole("button", { name: "Add", exact: true }).click();
        const values = { ...fixture.values, label: fixture.editedLabel, custom: fixture.custom };
        const { ["__proto__"]: _reserved, ...expected } = values;
        const edited = await waitPreview(page, values);
        states.push({ phase: "edited", ...edited });
        assert.equal(edited.unchangedPrototype, true);
        assert.deepEqual(JSON.parse(edited.rendered!), { props: expected, attrs: {} });
        assert.deepEqual(
          JSON.parse((await page.locator(".props-json-code code").textContent())!),
          values,
        );
        observation.copied = await checkCopy(page, expected);
        await page.locator(".props-control-row .monaco-editor").nth(2).waitFor();

        await page.locator(".props-save").click();
        await page.reload();
        await page.locator(".art-item").first().click();
        await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
        const restored = await waitPreview(page, values);
        states.push({ phase: "restored", ...restored });
        assert.equal(restored.unchangedPrototype, true);
        assert.equal(restored.ownProto, true);
        assert.deepEqual(JSON.parse(restored.rendered!), { props: expected, attrs: {} });
        assert.equal(await checkCopy(page, expected), observation.copied);
        await page.locator(".props-control-row .monaco-editor").nth(2).waitFor();

        await page.locator(".props-reset").click();
        const reset = await waitPreview(page, fixture.values);
        states.push({ phase: "reset", ...reset });
        assert.equal(reset.unchangedPrototype, true);
        assert.equal(reset.ownProto, true);
        assert.equal(reset.keys.includes("custom"), false);
        await page.locator(".props-control-row .monaco-editor").nth(1).waitFor();
        // Delete an authored key through the real editor; clearing must also remove
        // its own store entry without leaving an inherited value behind.
        await page
          .locator(".props-control-row")
          .filter({ hasText: "__proto__" })
          .first()
          .getByTitle("Remove prop", { exact: true })
          .click();
        const { ["__proto__"]: _removed, ...clearedValues } = fixture.values;
        const cleared = await waitPreview(page, clearedValues);
        states.push({ phase: "cleared", ...cleared });
        assert.equal(cleared.unchangedPrototype, true);
        assert.equal(cleared.ownProto, false);
        assert.deepEqual(JSON.parse(cleared.rendered!), { props: clearedValues, attrs: {} });
        // The separate frozen-props generator serves real HTML/module/Vue output.
        // Its raw object must come from JSON parsing, not a JS __proto__ initializer.
        const staticPage = await context.newPage();
        staticPage.on("pageerror", (error) => errors.push(String(error)));
        await staticPage.goto(`http://127.0.0.1:${address.port}/__musea__/preview?static=1`);
        await staticPage.locator("output").waitFor();
        const staticState = await staticPage.evaluate(() => {
          const store = Reflect.get(window, "__propsStaticStore");
          return {
            store: JSON.stringify(store),
            keys: Object.keys(store),
            ownProto: Object.hasOwn(store, "__proto__"),
            ordinaryPrototype: Object.getPrototypeOf(store) === Object.prototype,
            rendered: document.querySelector("output")!.textContent,
          };
        });
        states.push({ phase: "frozen-generator", ...staticState });
        assert.deepEqual(JSON.parse(staticState.store), fixture.values);
        assert.equal(staticState.ownProto, true);
        assert.equal(staticState.ordinaryPrototype, true);
        assert.deepEqual(JSON.parse(staticState.rendered!), { props: clearedValues, attrs: {} });
        assert.deepEqual(errors, []);
      } finally {
        await mkdir(output, { recursive: true });
        // The existing always-uploaded receipt owns this new complete observation.
        const receiptPath = path.join(output, "observations.json");
        let receipt: { observations?: unknown[] } = {};
        try {
          receipt = JSON.parse(await readFile(receiptPath, "utf8"));
        } catch {
          /* first failed test */
        }
        receipt.observations = [...(receipt.observations || []), { ...observation, errors }];
        await writeFile(receiptPath, JSON.stringify(receipt, null, 2));
        await browser.close();
        await server.close();
      }
    },
  );
}
