import assert from "node:assert/strict";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { VizePlugin } from "../dist/index.mjs";

// An isolated consumer supplies the selected Rspack major, dev server and browser.
const require = createRequire(path.resolve(process.env.VIZE_HMR_RUNTIME ?? ".", "package.json"));
const { rspack } = require("@rspack/core");
const { RspackDevServer } = require("@rspack/dev-server");
const { chromium } = require("playwright");
const packageRoot = fileURLToPath(new URL("../", import.meta.url));

for (const { nativeCss, autoRules, typescript } of [true, false].flatMap((typescript) => [
  { nativeCss: true, autoRules: true, typescript },
  { nativeCss: false, autoRules: true, typescript },
  { nativeCss: false, autoRules: false, typescript },
])) {
  void test(
    `browser HMR (${nativeCss ? "Native CSS" : "CssExtract"}, ${autoRules ? "auto" : "manual"}, ${typescript ? "TS" : "JS"})`,
    { timeout: 120_000 },
    async (t) => {
      t.diagnostic(`Rspack ${require("@rspack/core/package.json").version}`);
      const root = await fs.mkdtemp(path.join(os.tmpdir(), "vize-rspack-hmr-"));
      let server;
      let browser;
      t.after(async () => {
        await browser?.close();
        await server?.stop();
        await fs.rm(root, { recursive: true, force: true });
      });
      let source = `<script setup${typescript ? ' lang="ts"' : ""}>
import { ref, useCssModule } from 'vue';
import { seed } from './dependency.js';
import Child from './Child.vue';
import NamespaceChild from './NamespaceChild.vue';
${typescript ? "import { Ref } from 'vue';" : ""}
const count${typescript ? ": Ref<number>" : ""} = ref(seed);
const styles = useCssModule();
const theme = useCssModule('theme');
</script>
<template>
<button id="counter" :class="$style.button" @click="count++">before {{ count }}</button>
<div id="captured-style" :class="styles.button">captured</div>
<div id="named-style" :class="theme.label">named</div>
<Child :depth="0" /><NamespaceChild :depth="0" />
</template>
<style scoped>button { color: rgb(10, 20, 30); }</style>
<style module>
.variantOne { border-left: 3px solid rgb(110, 120, 130); }
.variantTwo { border-left: 5px solid rgb(130, 120, 110); }
.button { composes: variantOne; padding: 7px; }
</style>
<style module="theme">
.thin { outline: 1px solid black; }
.thick { outline: 2px solid black; }
.label { composes: thin; }
</style>`;
      await fs.writeFile(path.join(root, "App.vue"), source);
      await fs.writeFile(path.join(root, "dependency.js"), "export const seed = 0;");
      await fs.writeFile(
        path.join(root, "Child.vue"),
        `<script setup>
import App from './App.vue';
defineProps({ depth: Number });
</script><template><App v-if="depth > 0" /><span v-else id="cycle-leaf">leaf</span></template>`,
      );
      await fs.writeFile(
        path.join(root, "NamespaceChild.vue"),
        `<script setup>
import * as components from './App.vue';
defineProps({ depth: Number });
</script><template><component :is="components.default" v-if="depth > 0" /><span v-else id="namespace-leaf">namespace leaf</span></template>`,
      );
      await fs.writeFile(
        path.join(root, "index.html"),
        '<link rel="stylesheet" href="/styles.css"><div id="app"></div><script src="/bundle.js"></script>',
      );
      await fs.writeFile(
        path.join(root, "entry.js"),
        `import { createApp, h } from 'vue';
import App from './App.vue';
window.pageToken = Math.random();
createApp({ render: () => h(App) }).mount('#app');
if (module.hot) module.hot.addStatusHandler(status => { window.hotStatus = status; });`,
      );
      const compiler = rspack({
        context: root,
        mode: "development",
        entry: "./entry.js",
        devtool: "eval-source-map",
        output: {
          path: path.join(root, "dist"),
          filename: "bundle.js",
          cssFilename: "styles.css",
          publicPath: "/",
        },
        resolve: { alias: { vue: require.resolve("vue/dist/vue.runtime.esm-bundler.js") } },
        resolveLoader: {
          alias: Object.fromEntries(
            ["loader", "style-loader", "scope-loader", "jsx-loader"].map((name) => [
              `@vizejs/rspack-plugin/${name}`,
              path.join(
                packageRoot,
                "dist/loader",
                name === "loader" ? "index.mjs" : `${name}.mjs`,
              ),
            ]),
          ),
        },
        experiments: { css: nativeCss },
        module: {
          rules: [
            ...(autoRules
              ? [
                  {
                    test: /\.vue$/,
                    loader: "@vizejs/rspack-plugin/loader",
                    options: { sourceMap: true },
                  },
                ]
              : [
                  {
                    test: /\.vue$/,
                    oneOf: [
                      {
                        resourceQuery: /type=style/,
                        type: "javascript/auto",
                        use: [
                          rspack.CssExtractRspackPlugin.loader,
                          {
                            loader: require.resolve("css-loader"),
                            options: { modules: { localIdentName: "[local]__[hash:hex:6]" } },
                          },
                          "@vizejs/rspack-plugin/scope-loader",
                          "@vizejs/rspack-plugin/style-loader",
                        ],
                      },
                      {
                        loader: "@vizejs/rspack-plugin/loader",
                        options: { sourceMap: true, css: { native: false } },
                      },
                    ],
                  },
                ]),
            ...(!nativeCss && autoRules
              ? [
                  {
                    test: /\.css$/,
                    type: "javascript/auto",
                    use: [
                      rspack.CssExtractRspackPlugin.loader,
                      {
                        loader: require.resolve("css-loader"),
                        options: { modules: { localIdentName: "[local]__[hash:hex:6]" } },
                      },
                    ],
                  },
                ]
              : []),
          ],
        },
        plugins: [
          new VizePlugin({ autoRules, css: { native: nativeCss } }),
          new rspack.DefinePlugin({
            __VUE_OPTIONS_API__: true,
            __VUE_PROD_DEVTOOLS__: false,
            __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: false,
          }),
          ...(!nativeCss ? [new rspack.CssExtractRspackPlugin({ filename: "styles.css" })] : []),
        ],
        infrastructureLogging: { level: "error" },
        stats: "errors-only",
      });
      server = new RspackDevServer(
        {
          host: "127.0.0.1",
          port: 0,
          hot: true,
          liveReload: false,
          static: { directory: root, watch: false },
          client: { logging: "error", overlay: false },
        },
        compiler,
      );
      await server.start();
      browser = await chromium.launch({ headless: true });
      const page = await browser.newPage();
      const errors = [];
      page.on("pageerror", (error) => errors.push(error.message));
      page.on("console", (message) => {
        if (message.type() === "error") errors.push(message.text());
      });
      await page.goto(`http://127.0.0.1:${server.server.address().port}`);
      await page.locator("#cycle-leaf").waitFor();
      await page.locator("#namespace-leaf").waitFor();
      assert.deepEqual(errors, [], "circular default and namespace imports must load normally");
      const button = page.locator("#counter");
      await button.click();
      await button.click();
      assert.equal(await button.textContent(), "before 2");
      const token = await page.evaluate(() => window.pageToken);
      async function write(file, content) {
        await page.evaluate(() => {
          window.hotStatus = "waiting";
        });
        await fs.writeFile(path.join(root, file), content);
        await page.waitForFunction(() => window.hotStatus === "idle");
        // CssExtract debounces link replacement by 50ms after JS HMR is idle.
        // Require a fresh quiet interval and loaded sheets after each update.
        if (!nativeCss) {
          await page.waitForFunction(
            (since) => {
              const lastResponse = performance
                .getEntriesByType("resource")
                .reduce((latest, entry) => Math.max(latest, entry.responseEnd), since);
              return (
                performance.now() - lastResponse >= 100 &&
                [...document.querySelectorAll('link[rel="stylesheet"]')].every((link) => link.sheet)
              );
            },
            await page.evaluate(() => performance.now()),
          );
        }
        assert.equal(await page.evaluate(() => window.pageToken), token, "page must not reload");
      }
      async function edit(from, to) {
        assert.ok(source.includes(from));
        source = source.replace(from, to);
        await write("App.vue", source);
      }
      async function expectModuleBorder(width) {
        await page.waitForFunction(
          (expected) =>
            ["#counter", "#captured-style"].every(
              (selector) =>
                getComputedStyle(document.querySelector(selector)).borderLeftWidth === expected,
            ),
          width,
        );
      }
      await expectModuleBorder("3px");
      await edit("composes: variantOne", "composes: variantTwo");
      await expectModuleBorder("5px");
      assert.equal(
        await button.textContent(),
        "before 2",
        "useCssModule mappings update without losing state",
      );
      await edit("10, 20, 30", "40, 50, 60");
      await page.waitForFunction(
        () => getComputedStyle(document.querySelector("button")).color === "rgb(40, 50, 60)",
      );
      assert.equal(
        await button.textContent(),
        "before 2",
        "style-only update preserves state with source maps enabled",
      );
      await edit("before {{", "after {{");
      assert.equal(await button.textContent(), "after 2", "template-only update preserves state");
      await edit("padding: 7px", "padding: 13px");
      await page.waitForFunction(
        () => getComputedStyle(document.querySelector("button")).padding === "13px",
      );
      assert.equal(
        await button.textContent(),
        "after 2",
        "CSS Module updates preserve state after a template update",
      );
      await edit("composes: variantTwo", "composes: variantOne");
      await expectModuleBorder("3px");
      assert.equal(
        await button.textContent(),
        "after 2",
        "captured mappings survive repeated updates",
      );
      await edit("composes: thin", "composes: thick");
      await page.waitForFunction(
        () => getComputedStyle(document.querySelector("#named-style")).outlineWidth === "2px",
      );
      assert.equal(
        await button.textContent(),
        "after 2",
        "named useCssModule mappings preserve state",
      );
      await write("dependency.js", "export const seed = 3;");
      assert.equal(
        await button.textContent(),
        "after 3",
        "imported script changes reload even with identical SFC text",
      );
      await edit("ref(seed)", "ref(5)");
      assert.equal(await button.textContent(), "after 5", "script update reloads component");
      await edit("after {{", "again {{");
      assert.equal(await button.textContent(), "again 5");

      // External blocks are loader dependencies, not separate Vue component modules.
      await fs.writeFile(
        path.join(root, "script.js"),
        "export default { data() { return { count: 7 }; } };",
      );
      await fs.writeFile(
        path.join(root, "template.html"),
        '<button id="counter" @click="count++">external {{ count }}</button>',
      );
      await fs.writeFile(path.join(root, "style.css"), "button { color: rgb(70, 80, 90); }");
      await write(
        "App.vue",
        '<script src="./script.js"></script><template src="./template.html"></template><style scoped src="./style.css"></style>',
      );
      await button.click();
      assert.equal(await button.textContent(), "external 8");
      await write(
        "template.html",
        '<button id="counter" @click="count++">changed {{ count }}</button>',
      );
      assert.equal(
        await button.textContent(),
        "changed 8",
        "external template preserves Options API state",
      );
      await write("style.css", "button { color: rgb(90, 80, 70); }");
      await page.waitForFunction(
        () => getComputedStyle(document.querySelector("button")).color === "rgb(90, 80, 70)",
      );
      assert.equal(await button.textContent(), "changed 8", "external style preserves state");
      await write("script.js", "export default { data() { return { count: 9 }; } };");
      assert.equal(await button.textContent(), "changed 9", "external script reloads component");
      await write(
        "App.vue",
        '<script src="./script.js"></script><template src="./template.html"></template>',
      );
      await page.waitForFunction(
        () => getComputedStyle(document.querySelector("button")).color !== "rgb(90, 80, 70)",
      );
      assert.equal(await button.textContent(), "changed 9", "removing a style keeps the component");
      assert.deepEqual(errors, []);
    },
  );
}
