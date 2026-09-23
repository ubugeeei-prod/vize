import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { rspack, type RuleSetRule, type Stats } from "@rspack/core";
import { VizePlugin } from "./index.ts";
import { packageLoaderAliases } from "../test/helpers.ts";
import type { VizeRspackPluginOptions } from "../types/index.ts";

const loader = "@vizejs/rspack-plugin/loader";
const source = `<script>export default { name: "ConfigProbe" }</script>
<template><div class="config-probe">Hello</div></template>
<style>.config-probe { color: rgb(11, 22, 33); }</style>`;

async function build(
  t: TestContext,
  vueRule: RuleSetRule,
  options: {
    plugin?: VizeRspackPluginOptions;
    rules?: RuleSetRule[];
    entry?: string;
    files?: Record<string, string>;
  } = {},
) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-rspack-config-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const files = {
    "entry.js": options.entry ?? 'import App from "./src/App.vue"; console.log(App);',
    "src/App.vue": source,
    ...options.files,
  };
  for (const [name, content] of Object.entries(files)) {
    const filename = path.join(root, name);
    fs.mkdirSync(path.dirname(filename), { recursive: true });
    fs.writeFileSync(filename, content);
  }

  const outputPath = path.join(root, "dist");
  const compiler = rspack({
    mode: "production",
    context: root,
    entry: "./entry.js",
    devtool: false,
    output: { path: outputPath, filename: "bundle.js" },
    optimization: { minimize: false, concatenateModules: false },
    experiments: { css: true },
    externals: {
      vue: "commonjs vue",
      "vue/server-renderer": "commonjs vue/server-renderer",
      "@vue/server-renderer": "commonjs @vue/server-renderer",
    },
    infrastructureLogging: { level: "error" },
    resolveLoader: { alias: packageLoaderAliases },
    module: { rules: [vueRule, ...(options.rules ?? [])] },
    plugins: [new VizePlugin({ typescript: false, css: { native: true }, ...options.plugin })],
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
  const info = stats.toJson({ all: false, errors: true, modules: true, source: true });
  assert.equal(stats.hasErrors(), false, JSON.stringify(info.errors, null, 2));
  const css = Object.keys(stats.compilation.assets)
    .filter((name) => name.endsWith(".css"))
    .map((name) => fs.readFileSync(path.join(outputPath, name), "utf8"))
    .join("\n");
  return { css, modules: info.modules ?? [] };
}

for (const cloneCssRule of [false, true]) {
  void test(`production retains used SFC styles with sideEffects: false (${cloneCssRule ? "cloned" : "fallback"} rules)`, async (t) => {
    const result = await build(
      t,
      { test: /\.vue$/, loader, sideEffects: false },
      {
        rules: cloneCssRule
          ? [{ test: /\.css$/, type: "css/auto", use: ["builtin:lightningcss-loader"] }]
          : [],
        entry:
          'import App from "./src/App.vue"; import Unused from "./src/Unused.vue"; console.log(App);',
        files: { "src/Unused.vue": source.replaceAll("config-probe", "unused-probe") },
      },
    );
    assert.match(result.css, /\.config-probe/);
    assert.doesNotMatch(result.css, /\.unused-probe/);
  });
}

for (const [name, conditions] of [
  ["issuer", { issuer: /\.js$/ }],
  ["empty resource query", { resourceQuery: /^$/ }],
] satisfies Array<[string, Partial<RuleSetRule>]>) {
  void test(`style sub-requests are independent of the main rule's ${name}`, async (t) => {
    const result = await build(t, { test: /\.vue$/, loader, ...conditions });
    assert.match(result.css, /\.config-probe/);
  });
}

for (const [name, conditions] of [
  ["issuer", { issuer: /other-entry\.js$/ }],
  ["resource query", { resourceQuery: /raw/ }],
] satisfies Array<[string, Partial<RuleSetRule>]>) {
  void test(`main requests still honor the original ${name} filter`, async (t) => {
    const result = await build(
      t,
      { test: /\.vue$/, loader, ...conditions },
      { rules: [{ test: /\.vue$/, type: "asset/source" }] },
    );
    const main = result.modules.find((module) => module.name === "./src/App.vue");
    assert.equal(main?.moduleType, "asset/source");
    assert.equal(main?.source, source);
    assert.equal(result.css, "");
  });
}

for (const [name, conditions, rejectedPath] of [
  ["include", { include: /[/\\]src[/\\]/ }, "outside/App.vue"],
  ["exclude", { exclude: /[/\\]excluded[/\\]/ }, "src/excluded/App.vue"],
  ["resource", { resource: /[/\\]src[/\\]App\.vue$/ }, "outside/App.vue"],
] satisfies Array<[string, Partial<RuleSetRule>, string]>) {
  void test(`the ${name} file filter applies to both main and style requests`, async (t) => {
    const query = "?vue&type=style&index=0&lang=css";
    const result = await build(
      t,
      { test: /\.vue$/, loader, ...conditions },
      {
        entry:
          `import App from "./src/App.vue";\n` +
          `import raw from "./${rejectedPath}";\n` +
          `import rawStyle from "./${rejectedPath}${query}";\n` +
          `console.log(App, raw, rawStyle);`,
        files: { [rejectedPath]: source },
        rules: [
          {
            test: /\.vue$/,
            include: (filename) => filename.replaceAll("\\", "/").endsWith(`/${rejectedPath}`),
            type: "asset/source",
          },
        ],
      },
    );
    assert.match(result.css, /\.config-probe/);
    for (const suffix of ["", query]) {
      const excluded = result.modules.find(
        (module) => module.name === `./${rejectedPath}${suffix}`,
      );
      assert.equal(excluded?.moduleType, "asset/source");
      assert.equal(excluded?.source, source);
    }
  });
}

for (const explicitCss of [false, true]) {
  void test(`manual JSON options preserve SSR and ${explicitCss ? "override" : "inherit"} the CSS mode`, async (t) => {
    const options = JSON.stringify({
      ssr: true,
      hotReload: false,
      ...(explicitCss ? { css: { native: false } } : {}),
    });
    const result = await build(
      t,
      {
        test: /\.vue$/,
        oneOf: [
          {
            resourceQuery: /type=style/,
            type: "css/module",
            parser: { namedExports: !explicitCss },
            use: ["@vizejs/rspack-plugin/style-loader"],
          },
          { loader, options },
        ],
      },
      {
        plugin: { autoRules: false },
        files: { "src/App.vue": source.replace("<style>", "<style module>") },
      },
    );
    const main = result.modules.find((module) => module.name === "./src/App.vue");
    const compiled = String(main?.source);
    assert.match(compiled, /\bssrRender\b/);
    assert.match(compiled, /import \* as _cssModule_0 from /);
    if (explicitCss) assert.match(compiled, /__vize_resolve_css_module__/);
    else assert.doesNotMatch(compiled, /__vize_resolve_css_module__/);
    assert.notEqual(result.css, "");
  });
}
