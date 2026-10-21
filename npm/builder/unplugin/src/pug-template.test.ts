// `<template lang="pug">` through real Rollup and webpack builds renders exactly
// like the same component written with an HTML template
// (npm/builder/shared/pug-twins.cjs).
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { pathToFileURL } from "node:url";
import { rollup } from "rollup";
import webpack from "webpack";
import "./test/setup.ts";
import vizeRollup from "./rollup.ts";
import vizeWebpack from "./webpack.ts";
import { prepareOutputDir } from "./test/helpers.ts";

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

function project(name: string, source: string): string {
  const root = prepareOutputDir(name);
  fs.writeFileSync(path.join(root, "App.vue"), source);
  fs.writeFileSync(path.join(root, "entry.js"), 'export { default } from "./App.vue";\n');
  return root;
}

async function renderWithRollup(name: string, source: string): Promise<string> {
  const root = project(`rollup-${name}`, source);
  const bundle = await rollup({
    input: path.join(root, "entry.js"),
    external: ["vue"],
    plugins: [vizeRollup({ isProduction: true })],
  });
  const { output } = await bundle.generate({ format: "es" });
  await bundle.close();
  const file = path.join(root, "app.mjs");
  fs.writeFileSync(
    file,
    output[0].code.replace(
      /from ["']vue["']/g,
      `from ${JSON.stringify(pathToFileURL(vueRuntime).href)}`,
    ),
  );
  const { default: App } = (await import(pathToFileURL(file).href)) as { default: object };
  return renderToString(createSSRApp(App));
}

async function renderWithWebpack(name: string, source: string): Promise<string> {
  const root = project(`webpack-${name}`, source);
  const compiler = webpack({
    mode: "development",
    devtool: false,
    target: "node",
    entry: path.join(root, "entry.js"),
    externals: { vue: `commonjs ${vueRuntime}` },
    output: { path: root, filename: "app.cjs", library: { type: "commonjs2" } },
    infrastructureLogging: { level: "error" },
    plugins: [vizeWebpack({ isProduction: true })],
  });
  const stats = await new Promise<webpack.Stats>((resolve, reject) => {
    compiler.run((error, result) => {
      compiler.close(() => (error || !result ? reject(error) : resolve(result)));
    });
  });
  if (stats.hasErrors()) {
    throw new Error(JSON.stringify(stats.toJson({ all: false, errors: true }).errors, null, 2));
  }
  const App = (require(path.join(root, "app.cjs")) as { default: object }).default;
  return renderToString(createSSRApp(App));
}

for (const [bundler, render] of [
  ["rollup", renderWithRollup],
  ["webpack", renderWithWebpack],
] as const) {
  void test(`${bundler}: a pug template renders like its HTML twin`, async (t: TestContext) => {
    const html = await render("html", twins.html);
    t.assert.equal(await render("pug", twins.pug), html);
    t.assert.equal(await render("jade", twins.jade), html);
    t.assert.match(html, /^<section id="app" class="card active">/);
  });
}
