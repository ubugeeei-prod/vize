// #11568: a `<style src>` file in another directory resolves its relative
// `@import`, `@use` and `url()` from its own directory, as when the file is
// imported directly and as with vue-loader, whose module is the src file.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { rspack } from "@rspack/core";
import "./test/setup.ts";
import { VizePlugin } from "./plugin/index.ts";
import { packageLoaderAliases, packageRoot } from "./test/helpers.ts";

// A sass stand-in that resolves `@use` from the importing file's `url`, as Sass does.
const { installStandIns } = createRequire(import.meta.url)(
  path.join(packageRoot, "../shared/test-preprocessors.cjs"),
) as { installStandIns: (root: string) => void };

function project(t: TestContext, extra: Record<string, string> = {}): string {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-rspack-style-src-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  installStandIns(root);
  const files: Record<string, string> = {
    "src/styles/b.css": ".b { color: blue; }\n",
    "src/styles/img.svg": '<svg xmlns="http://www.w3.org/2000/svg"/>\n',
    "src/styles/_vars.scss": "$c: green;\n",
    "src/styles/a.css": '@import "./b.css";\n.a { background: url(./img.svg); }\n',
    "src/styles/s.scss": '@use "./vars" as v;\n.s { color: v.$c; background: url(./img.svg); }\n',
    "src/styles/m.css": ".m { color: teal; }\n",
    "src/components/Foo.vue": `<template><p :class="['a', 'b', 's', $style.m]">x</p></template>
<style src="../styles/a.css"></style>
<style scoped lang="scss" src="../styles/s.scss"></style>
<style module src="../styles/m.css"></style>
`,
    "src/entry.js": 'export { default } from "./components/Foo.vue";\n',
    "src/control.js": 'import "./styles/a.css";\nimport "./styles/s.scss";\n',
    ...extra,
  };
  for (const [file, content] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(path.join(root, file), content);
  }
  // A sass-loader stand-in, resolving from the resource as sass-loader does.
  fs.writeFileSync(
    path.join(root, "sass-loader.cjs"),
    `const sass = require(${JSON.stringify(path.join(root, "node_modules/sass"))});\nconst { pathToFileURL } = require("node:url");\nmodule.exports = function (source) { return sass.compileString(source, { url: pathToFileURL(this.resourcePath) }).css; };\n`,
  );
  return root;
}

const cssLoader = {
  loader: createRequire(path.join(packageRoot, "package.json")).resolve("css-loader"),
  options: { exportType: "string" },
};

function build(
  root: string,
  entry: string,
  native = true,
): Promise<{ css: string; js: string; errors: string[] }> {
  const output = path.join(root, `dist-${path.basename(entry, ".js")}-${native}`);
  const sassLoader = path.join(root, "sass-loader.cjs");
  const styleRules = native
    ? [
        { test: /\.css$/, type: "css/auto" as const },
        { test: /\.scss$/, use: [sassLoader], type: "css/auto" as const },
      ]
    : [
        { test: /\.css$/, use: [cssLoader], type: "javascript/auto" as const },
        { test: /\.scss$/, use: [cssLoader, sassLoader], type: "javascript/auto" as const },
      ];
  const compiler = rspack({
    mode: "production",
    devtool: false,
    optimization: { minimize: false },
    context: root,
    entry: { main: path.join(root, entry) },
    output: { path: output, filename: "bundle.js", cssFilename: "bundle.css", clean: true },
    externals: { vue: "vue" },
    experiments: { css: native },
    infrastructureLogging: { level: "error" },
    resolve: { extensions: ["...", ".js", ".vue"] },
    resolveLoader: { alias: packageLoaderAliases },
    module: {
      rules: [
        { test: /\.vue$/, use: [{ loader: "@vizejs/rspack-plugin/loader" }] },
        ...styleRules,
        { test: /\.svg$/, type: "asset/resource", generator: { filename: "[name][ext]" } },
      ],
    },
    plugins: [new VizePlugin({ css: { native } })],
  });
  return new Promise((resolve, reject) => {
    compiler.run((error, stats) => {
      compiler.close(() => {
        if (error || !stats) return reject(error);
        const read = (file: string) =>
          fs.existsSync(path.join(output, file))
            ? fs.readFileSync(path.join(output, file), "utf8")
            : "";
        resolve({
          css: read("bundle.css"),
          js: read("bundle.js"),
          errors: (stats.toJson({ errors: true }).errors ?? []).map((item) => item.message),
        });
      });
    });
  });
}

void test("#11568 rspack resolves a src block's @import, @use and url() from its file", async (t) => {
  const root = project(t);
  const control = await build(root, "src/control.js");
  const actual = await build(root, "src/entry.js");
  assert.deepEqual(control.errors, []);
  assert.deepEqual(actual.errors, []);
  for (const { css } of [control, actual]) {
    assert.match(css, /\.b\s*\{\s*color: blue;?\s*\}/);
    assert.match(css, /\.a\s*\{\s*background: url\("?img\.svg"?\);?\s*\}/);
  }
  assert.match(
    actual.css,
    /\.s\[data-v-[0-9a-f]{8}\]\s*\{\s*color: green;\s*background: url\("?img\.svg"?\);?\s*\}/,
  );
  // The CSS Module block still maps its class.
  const local = /\.([\w-]+)\s*\{\s*color: teal;?\s*\}/.exec(actual.css)?.[1];
  assert.ok(local && local !== "m", actual.css);
  assert.match(actual.js, new RegExp(`"${local}"`));

  // Negative control: an import missing beside the src file still fails.
  const missing = await build(
    project(t, { "src/styles/s.scss": '@use "./missing" as v;\n.s { color: v.$c; }\n' }),
    "src/entry.js",
  );
  assert.match(missing.errors.join("\n"), /Can't find stylesheet to import/);
});

void test("#11568 rspack with css-loader resolves a src block's references from its file", async (t) => {
  const root = project(t, {
    "src/components/Foo.vue": `<template><p class="a s">x</p></template>
<style src="../styles/a.css"></style>
<style scoped lang="scss" src="../styles/s.scss"></style>
`,
  });
  const { js, errors } = await build(root, "src/entry.js", false);
  assert.deepEqual(errors, []);
  assert.match(js, /\.b \{ color: blue; \}/);
  assert.match(js, /\.a \{ background: url\(\$\{___CSS_LOADER_URL_REPLACEMENT_0___\}\); \}/);
  assert.match(js, /\.s\[data-v-[0-9a-f]{8}\]\s*\{\s*color: green;/);
});
