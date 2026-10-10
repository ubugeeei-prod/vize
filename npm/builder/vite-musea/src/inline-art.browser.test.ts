import assert from "node:assert/strict";
import { cp, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";
import os from "node:os";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import { chromium } from "playwright";
import { stringifyQuery } from "vue-router";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";
import {
  frames,
  rememberDocuments,
  documentIdentity,
} from "../../../../tools/support/release/public_acceptance/gallery.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));
const defaults = { brand: "default", scheme: "light", locale: "en" };
const changed = { brand: "ocean", scheme: "dark", locale: "ja" };

void test(
  "native inline Art renders every real Self component and updates globals without replacing Documents",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1" },
  async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "musea-native-inline-"));
    const output = path.join(repository, "artifacts/musea-native-inline");
    await mkdir(output, { recursive: true });
    await cp(path.join(repository, "examples/vite-musea/src"), path.join(root, "src"), {
      recursive: true,
    });
    await cp(
      path.join(repository, "examples/vite-musea/musea.preview.ts"),
      path.join(root, "musea.preview.ts"),
    );
    await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
    await writeFile(
      path.join(root, "preview.setup.ts"),
      `import setup from './musea.preview.ts';
export default function publicSetup(app, context) {
  Reflect.set(window, '__publicDocumentToken', crypto.randomUUID());
  Reflect.set(window, '__publicSetupCalls', (Reflect.get(window, '__publicSetupCalls') || 0) + 1);
  Reflect.set(window, '__publicFirstGlobals', { ...context.globals.value });
  setup(app, context);
}
`,
    );
    const server = await createServer({
      root,
      configFile: false,
      resolve: {
        alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
      },
      plugins: [
        vize(),
        musea({
          include: ["src/**/*.vue"],
          inlineArt: true,
          previewSetup: "preview.setup.ts",
          toolbar: [
            {
              id: "brand",
              title: "Brand",
              type: "select",
              options: ["default", "ocean"],
              default: "default",
            },
            {
              id: "scheme",
              title: "Component theme",
              type: "toggle",
              default: "light",
              options: [
                { value: "light", label: "Light" },
                { value: "dark", label: "Dark" },
              ],
            },
            { id: "locale", title: "Locale", type: "select", options: ["en", "ja"], default: "en" },
          ],
        }),
      ],
      server: { host: "127.0.0.1", port: 0 },
    });
    const browser = await chromium.launch();
    const page = await browser.newPage();
    const errors: string[] = [];
    const responses: { url: string; status: number }[] = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    page.on("response", (response) =>
      responses.push({ url: response.url(), status: response.status() }),
    );
    const observations: unknown[] = [];
    try {
      await server.listen();
      const address = server.httpServer!.address();
      assert.ok(address && typeof address !== "string");
      await page.goto(`http://127.0.0.1:${address.port}/__musea__/`);
      await page.locator(".art-item").first().waitFor();
      const href = await page
        .locator(".art-item")
        .filter({ hasText: "Button" })
        .first()
        .getAttribute("href");
      assert.ok(href);
      const initial = new URL(href, page.url());
      initial.search = stringifyQuery({ museaGlobals: JSON.stringify(defaults) });
      initial.hash = "#variant-default";
      await page.goto(initial.href);
      const before = await frames(page, defaults);
      assert.equal(before.length, 6);
      const rendered = await page.evaluate(() =>
        [...document.querySelectorAll<HTMLIFrameElement>(".variant-card iframe")].map((frame) => ({
          variant: frame
            .contentDocument!.querySelector("[data-variant]")!
            .getAttribute("data-variant"),
          buttons: [
            ...frame.contentDocument!.querySelectorAll<HTMLButtonElement>("button.btn"),
          ].map((button) => ({
            text: button.textContent,
            disabled: button.disabled,
            className: button.className,
          })),
          unresolved: frame.contentDocument!.querySelectorAll("museacomponent").length,
        })),
      );
      assert.deepEqual(
        rendered.map((variant) => variant.buttons.length),
        [1, 1, 1, 1, 3, 4],
      );
      assert.ok(rendered.every((variant) => variant.unresolved === 0));
      assert.equal(rendered[3].buttons[0].disabled, true);
      observations.push({ phase: "native-inline-defaults", frames: before, rendered });
      await rememberDocuments(page);
      await page.getByRole("combobox", { name: "Brand", exact: true }).selectOption("1");
      await page
        .getByRole("group", { name: "Component theme", exact: true })
        .getByRole("button", { name: "Dark", exact: true })
        .click();
      await page.getByRole("combobox", { name: "Locale", exact: true }).selectOption("1");
      const after = await frames(page, changed);
      assert.equal((await documentIdentity(page)).unchanged, true);
      assert.deepEqual(
        after.map((frame) => frame.documentToken),
        before.map((frame) => frame.documentToken),
      );
      for (const frame of after) assert.deepEqual(frame.firstGlobals, defaults);
      assert.equal(new URL(page.url()).hash, "#variant-default");
      assert.deepEqual(JSON.parse(new URL(page.url()).searchParams.get("museaGlobals")!), changed);
      observations.push({ phase: "native-inline-globals-no-remount", frames: after });
      assert.deepEqual(errors, []);
      assert.ok(
        responses.every((response) => response.status < 400),
        JSON.stringify(responses),
      );
      await page.screenshot({ path: path.join(output, "gallery.png"), fullPage: true });
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify(
          {
            observations,
            errors,
            responses,
            fixture: await readFile(path.join(root, "src/components/MuseaButton.vue"), "utf8"),
          },
          null,
          2,
        ),
      );
    } catch (error) {
      const documents = await Promise.all(
        page
          .frames()
          .map(async (frame) => ({ url: frame.url(), html: await frame.content().catch(String) })),
      );
      await writeFile(
        path.join(output, "failure.json"),
        JSON.stringify({ error: String(error), errors, responses, documents }, null, 2),
      );
      await page.screenshot({ path: path.join(output, "failure.png"), fullPage: true });
      throw error;
    } finally {
      await browser.close();
      await server.close();
      await rm(root, { recursive: true, force: true });
    }
  },
);
