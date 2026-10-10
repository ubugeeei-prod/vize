import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, realpath, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium, type Browser, type Page } from "playwright";
import { createServer, type ViteDevServer } from "vite";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";
import { usageArtSource, usageComponentFixture } from "./usage-component-fixtures.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));

async function addProp(page: Page, name: string, value: unknown) {
  await page.getByRole("button", { name: "Add Prop", exact: true }).click();
  await page.getByRole("textbox", { name: "Prop name", exact: true }).fill(name);
  await page
    .getByRole("combobox", { name: "Prop control type", exact: true })
    .selectOption(typeof value === "string" ? "text" : "object");
  await page
    .getByRole("textbox", { name: "Default value", exact: true })
    .fill(typeof value === "string" ? value : JSON.stringify(value));
  await page.getByRole("button", { name: "Add", exact: true }).click();
}

void test(
  "native gallery clipboard uses actual imported component aliases and preserves all prop values",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1" },
  async () => {
    const fixture = await usageComponentFixture();
    const output = path.join(repository, "artifacts/musea-native-inline/usage-components");
    await mkdir(output, { recursive: true });
    const root = await realpath(await mkdtemp(path.join(os.tmpdir(), "musea-usage-component-")));
    let server: ViteDevServer | undefined;
    let browser: Browser | undefined;
    const observations: Record<string, unknown>[] = [];
    const errors: string[] = [];
    const warnings: string[] = [];
    try {
      await mkdir(path.join(root, "src"));
      await writeFile(path.join(root, "src/base-button.vue"), fixture.componentSource);
      const vectors = fixture.cases.filter((vector) => vector.scriptSetup?.startsWith("import "));
      for (const vector of vectors) {
        await writeFile(
          path.join(root, "src", vector.filename),
          usageArtSource(vector, fixture.componentSource),
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
      for (const vector of vectors) {
        const context = await browser.newContext({
          permissions: ["clipboard-read", "clipboard-write"],
        });
        const page = await context.newPage();
        page.on("pageerror", (error) => errors.push(String(error)));
        page.on("console", (message) => {
          if (message.type() === "warning") warnings.push(message.text());
        });
        const observed: Record<string, unknown> = { vector };
        observations.push(observed);
        await page.goto(`${origin}/__musea__/`);
        const response = page.waitForResponse((result) =>
          new URL(result.url()).pathname.endsWith("/palette"),
        );
        await page.locator(".art-item").filter({ hasText: vector.title }).first().click();
        await page.locator(".tab-btn").filter({ hasText: "Props" }).click();
        const palette = await (await response).json();
        observed.palette = palette;
        assert.equal(palette.title, vector.title);
        assert.equal(palette.componentTagName, vector.componentTagName);
        await page.getByRole("textbox", { name: "label", exact: true }).fill(fixture.editedLabel);
        await page
          .getByRole("textbox", { name: "constructor", exact: true })
          .fill(fixture.editedConstructor);
        await page
          .getByRole("textbox", { name: "hasOwnProperty", exact: true })
          .fill(fixture.editedHasOwnProperty);
        await addProp(page, fixture.customName, fixture.customValue);
        await addProp(page, "__proto__", fixture.reservedValue);
        const props = {
          label: fixture.editedLabel,
          constructor: fixture.editedConstructor,
          hasOwnProperty: fixture.editedHasOwnProperty,
        };
        const attrs = { [fixture.customName]: fixture.customValue };
        const values = Object.fromEntries([
          ...Object.entries(props),
          ...Object.entries(attrs),
          ["__proto__", fixture.reservedValue],
        ]);
        const expected = { props, attrs };
        await page.waitForFunction((serialized) => {
          const frame = document.querySelector<HTMLIFrameElement>(".props-preview iframe");
          return frame?.contentDocument?.querySelector("output")?.textContent === serialized;
        }, JSON.stringify(expected));
        observed.rendered = await page
          .frameLocator(".props-preview iframe")
          .locator("output")
          .textContent();
        observed.values = JSON.parse((await page.locator(".props-json-code code").textContent())!);
        assert.deepEqual(observed.values, values);
        await page.locator(".props-copy-btn").click();
        const copied = await page.evaluate(() => navigator.clipboard.readText());
        observed.copied = copied;
        assert.equal(copied, await page.locator(".props-usage-code code").textContent());
        assert.ok(copied.includes(`<${vector.componentTagName} `));
        const prefix = path.parse(vector.filename).name;
        const copiedFilename = `${prefix}Copied.vue`;
        // Ordinary isolated usage keeps its established template-only output.
        // Supply the real component binding in that consumer's script scope;
        // complete copied SFCs are executed byte-for-byte.
        const consumerSource = copied.startsWith("<script")
          ? copied
          : `<script setup lang="ts">\nimport ${vector.componentTagName} from './base-button.vue';\n</script>\n<template>\n${copied}\n</template>`;
        observed.consumerSource = consumerSource;
        await writeFile(path.join(root, "src", copiedFilename), consumerSource);
        await writeFile(
          path.join(root, `${prefix}-copied-entry.js`),
          `try { const {createApp}=await import('vue');
            const {default:Copied}=await import('/src/${copiedFilename}');
            createApp(Copied).mount('#app');
            window.__copiedUsage={html:document.querySelector('#app').innerHTML};
          } catch(error) { window.__copiedUsage={error:String(error)}; }`,
        );
        await writeFile(
          path.join(root, `${prefix}-copied.html`),
          `<!doctype html><html><body><main id="app"></main><script type="module" src="/${prefix}-copied-entry.js"></script></body></html>`,
        );
        const copiedPage = await context.newPage();
        copiedPage.on("pageerror", (error) => errors.push(String(error)));
        copiedPage.on("console", (message) => {
          if (message.type() === "warning") warnings.push(message.text());
        });
        await copiedPage.goto(`${origin}/${prefix}-copied.html`);
        await copiedPage.waitForFunction(() => Reflect.has(window, "__copiedUsage"));
        observed.executed = await copiedPage.evaluate(() => ({
          runtime: Reflect.get(window, "__copiedUsage"),
          rendered: document.querySelector("output")?.textContent,
          html: document.documentElement.outerHTML,
        }));
        const executed = observed.executed as { runtime: { error?: string }; rendered?: string };
        assert.equal(executed.runtime.error, undefined);
        assert.deepEqual(JSON.parse(executed.rendered!), expected);
        await page.screenshot({
          path: path.join(output, `${vector.filename}.png`),
          fullPage: true,
        });
        await copiedPage.screenshot({
          path: path.join(output, `${vector.filename}-copied.png`),
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
          error instanceof Error
            ? { name: error.name, message: error.message, stack: error.stack }
            : String(error),
      });
      throw error;
    } finally {
      try {
        await writeFile(
          path.join(output, "observations.json"),
          JSON.stringify(
            {
              fixture,
              root,
              observations,
              errors,
              warnings,
              component: await readFile(path.join(root, "src/base-button.vue"), "utf8").catch(
                (error: unknown) => ({ error: String(error) }),
              ),
            },
            null,
            2,
          ),
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
