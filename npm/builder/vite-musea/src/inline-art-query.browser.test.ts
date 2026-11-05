import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";
import { build, createServer, preview, type InlineConfig, type Plugin } from "vite";

import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";
import type { StaticGalleryPayload } from "./static-data.ts";

const repository = fileURLToPath(new URL("../../../../", import.meta.url));

void test(
  "inline Art builds and renders its host while another custom block keeps its runtime hook",
  { skip: process.env.VIZE_MUSEA_NATIVE_BROWSER_TESTS !== "1", timeout: 120000 },
  async (t) => {
    const root = await mkdtemp(path.join(os.tmpdir(), "musea-inline-query-browser-"));
    const output = path.join(repository, "artifacts/musea-native-inline-query");
    await mkdir(output, { recursive: true });
    t.after(() => rm(root, { recursive: true, force: true }));
    await mkdir(path.join(root, "src"));
    await writeFile(
      path.join(root, "src/Card.vue"),
      `<script setup>defineProps({ label: String });</script>
<template><button id="host">{{ label }}</button></template>
<docs>Owned by the docs plugin.</docs>
<art title="Registered inline" status="ready"><variant name="Default" default><Self label="Preview component" /></variant></art>`,
    );
    await writeFile(
      path.join(root, "app.html"),
      '<!doctype html><html><body><div id="app"></div><script type="module" src="/src/main.js"></script></body></html>',
    );
    await writeFile(
      path.join(root, "src/main.js"),
      `import { createApp } from "vue";
import Card from "./Card.vue";
Reflect.set(window, "__docs", Card.__docs);
createApp(Card, { label: "Runtime component" }).mount("#app");`,
    );
    const config = (): InlineConfig => ({
      root,
      configFile: false,
      logLevel: "warn",
      resolve: {
        alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
      },
      // Vize loads the raw block first; Musea must also claim it in transform.
      plugins: [
        vize(),
        musea({ include: ["src/**/*.vue"], inlineArt: true }),
        {
          name: "inline-art-query-docs-owner",
          transform(_code, id) {
            const query = new URLSearchParams(id.split("?", 2)[1]);
            if (query.get("type") !== "docs") return null;
            assert.equal(query.get("index"), "0");
            return 'export default component => { component.__docs = "docs runtime hook"; };';
          },
        } satisfies Plugin,
      ],
    });
    const browser = await chromium.launch();
    t.after(() => browser.close());
    const page = await browser.newPage();
    const errors: string[] = [];
    const observations: unknown[] = [];
    page.on("pageerror", (error) => errors.push(String(error)));
    const runtime = async (origin: string, phase: string) => {
      await page.goto(origin);
      await page.locator("#host").waitFor();
      const observed = await page.evaluate(() => ({
        text: document.querySelector("#host")!.textContent,
        docs: Reflect.get(window, "__docs"),
      }));
      assert.deepEqual(observed, { text: "Runtime component", docs: "docs runtime hook" });
      observations.push({ phase, ...observed });
    };
    try {
      const server = await createServer({
        ...config(),
        server: { host: "127.0.0.1", port: 0 },
      });
      try {
        await server.listen();
        const address = server.httpServer!.address();
        assert.ok(address && typeof address !== "string");
        await runtime(`http://127.0.0.1:${address.port}/app.html`, "development");
      } finally {
        await server.close();
      }
      await build({
        ...config(),
        build: {
          outDir: "dist",
          emptyOutDir: true,
          rollupOptions: { input: path.join(root, "app.html") },
        },
      });
      const manifest = JSON.parse(
        await readFile(path.join(root, "dist/__musea__/api/static.json"), "utf8"),
      ) as StaticGalleryPayload;
      assert.deepEqual(
        manifest.arts.map((art) => ({ title: art.metadata.title, variants: art.variants })),
        [
          {
            title: "Registered inline",
            variants: [
              {
                name: "Default",
                template: '<Self label="Preview component" />',
                isDefault: true,
                skipVrt: false,
              },
            ],
          },
        ],
      );
      const host = await preview({
        root,
        configFile: false,
        preview: { host: "127.0.0.1", port: 0 },
        build: { outDir: "dist" },
      });
      try {
        const address = host.httpServer.address();
        assert.ok(address && typeof address !== "string");
        const origin = `http://127.0.0.1:${address.port}`;
        await runtime(`${origin}/app.html`, "production");
        const art = manifest.arts[0]!;
        await page.goto(new URL(manifest.previews[art.path]!["Default"]!, origin).href);
        await page.locator("#host").waitFor();
        const observed = await page.evaluate(() => ({
          text: document.querySelector("#host")!.textContent,
          variant: document.querySelector("[data-variant]")!.getAttribute("data-variant"),
        }));
        assert.deepEqual(observed, { text: "Preview component", variant: "Default" });
        observations.push({ phase: "production-preview", ...observed });
      } finally {
        await new Promise<void>((resolve, reject) =>
          host.httpServer.close((error) => (error ? reject(error) : resolve())),
        );
      }
      assert.deepEqual(errors, []);
      await writeFile(
        path.join(output, "observations.json"),
        JSON.stringify({ observations, errors }, null, 2),
      );
    } catch (error) {
      await writeFile(
        path.join(output, "failure.json"),
        JSON.stringify(
          {
            error: String(error),
            stack: error instanceof Error ? error.stack : undefined,
            observations,
            errors,
            html: await page.content(),
          },
          null,
          2,
        ),
      );
      throw error;
    }
  },
);
