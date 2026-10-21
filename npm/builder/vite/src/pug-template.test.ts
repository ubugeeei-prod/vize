// `<template lang="pug">` through a real Vite build renders exactly like the
// same component written with an HTML template (npm/builder/shared/pug-twins.cjs),
// for both the client and the SSR compile.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { build, type Rollup } from "vite";

import { vize } from "./index.ts";

const require = createRequire(import.meta.url);
const twins = require("../../shared/pug-twins.cjs") as {
  pug: string;
  jade: string;
  html: string;
};
const vueRuntime = require.resolve("vue");
const serverRenderer = require.resolve("vue/server-renderer");

async function render(source: string, ssr: boolean): Promise<string> {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vite-pug-"));
  try {
    fs.writeFileSync(path.join(root, "App.vue"), source);
    fs.writeFileSync(path.join(root, "entry.js"), 'export { default } from "./App.vue";\n');
    const result = (await build({
      root,
      configFile: false,
      logLevel: "silent",
      plugins: [vize({ configMode: false })],
      build: {
        write: false,
        minify: false,
        ...(ssr
          ? { ssr: path.join(root, "entry.js") }
          : { lib: { entry: path.join(root, "entry.js"), formats: ["es"], fileName: "app" } }),
        rollupOptions: { external: [/^vue/] },
      },
    })) as Rollup.RollupOutput | Rollup.RollupOutput[];
    const chunk = (Array.isArray(result) ? result[0] : result).output.find(
      (item): item is Rollup.OutputChunk => item.type === "chunk",
    );
    const file = path.join(root, "app.mjs");
    fs.writeFileSync(
      file,
      (chunk?.code ?? "")
        .replace(
          /from ["']vue\/server-renderer["']/g,
          `from ${JSON.stringify(pathToFileURL(serverRenderer).href)}`,
        )
        .replace(/from ["']vue["']/g, `from ${JSON.stringify(pathToFileURL(vueRuntime).href)}`),
    );
    const { default: App } = (await import(pathToFileURL(file).href)) as { default: object };
    const { createSSRApp } = (await import(pathToFileURL(vueRuntime).href)) as typeof import("vue");
    const { renderToString } = (await import(pathToFileURL(serverRenderer).href)) as {
      renderToString: (app: unknown) => Promise<string>;
    };
    return await renderToString(createSSRApp(App));
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

for (const ssr of [false, true]) {
  void test(`vite ${ssr ? "SSR" : "client"}: a pug template renders like its HTML twin`, async () => {
    const [pug, html] = [await render(twins.pug, ssr), await render(twins.html, ssr)];
    assert.equal(pug, html);
    assert.equal(await render(twins.jade, ssr), html);
    assert.match(html, /^<section id="app" class="card active">/);
  });
}
