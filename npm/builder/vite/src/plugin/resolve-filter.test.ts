import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { interpreter } from "@rolldown/pluginutils";
import { build, type Rollup } from "vite";
import { vize } from "./index.ts";
import { RESOLVE_ID_FILTER, withResolveFilter } from "./resolve-filter.ts";
import { isPotentialVizeImporter, isPotentialVizeResolveId } from "./resolve.ts";

const require = createRequire(import.meta.url);

test("the host resolve filter retains every request the handler can accept", () => {
  const sources = [
    "vue",
    "vue?version=1",
    "vue/dist/vue.runtime.esm-bundler.js",
    "vue-router",
    "vue-router/auto?build=1",
    "@vue/server-renderer",
    "#imports",
    "some-package",
    "./local.ts",
    "virtual:vize-styles",
    "\0vize:style",
    "vize:page",
    "/@fs/C:/Page.vue",
    "/src/App.vue",
    "/src/App.vue?vue&vize",
    "/src/App.vue.tsx?vue&vize",
    "/src/App.vue.__vize_style_0",
    "/src/page.ts?macro=true",
    "/src/page.ts?definePage",
    "/src/style.css?vize-file=App",
    "/src/ordinary.ts",
    "virtual:foreign",
  ];
  const importers = [
    undefined,
    "/src/main.ts",
    "/src/App.vue",
    "/src/App.vue?vue&vize",
    "C:\\src\\App.vue",
    "\0vize:component",
    "vize:component",
  ];
  for (const source of sources) {
    for (const importer of importers) {
      if (isPotentialVizeResolveId(source) || isPotentialVizeImporter(importer)) {
        assert.equal(
          interpreter(RESOLVE_ID_FILTER, undefined, source, undefined, importer),
          true,
          JSON.stringify({ source, importer }),
        );
      }
    }
  }
  for (const source of ["./ordinary.ts", "some-package", "#imports"]) {
    assert.equal(
      interpreter(RESOLVE_ID_FILTER, undefined, source, undefined, "/src/main.ts"),
      false,
    );
  }
});

test("actual builds skip ordinary resolver edges and preserve SFC import resolution", async () => {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-resolve-filter-")));
  fs.writeFileSync(path.join(root, "package.json"), '{"type":"module"}');
  fs.writeFileSync(path.join(root, "local.ts"), 'export const local = "relative-probe";');
  fs.writeFileSync(path.join(root, "injected.ts"), 'export const injected = "alias-probe";');
  fs.writeFileSync(path.join(root, "bare.ts"), 'export const bare = "bare-probe";');
  fs.writeFileSync(
    path.join(root, "App.vue"),
    `<script setup lang="ts">
import { ref } from "vue";
import { local } from "./local";
import { injected } from "#imports";
import { bare } from "bare-package";
const value = ref(local + injected + bare);
</script><template><p>{{ value }}</p></template>`,
  );
  const ordinary = Array.from({ length: 40 }, (_, index) => {
    fs.writeFileSync(path.join(root, `ordinary${index}.ts`), `export const value = ${index};`);
    return `import { value as value${index} } from "./ordinary${index}";`;
  });
  fs.writeFileSync(
    path.join(root, "main.ts"),
    [
      ...ordinary,
      'import App from "./App.vue";',
      `export const values = [App, ${Array.from({ length: 40 }, (_, i) => `value${i}`).join(",")}];`,
    ].join("\n"),
  );
  const run = async (filtered: boolean) => {
    const seen: string[] = [];
    const plugins = vize({ configMode: false });
    const plugin = plugins.find((value) => value.name === "vite-plugin-vize");
    assert(plugin?.resolveId && typeof plugin.resolveId === "object");
    const hook = plugin.resolveId;
    plugin.resolveId = withResolveFilter(function (source, importer, options) {
      seen.push(source.replaceAll("\\", "/"));
      return hook.handler.call(this, source, importer, options);
    });
    if (!filtered) Object.assign(plugin.resolveId, { filter: undefined });
    let nativeFilters = false;
    plugins.push({
      name: "observe-resolve-filter-host",
      buildStart() {
        nativeFilters = typeof this.meta.rolldownVersion === "string";
      },
    });
    const result = (await build({
      root,
      configFile: false,
      logLevel: "silent",
      plugins,
      resolve: {
        alias: [
          { find: /^vue$/u, replacement: require.resolve("vue") },
          { find: "#imports", replacement: path.join(root, "injected.ts") },
          { find: "bare-package", replacement: path.join(root, "bare.ts") },
        ],
      },
      build: { write: false, minify: false, rollupOptions: { input: path.join(root, "main.ts") } },
    })) as Rollup.RollupOutput;
    return { seen, nativeFilters, output: result.output };
  };
  try {
    const baseline = await run(false);
    const filtered = await run(true);
    assert.deepEqual(filtered.output, baseline.output);
    assert.equal(baseline.seen.filter((source) => source.startsWith("./ordinary")).length, 40);
    assert.equal(
      filtered.seen.filter((source) => source.startsWith("./ordinary")).length,
      filtered.nativeFilters ? 0 : 40,
    );
    for (const source of [
      "./local",
      path.join(root, "injected.ts"),
      path.join(root, "bare.ts"),
    ].map((source) => source.replaceAll("\\", "/"))) {
      assert.equal(baseline.seen.includes(source), true, `exercise the SFC's ${source} import`);
      assert.equal(filtered.seen.includes(source), true, `retain the SFC's ${source} import`);
    }
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});
