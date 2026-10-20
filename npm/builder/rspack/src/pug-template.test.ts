// `<template lang="pug">` through a real Rspack build renders exactly like the
// same component written with an HTML template (npm/builder/shared/pug-twins.cjs).
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { rspack } from "@rspack/core";
import "./test/setup.ts";
import { VizePlugin } from "./plugin/index.ts";
import { packageLoaderAliases, prepareOutputDir } from "./test/helpers.ts";

const require = createRequire(import.meta.url);
const twins = require("../../shared/pug-twins.cjs") as {
  pug: string;
  jade: string;
  html: string;
};
const vueRuntime = require.resolve("vue");
const { renderToString } = require("vue/server-renderer") as {
  renderToString: (app: unknown) => Promise<string>;
};
const { createSSRApp } = require("vue") as typeof import("vue");

function render(name: string, source: string): Promise<string> {
  const root = prepareOutputDir(`pug-template-${name}`);
  fs.writeFileSync(path.join(root, "App.vue"), source);
  fs.writeFileSync(path.join(root, "entry.ts"), 'export { default } from "./App.vue";\n');
  const compiler = rspack({
    mode: "development",
    devtool: false,
    target: "node",
    context: root,
    entry: { main: path.join(root, "entry.ts") },
    output: { path: root, filename: "app.cjs", library: { type: "commonjs2" } },
    externals: { vue: `commonjs ${vueRuntime}` },
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
        { test: /\.vue$/, use: [{ loader: "@vizejs/rspack-plugin/loader" }] },
      ],
    },
    plugins: [new VizePlugin()],
  });
  return new Promise((resolve, reject) => {
    compiler.run((error, stats) => {
      compiler.close(() => {
        if (error || !stats || stats.hasErrors()) {
          reject(error ?? new Error(JSON.stringify(stats?.toJson({ errors: true }).errors)));
          return;
        }
        const App = (require(path.join(root, "app.cjs")) as { default: object }).default;
        resolve(renderToString(createSSRApp(App)));
      });
    });
  });
}

void test("rspack: a pug template renders like its HTML twin", async (t: TestContext) => {
  const html = await render("html", twins.html);
  t.assert.equal(await render("pug", twins.pug), html);
  t.assert.equal(await render("jade", twins.jade), html);
  t.assert.match(html, /^<section id="app" class="card active">/);
});
