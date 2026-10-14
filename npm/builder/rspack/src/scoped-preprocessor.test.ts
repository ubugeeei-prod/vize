// Real-bundler order check for `<style scoped lang="scss">`: the host's
// preprocessor loader runs first and the scope loader second, as in Vue. The
// stand-in preprocessor flattens nesting the way Sass does, so scoping before
// it would yield `[data-v-x] .root` and leave `:deep()` unresolved.
import path from "node:path";
import { test, type TestContext } from "node:test";
import { rspack } from "@rspack/core";
import "./test/setup.ts";
import { VizePlugin } from "./plugin/index.ts";
import {
  packageLoaderAliases,
  packageRoot,
  prepareOutputDir,
  resolveFixturePath,
} from "./test/helpers.ts";

const FIXTURE = "scoped-preprocessor";
// Shared with @vizejs/unplugin's scoped-preprocessor test.
const FAKE_SCSS_LOADER = path.join(
  packageRoot,
  "..",
  "unplugin",
  "src",
  "test",
  "fixtures",
  "scoped-preprocessor",
  "fake-scss-loader.cjs",
);

function build(native: boolean): Promise<string> {
  const outputPath = prepareOutputDir(`scoped-preprocessor-${native ? "native" : "css-loader"}`);
  const cssLoader = { loader: "css-loader", options: { exportType: "string" } };
  const styleRules = native
    ? [{ test: /\.scss$/, use: [FAKE_SCSS_LOADER], type: "css/auto" as const }]
    : [
        { test: /\.css$/, use: [cssLoader], type: "javascript/auto" as const },
        { test: /\.scss$/, use: [cssLoader, FAKE_SCSS_LOADER], type: "javascript/auto" as const },
      ];
  const compiler = rspack({
    mode: "development",
    devtool: false,
    context: resolveFixturePath(FIXTURE, "."),
    entry: { main: resolveFixturePath(FIXTURE, "entry.ts") },
    output: { path: outputPath, filename: "bundle.js", clean: true },
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
        ...styleRules,
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
        resolve(
          Object.values(stats.compilation.assets)
            .map((asset) => asset.source().toString())
            .join("\n"),
        );
      });
    });
  });
}

function assertVueScopedCss(t: TestContext, css: string) {
  const scope = css.match(/\[(data-v-[0-9a-f]+)\]/)?.[1];
  t.assert.ok(scope, `no scope attribute in:\n${css}`);
  t.assert.ok(css.includes(`.root[${scope}]`), "root rule not scoped in place");
  t.assert.ok(!css.includes(`[${scope}] .root`), "root rule became a descendant");
  t.assert.ok(css.includes(`.root .title[${scope}]`), "descendant rule mis-scoped");
  t.assert.ok(css.includes(`.root[${scope}] .inner`), ":deep() not resolved");
  t.assert.ok(!css.includes(":deep("), ":deep() left in output");
  t.assert.ok(css.includes(`.plain[${scope}]`), "plain scoped block not scoped");
}

void test("rspack native CSS: preprocessor loader runs before the scope loader", async (t: TestContext) => {
  assertVueScopedCss(t, await build(true));
});

void test("rspack css-loader: scope loader sits between preprocessor and css-loader", async (t: TestContext) => {
  assertVueScopedCss(t, await build(false));
});
