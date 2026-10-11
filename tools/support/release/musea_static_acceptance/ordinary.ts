import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { build, type Plugin } from "vite";
import { chromium } from "playwright";

void test(
  "installed colon ordinary controls retain native rendering and static URL identities",
  { timeout: 120_000 },
  async (t) => {
    const consumer = fs.realpathSync(process.cwd()),
      authority = path.join(consumer, "authority");
    const vizeName = "@vizejs/vite-plugin",
      museaName = "@vizejs/vite-plugin-musea";
    const { default: vize } = (await import(vizeName)) as { default: () => Plugin[] };
    const { musea } = (await import(museaName)) as {
      musea: (options: { include: string[]; basePath: string }) => Plugin[];
    };
    const helper = await import(
      new URL("../drivers/inline-art-static.fixtures.ts", import.meta.url).href
    );
    const root = fs.mkdtempSync(path.join(consumer, "ordinary-")),
      output = path.join(authority, "artifacts/musea-static-ordinary");
    fs.mkdirSync(output, { recursive: true });
    fs.cpSync(
      path.join(authority, "tests/tooling/fixtures/musea/static-variant-name/ordinary"),
      path.join(root, "src"),
      { recursive: true },
    );
    const originalHost = fs.readFileSync(
      path.join(authority, "tests/tooling/fixtures/musea/static-variant-name/Host.vue"),
    );
    fs.writeFileSync(path.join(root, "src/Host.vue"), originalHost);
    await build({
      root,
      configFile: false,
      base: "/built/",
      logLevel: "warn",
      cacheDir: path.join(root, ".vite-cache"),
      resolve: {
        alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
      },
      plugins: [vize(), musea({ include: ["src/**/*.art.vue"], basePath: "/gallery/" })],
      build: { outDir: path.join(root, "dist"), emptyOutDir: true, minify: true },
    });
    const bytes = fs.readFileSync(path.join(root, "dist/gallery/api/static.json"));
    fs.writeFileSync(path.join(output, "static.json"), bytes);
    const manifest = JSON.parse(bytes.toString());
    assert.equal(manifest.arts.length, 1);
    const art = manifest.arts[0];
    assert.deepEqual(
      art.variants.map((variant: { name: string }) => variant.name),
      ["Default", "State enabled"],
    );
    assert.equal(art.variants[0].isDefault, true);
    const host = await helper.hostNativeStaticGallery(path.join(root, "dist")),
      browser = await chromium.launch();
    t.after(async () => {
      await browser.close();
      await host.close();
    });
    const page = await browser.newPage(),
      errors: string[] = [],
      observations = [];
    const records: { url: string; status: number; sha256?: string; error?: string }[] = [],
      pending: Promise<void>[] = [];
    fs.mkdirSync(path.join(output, "responses"));
    page.on("response", (response) => {
      const record: (typeof records)[number] = { url: response.url(), status: response.status() };
      records.push(record);
      pending.push(
        (async () => {
          try {
            const bytes = await response.body();
            record.sha256 = createHash("sha256").update(bytes).digest("hex");
            fs.writeFileSync(path.join(output, "responses", record.sha256), bytes);
          } catch (error) {
            record.error = String(error);
          }
        })(),
      );
    });
    page.on("pageerror", (error) => errors.push(String(error)));
    page.on("console", (message) => {
      if (message.type() === "error") errors.push(message.text());
    });
    try {
      for (const variant of art.variants) {
        assert.ok(Object.hasOwn(manifest.previews[art.path], variant.name));
        const id = createHash("sha256")
          .update(art.path)
          .update("\0")
          .update(variant.name)
          .digest("hex")
          .slice(0, 20);
        const url = new URL(manifest.previews[art.path][variant.name], host.origin);
        assert.equal(url.pathname, `/built/gallery/preview/${id}.html`);
        await page.goto(url.href);
        await page.locator("button.native-host").waitFor();
        const rendered = await page.evaluate(() => {
          const button = document.querySelector<HTMLButtonElement>("button.native-host")!;
          return {
            variant: document.querySelector("[data-variant]")!.getAttribute("data-variant"),
            text: button.textContent,
            color: getComputedStyle(button).color,
            scoped: [...button.attributes].some((attribute) =>
              attribute.name.startsWith("data-v-"),
            ),
            unresolved: document.querySelectorAll("host,museacomponent").length,
            html: document.documentElement.outerHTML,
          };
        });
        assert.equal(rendered.variant, variant.name);
        assert.equal(rendered.text, "Native host");
        assert.equal(rendered.color, "rgb(12, 34, 56)");
        assert.equal(rendered.scoped, true);
        assert.equal(rendered.unresolved, 0);
        observations.push({ url: url.href, rendered });
        await page.screenshot({ path: path.join(output, `${id}.png`), fullPage: true });
      }
      assert.deepEqual(errors, []);
      await Promise.all(pending);
      assert.ok(
        records.every((record) => record.status < 400 && record.error === undefined),
        JSON.stringify(records),
      );
      assert.ok(host.responses.every((response: { status: number }) => response.status === 200));
    } catch (error) {
      await Promise.all(pending);
      fs.writeFileSync(
        path.join(output, "failure.json"),
        JSON.stringify(
          {
            error: String(error),
            html: await page.content(),
            observations,
            errors,
            records,
            server: host.responses,
          },
          null,
          2,
        ) + "\n",
      );
      await page.screenshot({ path: path.join(output, "failure.png"), fullPage: true });
      throw error;
    } finally {
      fs.writeFileSync(
        path.join(output, "observations.json"),
        JSON.stringify({ observations, errors, records, server: host.responses }, null, 2) + "\n",
      );
    }
  },
);
