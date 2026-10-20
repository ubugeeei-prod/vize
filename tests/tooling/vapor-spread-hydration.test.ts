// `setDynamicProps` needs its `staticKeys` argument to patch statically named,
// changing props while adopting server-rendered Vapor nodes (#8859).
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { installDom } from "./support/upstream/pattern-runtime.ts";

const require = createRequire(import.meta.url);
const requireFromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const supportedVaporRuntime = pathToFileURL(
  requireFromUi.resolve("vue-vapor-runtime-supported/dist/vue.runtime-with-vapor.esm-browser.js"),
).href;
const supportedVaporVersion = requireFromUi("vue-vapor-runtime-supported/package.json")
  .version as string;
const { compileSfc } =
  require("../../npm/native/index.js") as typeof import("../../npm/native/index.js");

const source = `<script setup vapor>
import { ref } from "vue"
const attrs = {}
const title = ref("client")
</script>
<template><p v-bind="attrs" :title="title">content</p></template>`;

test("a spread patches a changing static prop while adopting SSR markup", async () => {
  const compiled = compileSfc(source, { filename: "App.vue", vapor: true });
  assert.deepEqual(compiled.errors, []);
  assert.deepEqual(compiled.warnings, []);
  assert.match(compiled.code, /setDynamicProps\([^\n]+\["title"\]\)/u);

  const directory = mkdtempSync(path.join(tmpdir(), "vize-vapor-spread-hydration-"));
  try {
    const modulePath = path.join(directory, "App.mjs");
    writeFileSync(
      modulePath,
      compiled.code.replaceAll(
        /from ['"]vue['"]/g,
        `from ${JSON.stringify(supportedVaporRuntime)}`,
      ),
    );
    await installDom();
    const vue = await import(supportedVaporRuntime);
    assert.equal(vue.version, supportedVaporVersion);
    const { default: App } = await import(pathToFileURL(modulePath).href);
    const host = document.body.appendChild(document.createElement("div"));
    host.innerHTML = '<p title="server">content</p>';
    const serverParagraph = host.querySelector("p");
    const warnings: string[] = [];
    const originalWarn = console.warn;
    const app = vue.createVaporSSRApp(App);
    app.config.errorHandler = (error: Error) => warnings.push(error.message);
    app.config.warnHandler = (message: string) => warnings.push(message);
    console.warn = (...args) => warnings.push(args.join(" "));
    try {
      app.mount(host);
      assert.strictEqual(host.querySelector("p"), serverParagraph);
      assert.equal(host.querySelector("p")?.getAttribute("title"), "client");
      assert.equal(warnings.length, 1);
      assert.match(warnings[0] ?? "", /Hydration attribute mismatch/u);
    } finally {
      console.warn = originalWarn;
      app.unmount();
      host.remove();
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
