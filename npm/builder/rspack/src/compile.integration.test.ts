import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { createRequire } from "node:module";
import { test, type TestContext } from "node:test";
import { rspack, type Stats } from "@rspack/core";
import { TraceMap, originalPositionFor } from "@jridgewell/trace-mapping";
import { createSSRApp, type Component } from "vue";
import { renderToString } from "vue/server-renderer";
import { VizePlugin } from "./plugin/index.ts";
import { packageLoaderAliases } from "./test/helpers.ts";
import type { SourceMapV3 } from "./shared/source-map.ts";

const require = createRequire(import.meta.url);
const source = `<script>
export default {
  name: "MapProbe",
  data() { return { message: "<mapping>" }; }
};
</script>
<template><section :class="$style.probe">{{ message }}</section></template>
<style module>.probe { color: rgb(11, 22, 33); }</style>
<style module="theme">.accent { font-weight: bold; }</style>`;

async function build(
  t: TestContext,
  options: { nativeCss?: boolean; namedExport?: boolean; sourceMap?: boolean; ssr?: boolean } = {},
) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-rspack-compile-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.writeFileSync(path.join(root, "App.vue"), source);
  fs.writeFileSync(path.join(root, "entry.js"), 'export { default } from "./App.vue";');
  const nativeCss = options.nativeCss ?? true;
  const outputPath = path.join(root, "dist");
  const compiler = rspack({
    mode: "production",
    target: options.ssr ? "node" : "web",
    context: root,
    entry: "./entry.js",
    devtool: "source-map",
    output: {
      path: outputPath,
      filename: "bundle.cjs",
      publicPath: "",
      library: { type: "commonjs2" },
    },
    optimization: { minimize: false, concatenateModules: false },
    experiments: { css: nativeCss },
    externals: {
      vue: `commonjs ${require.resolve("vue")}`,
      "vue/server-renderer": `commonjs ${require.resolve("vue/server-renderer")}`,
      "@vue/server-renderer": `commonjs ${require.resolve("vue/server-renderer")}`,
    },
    infrastructureLogging: { level: "error" },
    resolveLoader: { alias: packageLoaderAliases },
    module: {
      rules: [
        {
          test: /\.vue$/,
          loader: "@vizejs/rspack-plugin/loader",
          options: { ssr: options.ssr, sourceMap: options.sourceMap },
        },
        ...(!nativeCss
          ? [
              {
                test: /\.css$/,
                type: "javascript/auto",
                use: [
                  rspack.CssExtractRspackPlugin.loader,
                  {
                    loader: require.resolve("css-loader"),
                    options: {
                      modules: {
                        namedExport: options.namedExport,
                        localIdentName: "[local]__[hash:hex:6]",
                      },
                    },
                  },
                ],
              },
            ]
          : []),
      ],
    },
    plugins: [
      new VizePlugin({ typescript: false, css: { native: nativeCss } }),
      ...(!nativeCss ? [new rspack.CssExtractRspackPlugin({ filename: "styles.css" })] : []),
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
  assert.equal(stats.hasWarnings(), false, JSON.stringify(info.warnings, null, 2));
  const filename = path.join(outputPath, "bundle.cjs");
  const code = fs.readFileSync(filename, "utf8");
  const map = JSON.parse(fs.readFileSync(filename + ".map", "utf8")) as SourceMapV3;
  const component = require(filename).default as Component & {
    __cssModules: Record<string, Record<string, string>>;
  };
  const css = fs
    .readdirSync(outputPath)
    .filter((file) => file.endsWith(".css"))
    .map((file) => fs.readFileSync(path.join(outputPath, file), "utf8"))
    .join("\n");
  return { code, map, component, css };
}

function assertScriptMapping(code: string, map: SourceMapV3) {
  const marker = 'name: "MapProbe"';
  const offset = code.indexOf(marker);
  assert.notEqual(offset, -1);
  const prefix = code.slice(0, offset);
  const original = originalPositionFor(new TraceMap(JSON.stringify(map)), {
    line: prefix.split("\n").length,
    column: offset - prefix.lastIndexOf("\n") - 1,
  });
  assert.ok(original.source?.endsWith("/App.vue"));
  assert.equal(original.line, 3);
  assert.equal(original.column, 2);
  assert.ok(map.sourcesContent?.includes(source));
}

void test("final Rspack bundles map script lines and columns back to the Vue source", async (t) => {
  const result = await build(t);
  assertScriptMapping(result.code, result.map);
  assert.ok(result.component.__cssModules.$style.probe);
  assert.ok(result.css.includes(result.component.__cssModules.$style.probe));
});

void test("SSR output renders escaped content and retains its script source map", async (t) => {
  const result = await build(t, { ssr: true });
  assertScriptMapping(result.code, result.map);
  assert.doesNotMatch(result.code, /module\.hot/);
  const html = await renderToString(createSSRApp(result.component));
  assert.match(html, /&lt;mapping&gt;/);
  assert.ok(html.includes(result.component.__cssModules.$style.probe));
});

void test("sourceMap: false omits native Vue mappings even with Rspack devtool enabled", async (t) => {
  const result = await build(t, { sourceMap: false });
  assert.equal(result.map.sourcesContent?.includes(source), false);
});

for (const namedExport of [false, true]) {
  void test(`extracted CSS modules expose class names with css-loader ${namedExport ? "named" : "default"} exports`, async (t) => {
    const result = await build(t, { nativeCss: false, namedExport });
    assertScriptMapping(result.code, result.map);
    for (const [module, name] of [
      ["$style", "probe"],
      ["theme", "accent"],
    ]) {
      const className = result.component.__cssModules[module][name];
      assert.equal(typeof className, "string");
      assert.ok(className.length > 0);
      assert.ok(result.css.includes(className));
    }
  });
}
