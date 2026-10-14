// A Vapor SFC rendered on the server by Vize hydrates with Vize's client output
// under the published Vue release: no mismatch, every server node is adopted,
// and the component stays reactive (#8859).
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";

import { installDom, vaporRuntime } from "./support/upstream/pattern-runtime.ts";

const require = createRequire(import.meta.url);
const { compileSfc } =
  require("../../npm/native/index.js") as typeof import("../../npm/native/index.js");
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const serverVue = pathToFileURL(fromUi.resolve("vue-vapor-runtime")).href;
const serverRenderer = pathToFileURL(fromUi.resolve("vue-vapor-runtime/server-renderer")).href;

const source = `<script setup vapor>
import { ref } from "vue"
const title = ref("Fruits")
const open = ref(true)
const items = ref(["apple", "banana"])
const tone = ref("warm")
</script>
<template>
  <section :class="tone">
    <h1>{{ title }}</h1>
    <p v-if="open">open</p>
    <ul>
      <li v-for="item in items" :key="item">{{ item }}</li>
    </ul>
    <button @click="items.push('cherry')">add</button>
  </section>
</template>`;

async function load(directory: string, name: string, code: string) {
  const file = path.join(directory, name);
  writeFileSync(file, code);
  return import(pathToFileURL(file).href);
}

test("a Vapor SFC rendered on the server hydrates without mismatch", async () => {
  const server = compileSfc(source, { filename: "App.vue", vapor: true, ssr: true });
  const client = compileSfc(source, { filename: "App.vue", vapor: true });
  assert.deepEqual(server.errors, []);
  assert.deepEqual(server.warnings, []);
  assert.deepEqual(client.errors, []);

  const directory = mkdtempSync(path.join(tmpdir(), "vize-vapor-ssr-"));
  try {
    const { default: ServerApp } = await load(
      directory,
      "server.mjs",
      server.code
        .replaceAll(
          /from ['"](?:@vue\/server-renderer|vue\/server-renderer)['"]/g,
          `from "${serverRenderer}"`,
        )
        .replaceAll(/from ['"]vue['"]/g, `from "${serverVue}"`),
    );
    assert.equal(ServerApp.__vapor, true);
    const { createSSRApp } = await import(serverVue);
    const { renderToString } = await import(serverRenderer);
    const html = await renderToString(createSSRApp(ServerApp));

    await installDom();
    const vue = await import(vaporRuntime);
    const { default: App } = await load(
      directory,
      "client.mjs",
      client.code.replaceAll(/from ['"]vue['"]/g, `from ${JSON.stringify(vaporRuntime)}`),
    );
    const host = document.body.appendChild(document.createElement("div"));
    host.innerHTML = html;
    const serverElements = [...host.querySelectorAll("*")];
    const warnings: string[] = [];
    const originalWarn = console.warn;
    const app = vue.createVaporSSRApp(App);
    app.config.errorHandler = (error: Error) => warnings.push(error.message);
    app.config.warnHandler = (message: string) => warnings.push(message);
    console.warn = (...args) => warnings.push(args.join(" "));
    try {
      app.mount(host);
      assert.deepEqual(warnings, []);
      // Hydration adopts every server element instead of re-creating it.
      const hydrated = [...host.querySelectorAll("*")];
      assert.equal(hydrated.length, serverElements.length);
      hydrated.forEach((element, index) => assert.strictEqual(element, serverElements[index]));
      assert.equal(host.querySelector("section")?.className, "warm");

      host.querySelector("button")?.dispatchEvent(new window.Event("click", { bubbles: true }));
      await vue.nextTick();
      assert.deepEqual(
        [...host.querySelectorAll("li")].map((li) => li.textContent),
        ["apple", "banana", "cherry"],
      );
      assert.deepEqual(warnings, []);
    } finally {
      console.warn = originalWarn;
      app.unmount();
      host.remove();
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
