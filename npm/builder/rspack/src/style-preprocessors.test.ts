// Real Rspack builds of SFCs whose scoped and module blocks use scss, sass,
// less or stylus, through css-loader and through native CSS. The host's
// preprocessor loader (the stand-in in npm/builder/shared) must run before
// Vize scopes; `v-bind()` must read the `useCssVars` variable and `$style`
// must be the class map css-loader or Rspack generated.
import fs from "node:fs";
import vm from "node:vm";
import { createRequire } from "node:module";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { rspack } from "@rspack/core";
import "./test/setup.ts";
import { VizePlugin } from "./plugin/index.ts";
import { packageLoaderAliases, packageRoot, prepareOutputDir } from "./test/helpers.ts";

const require = createRequire(import.meta.url);
const CSS_LOADER = require.resolve("css-loader");
const STAND_IN_LOADER = path.join(packageRoot, "..", "shared", "test-preprocessor-loader.cjs");

const BRACED = {
  scoped: `.root { color: v-bind(accent); .title { color: blue; } :deep(.inner) { color: green; } }`,
  module: `.badge { .icon { color: red; } }`,
};
const INDENTED = {
  scoped: `.root\n  color: v-bind(accent)\n  .title\n    color: blue\n  :deep(.inner)\n    color: green\n`,
  module: `.badge\n  .icon\n    color: red\n`,
};
const LANGS: Record<string, { body: typeof BRACED; test: RegExp }> = {
  scss: { body: BRACED, test: /\.scss$/ },
  sass: { body: INDENTED, test: /\.sass$/ },
  less: { body: BRACED, test: /\.less$/ },
  stylus: { body: INDENTED, test: /\.styl(us)?$/ },
};

function sfc(lang: string): string {
  const { body } = LANGS[lang];
  return `<script setup lang="ts">
const accent = "red";
</script>

<template>
  <div class="root"><p class="title">x</p><span :class="$style.badge" /></div>
</template>

<style scoped lang="${lang}">
${body.scoped}
</style>

<style module lang="${lang}">
${body.module}
</style>
`;
}

type Built = { css: string; js: string; modules: Record<string, Record<string, string>> };

/** Evaluate the bundle with the real Vue runtime and read the component's CSS modules. */
function cssModulesOf(js: string): Record<string, Record<string, string>> {
  const context = vm.createContext({ vue: require("vue"), console });
  vm.runInContext(js, context);
  const app = (context as { __app?: { default?: { __cssModules?: Record<string, never> } } }).__app;
  return app?.default?.__cssModules ?? {};
}

function build(lang: string, native: boolean): Promise<Built> {
  const name = `style-preprocessors-${lang}-${native ? "native" : "css-loader"}`;
  const root = prepareOutputDir(name);
  fs.writeFileSync(path.join(root, "App.vue"), sfc(lang));
  fs.writeFileSync(path.join(root, "entry.ts"), 'export { default } from "./App.vue";\n');
  const cssLoader = {
    loader: CSS_LOADER,
    // As vue-loader documents: enable modules for `<style module>` requests.
    options: {
      modules: {
        auto: (_resourcePath: string, resourceQuery: string) => /[?&]module/.test(resourceQuery),
        namedExport: false,
      },
    },
  };
  const styleRule = native
    ? { test: LANGS[lang].test, use: [STAND_IN_LOADER], type: "css/auto" as const }
    : {
        test: LANGS[lang].test,
        use: [cssLoader, STAND_IN_LOADER],
        type: "javascript/auto" as const,
      };
  const compiler = rspack({
    mode: "development",
    devtool: false,
    context: root,
    entry: { main: path.join(root, "entry.ts") },
    output: {
      path: path.join(root, "dist"),
      filename: "bundle.js",
      clean: true,
      library: { type: "var", name: "__app" },
    },
    externals: { vue: "vue" },
    experiments: { css: native },
    infrastructureLogging: { level: "error" },
    resolve: { extensions: ["...", ".ts", ".js", ".vue"] },
    resolveLoader: { alias: packageLoaderAliases },
    module: {
      rules: [
        {
          test: /\.ts$/,
          loader: "builtin:swc-loader",
          options: { jsc: { parser: { syntax: "typescript" } } },
        },
        {
          test: /\.vue$/,
          resourceQuery: { not: [/type=/] },
          enforce: "post" as const,
          loader: "builtin:swc-loader",
          options: { jsc: { parser: { syntax: "typescript" } } },
          type: "javascript/auto",
        },
        { test: /\.vue$/, use: [{ loader: "@vizejs/rspack-plugin/loader" }] },
        styleRule,
      ],
    },
    plugins: [new VizePlugin({ css: { native } })],
  });

  return new Promise((resolve, reject) => {
    compiler.run((error, stats) => {
      compiler.close((closeError) => {
        if (error || closeError || !stats) {
          reject(error ?? closeError ?? new Error("rspack returned no stats"));
          return;
        }
        if (stats.hasErrors()) {
          reject(new Error(JSON.stringify(stats.toJson({ errors: true }).errors, null, 2)));
          return;
        }
        const assets = Object.entries(stats.compilation.assets).map(([file, asset]) => ({
          file,
          source: asset.source().toString(),
        }));
        const js = assets.find((asset) => asset.file.endsWith(".js"))?.source ?? "";
        const css = native
          ? assets
              .filter((asset) => asset.file.endsWith(".css"))
              .map((asset) => asset.source)
              .join("\n")
          : [...js.matchAll(/___CSS_LOADER_EXPORT___\.push\(\[module\.id, `([^`]*)`/g)]
              .map((match) => match[1].replaceAll("\\n", "\n"))
              .join("\n");
        resolve({ css, js, modules: cssModulesOf(js) });
      });
    });
  });
}

function selectors(css: string): string[] {
  return [...css.replace(/\/\*[^]*?\*\//g, "").matchAll(/([^{}]+)\{[^{}]*\}/g)]
    .map((match) => match[1].trim())
    .filter(Boolean);
}

for (const lang of Object.keys(LANGS)) {
  for (const native of [false, true]) {
    const mode = native ? "native CSS" : "css-loader";
    void test(`rspack ${mode} + ${lang}: preprocessed, then scoped; v-bind and $style line up`, async (t: TestContext) => {
      const { css, js, modules } = await build(lang, native);
      const scope = /\[(data-v-[0-9a-f]+)\]/.exec(css)?.[1];
      const varName = /useCssVars\)[^]*?"([\w-]+)": accent/.exec(js)?.[1];
      const { badge, icon } = modules.$style ?? {};
      t.assert.ok(
        scope && varName && badge && icon,
        `missing pieces: ${JSON.stringify({ scope, varName, modules })}\n${css}`,
      );
      t.assert.deepEqual(selectors(css), [
        `.root[${scope}]`,
        `.root .title[${scope}]`,
        `.root[${scope}] .inner`,
        `.${badge} .${icon}`,
      ]);
      t.assert.match(css, new RegExp(`color: var\\(--${varName}\\)`));
    });
  }
}
