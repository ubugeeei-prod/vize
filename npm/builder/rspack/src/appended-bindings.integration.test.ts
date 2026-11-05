// #11583: component bindings are appended to the exported `_sfc_main` after the
// module instead of spliced in before `export default`. A real development
// Rspack build with HMR must still hand every importer a component that
// carries `__scopeId`, CSS modules, custom element `styles`, custom-block
// effects and its HMR id before the first render.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { test } from "node:test";
import { rspack, type Stats } from "@rspack/core";
import { createSSRApp, type Component } from "vue";
import { renderToString } from "vue/server-renderer";
import { VizePlugin } from "./plugin/index.ts";
import { packageLoaderAliases } from "./test/helpers.ts";

const require = createRequire(import.meta.url);

const FILES: Record<string, string> = {
  "Setup.vue": `<script setup>
const label = "setup 日本 🐱";
</script>
<template><p class="setup" :class="$style.mark">{{ label }}</p></template>
<style scoped>.setup { color: red; }</style>
<style module>.mark { color: blue; }</style>
<i18n>{ "en": { "hello": "Hello" } }</i18n>`,
  "Options.vue": `<script>
export default { name: "Options", data: () => ({ label: "options é" }) };
</script>
<template><p :class="theme.tone">{{ label }}</p></template>
<style scoped>p { color: red; }</style>
<style module="theme">.tone { color: green; }</style>`,
  "Template.vue": `<template><p class="template">template 🐱</p></template>
<style scoped>.template { color: red; }</style>`,
  "Widget.ce.vue": `<script setup>const label = "widget";</script>
<template><p>{{ label }}</p></template>
<style>p { color: rgb(1, 2, 3); }</style>`,
  // Negative control: the module exports no \`_sfc_main\`, so nothing is
  // attached and the re-exported component must still work.
  "Reexport.vue": `<script>export { default } from "./plain.js";</script>
<style scoped>p { color: red; }</style>`,
  "plain.js": `export default { name: "Plain", render: () => "plain component" };`,
  "i18n-loader.cjs": `module.exports = function () {
  return "export default function (component) { component.__i18n = ['loaded']; }";
};`,
  "css-text-loader.cjs": `module.exports = function (css) { return "export default " + JSON.stringify(css); };`,
};
const NAMES = ["Setup", "Options", "Template", "Widget", "Reexport"] as const;
type Name = (typeof NAMES)[number];

/** What each importer saw at import time, before anything rendered. */
const ENTRY = `${NAMES.map((name) => `import ${name} from "./${name === "Widget" ? "Widget.ce" : name}.vue";`).join("\n")}
const components = { ${NAMES.join(", ")} };
const keys = ["__scopeId", "__cssModules", "styles", "__i18n", "__hmrId", "__file"];
export const seen = Object.fromEntries(Object.entries(components).map(([name, component]) => [
  name,
  Object.fromEntries(keys.filter((key) => key in component).map((key) => [key, component[key]])),
]));
export { components };`;

type Bindings = {
  __scopeId?: string;
  __cssModules?: Record<string, Record<string, string>>;
  styles?: string[];
  __i18n?: string[];
  __hmrId?: string;
  __file?: string;
};

async function build() {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-rspack-appended-")));
  for (const [file, content] of Object.entries({ ...FILES, "entry.js": ENTRY })) {
    fs.writeFileSync(path.join(root, file), content);
  }
  const outputPath = path.join(root, "dist");
  const compiler = rspack({
    mode: "development",
    target: "node",
    context: root,
    entry: "./entry.js",
    devtool: false,
    output: { path: outputPath, filename: "bundle.cjs", library: { type: "commonjs2" } },
    experiments: { css: true },
    externals: {
      vue: `commonjs ${require.resolve("vue")}`,
      "vue/server-renderer": `commonjs ${require.resolve("vue/server-renderer")}`,
    },
    infrastructureLogging: { level: "error" },
    resolveLoader: { alias: packageLoaderAliases },
    module: {
      rules: [
        {
          test: /\.vue$/,
          oneOf: [
            {
              resourceQuery: /type=style.*inline/,
              type: "javascript/auto",
              use: [
                path.join(root, "css-text-loader.cjs"),
                "@vizejs/rspack-plugin/scope-loader",
                "@vizejs/rspack-plugin/style-loader",
              ],
            },
            {
              resourceQuery: /type=style.*module/,
              type: "css/module",
              use: ["@vizejs/rspack-plugin/scope-loader", "@vizejs/rspack-plugin/style-loader"],
            },
            {
              resourceQuery: /type=style/,
              type: "css",
              use: ["@vizejs/rspack-plugin/scope-loader", "@vizejs/rspack-plugin/style-loader"],
            },
            { resourceQuery: /type=i18n/, use: [path.join(root, "i18n-loader.cjs")] },
            { use: ["@vizejs/rspack-plugin/loader"] },
          ],
        },
      ],
    },
    plugins: [
      new VizePlugin({ autoRules: false, typescript: false, css: { native: true } }),
      new rspack.HotModuleReplacementPlugin(),
    ],
  });
  const stats = await new Promise<Stats>((resolve, reject) => {
    compiler.run((error, result) => {
      compiler.close((closeError) => {
        if (error || closeError) reject(error ?? closeError);
        else if (!result) reject(new Error("Rspack did not return stats"));
        else resolve(result);
      });
    });
  });
  const info = stats.toJson({ all: false, errors: true, warnings: true });
  assert.equal(stats.hasErrors(), false, JSON.stringify(info.errors, null, 2));
  return { root, bundle: path.join(outputPath, "bundle.cjs") };
}

void test("appended bindings are on every component before its first render", async (t) => {
  const { root, bundle } = await build();
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const records: { id: string; bindings: Bindings }[] = [];
  const runtime = globalThis as { __VUE_HMR_RUNTIME__?: unknown };
  runtime.__VUE_HMR_RUNTIME__ = {
    createRecord(id: string, component: Bindings) {
      records.push({ id, bindings: { ...component } });
      return true;
    },
    rerender() {},
    reload() {},
  };
  t.after(() => delete runtime.__VUE_HMR_RUNTIME__);
  const { seen, components } = require(bundle) as {
    seen: Record<Name, Bindings>;
    components: Record<Name, Component & Bindings>;
  };

  for (const name of ["Setup", "Options", "Template"] as const) {
    assert.match(seen[name].__scopeId ?? "", /^data-v-[0-9a-f]{8}$/, `${name} __scopeId`);
  }
  assert.equal(typeof seen.Setup.__cssModules?.$style.mark, "string", "Setup $style");
  assert.equal(typeof seen.Options.__cssModules?.theme.tone, "string", "Options theme");
  assert.deepEqual(seen.Setup.__i18n, ["loaded"], "the custom block ran on _sfc_main");
  assert.equal(seen.Widget.styles?.length, 1, "custom element styles");
  assert.match(seen.Widget.styles?.[0] ?? "", /rgb\(1, 2, 3\)/);
  for (const name of ["Setup", "Options", "Template", "Widget"] as const) {
    assert.match(seen[name].__hmrId ?? "", /^[0-9a-f]{8}$/, `${name} __hmrId`);
    assert.equal(seen[name].__file, `${name === "Widget" ? "Widget.ce" : name}.vue`);
  }
  // HMR registers each component after every other binding was attached.
  assert.deepEqual(
    new Set(records.map(({ id }) => id)),
    new Set(
      (["Setup", "Options", "Template", "Widget"] as const).map((name) => seen[name].__hmrId),
    ),
  );
  assert.equal(records.length, 4);
  for (const { bindings } of records) assert.ok(bindings.__file, "registered after __file");
  const setup = records.find(({ id }) => id === seen.Setup.__hmrId)!.bindings;
  assert.equal(setup.__scopeId, seen.Setup.__scopeId);
  assert.ok(setup.__cssModules?.$style, "registered after the CSS module binding");
  assert.deepEqual(setup.__i18n, ["loaded"], "registered after the custom block");

  // Negative control: no `_sfc_main`, so nothing was attached; it still works.
  assert.deepEqual(seen.Reexport, {});
  assert.equal(components.Reexport.name, "Plain");

  const html = Object.fromEntries(
    await Promise.all(
      NAMES.filter((name) => name !== "Widget").map(
        async (name) => [name, await renderToString(createSSRApp(components[name]))] as const,
      ),
    ),
  );
  for (const name of ["Setup", "Options", "Template"] as const) {
    assert.ok(html[name].includes(seen[name].__scopeId!), `${name} renders its scope`);
  }
  assert.ok(html.Setup.includes(seen.Setup.__cssModules!.$style.mark));
  assert.ok(html.Setup.includes("setup 日本 🐱"));
  assert.ok(html.Options.includes(seen.Options.__cssModules!.theme.tone));
  assert.equal(html.Reexport, "plain component");
});
