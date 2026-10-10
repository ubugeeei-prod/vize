import assert from "node:assert/strict";
import { mkdir, writeFile } from "node:fs/promises";
import type { Server } from "node:http";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Browser } from "playwright";
import { createServer, type ViteDevServer } from "vite";
import vize from "../../vite/src/index.ts";
import type { PaletteApiResponse } from "../gallery/api.ts";
import { getPaletteStateStorageKey } from "../gallery/composables/paletteState.ts";
import { musea } from "./plugin/index.ts";
import {
  addRawProp,
  currentUnsupportedProps,
  editUnsupportedJson,
  observeUnsupportedMessages,
  rawEditorPrototype,
  supportedView,
  unsupportedMessage,
  waitUnsupportedPreview,
  waitUnsupportedProps,
} from "./unsupported-props.browser-controls.ts";
import {
  buildUnsupportedPropsCopy,
  buildUnsupportedPropsGallery,
  closeUnsupportedPropsHost,
  createUnsupportedPropsHost,
  listenUnsupportedPropsHost,
  rawFilename,
  rawTitle,
  vueAlias,
  writeUnsupportedPropsFixture,
} from "./unsupported-props.browser-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));

for (const mode of ["dev", "static"] as const) {
  void test(
    `genuine ${mode} gallery retains unsupported raw props and applies only supported edits`,
    { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1", timeout: 120_000 },
    async () => {
      const output = path.join(repository, "artifacts/musea-native-inline/unsupported-props", mode);
      const root = path.join(output, "project");
      await mkdir(output, { recursive: true });
      let browser: Browser | undefined;
      let dev: ViteDevServer | undefined;
      let host: Server | undefined;
      const observed: Record<string, unknown> = { mode, root };
      const errors: string[] = [];
      const warnings: string[] = [];
      try {
        const { fixture, vector } = await writeUnsupportedPropsFixture(root);
        observed.fixture = fixture;
        browser = await chromium.launch();
        let origin: string;
        let payload: Awaited<ReturnType<typeof buildUnsupportedPropsGallery>> | undefined;
        if (mode === "dev") {
          dev = await createServer({
            root,
            configFile: false,
            resolve: { alias: { vue: vueAlias } },
            plugins: [vize(), musea({ include: ["src/**/*.art.vue"] })],
            server: { host: "127.0.0.1", port: 0 },
          });
          await dev.listen();
          const address = dev.httpServer!.address();
          assert.ok(address && typeof address !== "string");
          origin = `http://127.0.0.1:${address.port}`;
        } else {
          payload = await buildUnsupportedPropsGallery(root);
          observed.staticPayload = payload;
          host = createUnsupportedPropsHost(path.join(root, "dist"));
          origin = await listenUnsupportedPropsHost(host);
        }
        const gallery = `${origin}${mode === "static" ? "/site" : ""}/__musea__/`;
        const context = await browser.newContext({
          permissions: ["clipboard-read", "clipboard-write"],
        });
        const page = await context.newPage();
        const observePage = (current: typeof page) => {
          current.on("pageerror", (error) => errors.push(String(error)));
          current.on("console", (message) => {
            if (message.type() === "warning") warnings.push(message.text());
          });
        };
        observePage(page);
        const artPath = path.join(root, "src", vector.filename.replace(".vue", ".art.vue"));
        const storageKey = getPaletteStateStorageKey(artPath);
        const seed = Object.fromEntries(vector.names.map((name) => [name, `Retained ${name}`]));
        await page.goto(gallery);
        await page.evaluate(
          ({ key, serialized }) => {
            localStorage.setItem(key, serialized);
          },
          {
            key: storageKey,
            serialized: JSON.stringify({
              version: 1,
              values: seed,
              customProps: [],
              deletedPaletteProps: [],
            }),
          },
        );
        const openProps = async (title = vector.title) => {
          await page.goto(gallery);
          await page.locator(".art-item").filter({ hasText: title }).first().click();
          await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
          await page.locator(".props-json-code code").waitFor();
        };
        await openProps();
        const palette: PaletteApiResponse =
          mode === "static"
            ? (payload!.details[artPath].palette as PaletteApiResponse)
            : await (
                await context.request.get(
                  `${gallery}api/arts/${encodeURIComponent(artPath)}/palette`,
                )
              ).json();
        observed.palette = palette;
        assert.deepEqual(palette.unsupportedProps, ["__proto__"]);
        assert.deepEqual(
          palette.controls.map((control: { name: string }) => control.name),
          vector.names,
        );
        await waitUnsupportedProps(page, seed);
        assert.equal(
          await page.getByRole("textbox", { name: "__proto__", exact: true }).isDisabled(),
          true,
        );
        assert.match((await page.getByRole("note").textContent())!, /__proto__ is retained only/);
        observed.retained = await rawEditorPrototype(page);
        assert.equal((observed.retained as { ownProto: boolean }).ownProto, true);
        await observeUnsupportedMessages(page);
        const values = Object.fromEntries(
          vector.names.map((name) => [
            name,
            name === "__proto__" ? seed[name] : fixture.editedValues[name],
          ]),
        );
        for (const [name, value] of Object.entries(values)) {
          if (name !== "__proto__")
            await page.getByRole("textbox", { name, exact: true }).fill(value);
        }
        await waitUnsupportedProps(page, values);
        observed.controlRendered = await waitUnsupportedPreview(page, values);
        const applied = (current: Record<string, string>) =>
          Object.fromEntries(Object.entries(current).filter(([name]) => name !== "__proto__"));
        observed.controlMessage = await unsupportedMessage(page, applied(values));
        const edited = { ...values, label: "Ordinary Code edit retains raw proto" };
        await editUnsupportedJson(page, edited);
        await waitUnsupportedProps(page, edited);
        observed.codeRendered = await waitUnsupportedPreview(page, edited);
        observed.codeMessage = await unsupportedMessage(page, applied(edited));
        assert.equal(await page.locator(".props-code-editor .props-code-error").count(), 0);
        observed.codeRetained = await rawEditorPrototype(page);
        const changed = { ...edited, ["__proto__"]: "Refused changed proto" };
        await editUnsupportedJson(page, changed);
        await page.locator(".props-code-editor .props-code-error").waitFor();
        observed.refusedChange = await page
          .locator(".props-code-editor .props-code-error")
          .textContent();
        assert.match(observed.refusedChange as string, /__proto__ cannot be applied by Vue/);
        assert.deepEqual(await currentUnsupportedProps(page), edited);
        observed.refusedChangeRendered = await waitUnsupportedPreview(page, edited);
        await editUnsupportedJson(page, edited);
        await waitUnsupportedProps(page, edited);
        await page.getByRole("button", { name: "Save", exact: true }).click();
        observed.savedRaw = await page.evaluate((key) => localStorage.getItem(key), storageKey);
        assert.deepEqual(JSON.parse(observed.savedRaw as string), {
          version: 1,
          values: edited,
          customProps: [],
          deletedPaletteProps: [],
        });
        await openProps();
        await waitUnsupportedProps(page, edited);
        observed.restoredRendered = await waitUnsupportedPreview(page, edited);
        observed.restored = await rawEditorPrototype(page);
        assert.equal(
          await page.getByRole("textbox", { name: "__proto__", exact: true }).isDisabled(),
          true,
        );
        await page.locator(".props-copy-btn").click();
        const copied = await page.evaluate(() => navigator.clipboard.readText());
        observed.copied = copied;
        assert.equal(copied, await page.locator(".props-usage-code code").textContent());
        assert.equal(copied.includes("__proto__"), false);
        assert.match(copied, /CompactProbe/);
        await buildUnsupportedPropsCopy(root, copied);
        let copyOrigin = origin;
        if (!host) {
          host = createUnsupportedPropsHost(path.join(root, "dist"));
          copyOrigin = await listenUnsupportedPropsHost(host);
          observed.copyOrigin = copyOrigin;
        }
        const copiedPage = await context.newPage();
        observePage(copiedPage);
        await copiedPage.goto(`${copyOrigin}/site/clipboard.html`);
        await copiedPage.locator("output").waitFor();
        observed.copiedRendered = await copiedPage.locator("output").textContent();
        assert.deepEqual(JSON.parse(observed.copiedRendered as string), supportedView(edited));
        observed.copiedHtml = await copiedPage.content();
        const removed = applied(edited);
        await editUnsupportedJson(page, removed);
        await waitUnsupportedProps(page, removed);
        observed.removed = await rawEditorPrototype(page);
        assert.equal((observed.removed as { ownProto: boolean }).ownProto, false);
        await editUnsupportedJson(page, { ...removed, ["__proto__"]: "Refused addition" });
        await page.locator(".props-code-editor .props-code-error").waitFor();
        observed.refusedAddition = await page
          .locator(".props-code-editor .props-code-error")
          .textContent();
        assert.match(observed.refusedAddition as string, /__proto__ cannot be applied by Vue/);
        assert.deepEqual(await currentUnsupportedProps(page), removed);
        observed.refusedAdditionRendered = await waitUnsupportedPreview(page, removed);
        await editUnsupportedJson(page, removed);
        await page.locator(".props-mode-btn").filter({ hasText: "Controls" }).click();
        await page
          .locator(".props-control-row")
          .filter({ has: page.getByRole("textbox", { name: "__proto__", exact: true }) })
          .locator(".props-remove-btn")
          .click();
        await page.getByRole("button", { name: "Add Prop", exact: true }).click();
        await page.getByRole("textbox", { name: "Prop name", exact: true }).fill("__proto__");
        await page
          .getByRole("textbox", { name: "Default value", exact: true })
          .fill("Refused control addition");
        await page.getByRole("button", { name: "Add", exact: true }).click();
        await page.locator(".props-add-error").waitFor();
        observed.refusedControlAddition = await page.locator(".props-add-error").textContent();
        assert.match(
          observed.refusedControlAddition as string,
          /__proto__ cannot be applied by Vue/,
        );
        assert.deepEqual(await currentUnsupportedProps(page), removed);
        assert.equal(
          await page.getByRole("textbox", { name: "__proto__", exact: true }).count(),
          0,
        );
        await page.getByRole("button", { name: "Cancel", exact: true }).click();
        await page
          .locator(".props-control-row")
          .filter({ has: page.getByRole("textbox", { name: "scope.name", exact: true }) })
          .locator(".props-remove-btn")
          .click();
        await addRawProp(page, "scope.name", "Supported re-added custom prop");
        const readded = { ...removed, ["scope.name"]: "Supported re-added custom prop" };
        await waitUnsupportedProps(page, readded);
        observed.readdedRendered = await waitUnsupportedPreview(page, readded);
        await page.locator(".props-reset").click();
        const empty = Object.fromEntries(vector.names.map((name) => [name, ""]));
        await waitUnsupportedProps(page, empty);
        observed.resetRendered = await waitUnsupportedPreview(page, empty);
        assert.equal(await page.evaluate((key) => localStorage.getItem(key), storageKey), null);
        await openProps();
        await waitUnsupportedProps(page, empty);
        observed.resetReloaded = await rawEditorPrototype(page);
        observed.resetReloadRendered = await waitUnsupportedPreview(page, empty);
        await page.screenshot({ path: path.join(output, "supported.png"), fullPage: true });
        await openProps(rawTitle);
        await waitUnsupportedProps(page, {});
        const rawPath = path.join(root, "src", rawFilename);
        const rawPalette: PaletteApiResponse =
          mode === "static"
            ? (payload!.details[rawPath].palette as PaletteApiResponse)
            : await (
                await context.request.get(
                  `${gallery}api/arts/${encodeURIComponent(rawPath)}/palette`,
                )
              ).json();
        observed.rawPalette = rawPalette;
        assert.equal(Object.hasOwn(rawPalette, "unsupportedProps"), false);
        assert.deepEqual(rawPalette.controls, []);
        assert.equal(await page.getByRole("note").count(), 0);
        await observeUnsupportedMessages(page);
        let raw = Object.fromEntries(
          ["label", "__proto__", "constructor", "hasOwnProperty"].map((name) => [
            name,
            `Raw ${name}`,
          ]),
        );
        for (const [name, value] of Object.entries(raw)) await addRawProp(page, name, value);
        await waitUnsupportedProps(page, raw);
        observed.rawEditor = await rawEditorPrototype(page);
        observed.rawMessage = await unsupportedMessage(page, raw);
        raw = { ...raw, ["__proto__"]: "Raw unflagged Code edit" };
        await editUnsupportedJson(page, raw);
        await waitUnsupportedProps(page, raw);
        observed.rawCodeMessage = await unsupportedMessage(page, raw);
        assert.equal(await page.locator(".props-code-editor .props-code-error").count(), 0);
        await page.locator(".props-mode-btn").filter({ hasText: "Controls" }).click();
        assert.equal(
          await page.getByRole("textbox", { name: "__proto__", exact: true }).isEnabled(),
          true,
        );
        assert.equal(await page.getByRole("note").count(), 0);
        await page.locator(".props-copy-btn").click();
        observed.rawCopied = await page.evaluate(() => navigator.clipboard.readText());
        assert.match(observed.rawCopied as string, /__proto__/);
        await page.getByRole("button", { name: "Save", exact: true }).click();
        const rawKey = getPaletteStateStorageKey(rawPath);
        observed.rawSaved = await page.evaluate((key) => localStorage.getItem(key), rawKey);
        assert.deepEqual(JSON.parse(observed.rawSaved as string).values, raw);
        await openProps(rawTitle);
        await waitUnsupportedProps(page, raw);
        observed.rawRestored = await rawEditorPrototype(page);
        assert.equal(
          await page.getByRole("textbox", { name: "__proto__", exact: true }).isEnabled(),
          true,
        );
        await page.locator(".props-reset").click();
        await waitUnsupportedProps(page, {});
        assert.equal(await page.evaluate((key) => localStorage.getItem(key), rawKey), null);
        assert.equal(await page.getByRole("note").count(), 0);
        observed.rawCleared = await rawEditorPrototype(page);
        await page.screenshot({ path: path.join(output, "raw.png"), fullPage: true });
        await context.close();
        assert.deepEqual(errors, []);
        assert.deepEqual(
          warnings.filter((message) => message.startsWith("[Vue warn]")),
          [],
        );
      } catch (error) {
        observed.failure =
          error instanceof Error ? { message: error.message, stack: error.stack } : String(error);
        throw error;
      } finally {
        try {
          await writeFile(
            path.join(output, "observations.json"),
            JSON.stringify({ observed, errors, warnings }, null, 2),
          );
        } finally {
          try {
            await browser?.close();
          } finally {
            try {
              await closeUnsupportedPropsHost(host);
            } finally {
              await dev?.close();
            }
          }
        }
      }
    },
  );
}
