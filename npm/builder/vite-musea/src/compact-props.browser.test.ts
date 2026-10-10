import assert from "node:assert/strict";
import { mkdir, mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Browser, type Page } from "playwright";
import { createServer, type ViteDevServer } from "vite";
import vize from "../../vite/src/index.ts";
import { getPaletteStateStorageKey } from "../gallery/composables/paletteState.ts";
import { compactPropsArtSource, compactPropsFixture } from "./compact-props-fixtures.ts";
import { musea } from "./plugin/index.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));

async function rendered(page: Page, expected: unknown) {
  await page.waitForFunction((serialized) => {
    const frame = document.querySelector<HTMLIFrameElement>(".props-preview iframe");
    return frame?.contentDocument?.querySelector("output")?.textContent === serialized;
  }, JSON.stringify(expected));
  return page.frameLocator(".props-preview iframe").locator("output").textContent();
}

void test(
  "native compact props edit, render, copy, persist and clear with literal keys",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1" },
  async () => {
    const fixture = await compactPropsFixture();
    const output = path.join(repository, "artifacts/musea-native-inline/compact-props");
    await mkdir(output, { recursive: true });
    const root = await realpath(await mkdtemp(path.join(os.tmpdir(), "musea-compact-props-")));
    let server: ViteDevServer | undefined;
    let browser: Browser | undefined;
    const observations: Record<string, unknown>[] = [];
    const errors: string[] = [];
    const warnings: string[] = [];
    try {
      await mkdir(path.join(root, "src"));
      for (const vector of fixture.browserCases) {
        await writeFile(path.join(root, "src", vector.filename), vector.source);
        await writeFile(
          path.join(root, "src", vector.filename.replace(".vue", ".art.vue")),
          compactPropsArtSource(vector),
        );
      }
      await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
      server = await createServer({
        root,
        configFile: false,
        resolve: {
          alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
        },
        plugins: [vize(), musea({ include: ["src/**/*.vue"], inlineArt: true })],
        server: { host: "127.0.0.1", port: 0 },
      });
      browser = await chromium.launch();
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      const origin = `http://127.0.0.1:${address.port}`;
      for (const vector of fixture.browserCases) {
        const context = await browser.newContext({
          permissions: ["clipboard-read", "clipboard-write"],
        });
        const page = await context.newPage();
        const observe = (current: Page) => {
          current.on("pageerror", (error) => errors.push(String(error)));
          current.on("console", (message) => {
            if (message.type() === "warning") warnings.push(message.text());
          });
        };
        observe(page);
        const observed: Record<string, unknown> = { vector };
        observations.push(observed);
        const openProps = async () => {
          await page.goto(`${origin}/__musea__/`);
          const response = page.waitForResponse((result) =>
            new URL(result.url()).pathname.endsWith("/palette"),
          );
          await page.locator(".art-item").filter({ hasText: vector.title }).first().click();
          await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
          return response;
        };
        const response = await openProps();
        const palette = await response.json();
        observed.palette = palette;
        const controls = vector.names.map((name) => ({
          name,
          control: "text",
          required: false,
          options: [],
        }));
        const title = vector.title.replace(/[^A-Za-z0-9]/g, "");
        const fields = vector.names
          .map((name) => `  ${name === "scope.name" ? '"scope.name"' : name}?: string;`)
          .join("\n");
        const unsupported = vector.names.includes("__proto__");
        assert.deepEqual(palette, {
          title: vector.title,
          componentTagName: "CompactProbe",
          controls,
          groups: [],
          json: JSON.stringify({ title: vector.title, controls }, null, 2),
          typescript: `export interface ${title}Props {\n${fields}\n}\n`,
          ...(unsupported ? { unsupportedProps: ["__proto__"] } : {}),
        });
        observed.analysis = await (
          await context.request.get(response.url().replace(/\/palette$/, "/analysis"))
        ).json();
        assert.deepEqual(observed.analysis, {
          props: vector.names.map((name) => ({ name, type: "string", required: false })),
          emits: [],
        });
        await page
          .frameLocator(".props-preview iframe")
          .locator("body")
          .evaluate(() => {
            window.addEventListener("message", (event) => {
              if (event.data?.type !== "musea:set-props") return;
              const props = event.data.payload.props;
              Reflect.set(
                window,
                "__compactPropsMessage",
                JSON.stringify({
                  props,
                  keys: Object.keys(props),
                  ownProto: Object.hasOwn(props, "__proto__"),
                  prototypeUnchanged: Object.getPrototypeOf(props) === Object.prototype,
                }),
              );
            });
          });
        const initialValues = JSON.parse(
          (await page.locator(".props-json-code code").textContent())!,
        );
        observed.initialValues = initialValues;
        if (unsupported) {
          assert.equal(
            await page.getByRole("textbox", { name: "__proto__", exact: true }).isDisabled(),
            true,
          );
          assert.match((await page.getByRole("note").textContent())!, /__proto__ is retained only/);
        }
        const values = Object.fromEntries(
          vector.names.map((name) => [
            name,
            unsupported && name === "__proto__" ? initialValues[name] : fixture.editedValues[name],
          ]),
        );
        for (const [name, value] of Object.entries(values)) {
          if (unsupported && name === "__proto__") continue;
          await page.getByRole("textbox", { name, exact: true }).fill(value);
        }
        const view = (current: Record<string, string>) => ({
          props: Object.fromEntries(
            Object.entries(current).filter(([name]) => name !== "__proto__"),
          ),
          attrs: {},
          ...(vector.names.includes("__proto__")
            ? { ownProto: false, prototypeUnchanged: true }
            : {}),
        });
        observed.editedRendered = await rendered(page, view(values));
        observed.currentValues = JSON.parse(
          (await page.locator(".props-json-code code").textContent())!,
        );
        assert.deepEqual(observed.currentValues, values);
        observed.messageRaw = await page
          .frameLocator(".props-preview iframe")
          .locator("body")
          .evaluate(() => Reflect.get(window, "__compactPropsMessage"));
        observed.message = JSON.parse(observed.messageRaw as string);
        const applied = Object.fromEntries(
          Object.entries(values).filter(([name]) => !unsupported || name !== "__proto__"),
        );
        assert.deepEqual(observed.message, {
          props: applied,
          keys: vector.names.filter((name) => !unsupported || name !== "__proto__"),
          ownProto: false,
          prototypeUnchanged: true,
        });
        await page.locator(".props-copy-btn").click();
        const copied = await page.evaluate(() => navigator.clipboard.readText());
        observed.copied = copied;
        assert.equal(copied, await page.locator(".props-usage-code code").textContent());
        assert.ok(copied.startsWith('<script setup lang="ts">'));
        if (unsupported) assert.equal(copied.includes("__proto__"), false);
        const prefix = path.parse(vector.filename).name;
        const copiedFilename = `${prefix}Copied.vue`;
        await writeFile(path.join(root, "src", copiedFilename), copied);
        await writeFile(
          path.join(root, `${prefix}-entry.js`),
          `try { const {createApp}=await import('vue');
            const {default:Copied}=await import('/src/${copiedFilename}');
            createApp(Copied).mount('#app');
            window.__compactCopied={html:document.querySelector('#app').innerHTML};
          } catch(error) { window.__compactCopied={error:String(error)}; }`,
        );
        await writeFile(
          path.join(root, `${prefix}-copied.html`),
          `<!doctype html><html><body><main id="app"></main><script type="module" src="/${prefix}-entry.js"></script></body></html>`,
        );
        const copiedPage = await context.newPage();
        observe(copiedPage);
        await copiedPage.goto(`${origin}/${prefix}-copied.html`);
        await copiedPage.waitForFunction(() => Reflect.has(window, "__compactCopied"));
        observed.executed = await copiedPage.evaluate(() => ({
          runtime: Reflect.get(window, "__compactCopied"),
          rendered: document.querySelector("output")?.textContent,
          html: document.documentElement.outerHTML,
        }));
        const executed = observed.executed as { runtime: { error?: string }; rendered?: string };
        assert.equal(executed.runtime.error, undefined);
        assert.deepEqual(JSON.parse(executed.rendered!), view(values));
        const compiled = await context.request.get(`${origin}/src/${copiedFilename}`);
        assert.ok(compiled.ok());
        observed.nativeCompiledModule = await compiled.text();
        const artPath = path.join(root, "src", vector.filename.replace(".vue", ".art.vue"));
        const storageKey = getPaletteStateStorageKey(artPath);
        await page.getByRole("button", { name: "Save", exact: true }).click();
        observed.savedRaw = await page.evaluate((key) => localStorage.getItem(key), storageKey);
        observed.saved = JSON.parse(observed.savedRaw as string);
        assert.deepEqual(observed.saved, {
          version: 1,
          values,
          customProps: [],
          deletedPaletteProps: [],
        });
        await openProps();
        observed.restoredRendered = await rendered(page, view(values));
        observed.restoredValues = JSON.parse(
          (await page.locator(".props-json-code code").textContent())!,
        );
        assert.deepEqual(observed.restoredValues, values);
        await page.locator(".props-reset").click();
        const empty = Object.fromEntries(vector.names.map((name) => [name, ""]));
        observed.clearedRendered = await rendered(page, view(empty));
        observed.clearedSaved = await page.evaluate((key) => localStorage.getItem(key), storageKey);
        assert.equal(observed.clearedSaved, null);
        await openProps();
        observed.clearedReloadRendered = await rendered(page, view(empty));
        observed.clearedReloadValues = JSON.parse(
          (await page.locator(".props-json-code code").textContent())!,
        );
        assert.deepEqual(observed.clearedReloadValues, empty);
        await page.screenshot({ path: path.join(output, `${prefix}.png`), fullPage: true });
        await copiedPage.screenshot({
          path: path.join(output, `${prefix}-copied.png`),
          fullPage: true,
        });
        await context.close();
      }
      assert.deepEqual(errors, []);
      assert.deepEqual(
        warnings.filter((message) => message.startsWith("[Vue warn]")),
        [],
      );
    } catch (error) {
      observations.push({
        failure:
          error instanceof Error ? { message: error.message, stack: error.stack } : String(error),
      });
      throw error;
    } finally {
      try {
        await writeFile(
          path.join(output, "observations.json"),
          JSON.stringify({ fixture, root, observations, errors, warnings }, null, 2),
        );
      } finally {
        try {
          await browser?.close();
        } finally {
          try {
            await server?.close();
          } finally {
            await rm(root, { recursive: true, force: true });
          }
        }
      }
    }
  },
);
