import { test } from "node:test";
import fs from "node:fs";
import path from "node:path";
import webpack from "webpack";
import "./test/setup.ts";
import vize from "./webpack.ts";
import { injectWebpackVueDefines } from "./unplugin.ts";
import { normalizeSnapshot, prepareOutputDir, resolveFixturePath } from "./test/helpers.ts";

const CSS_LOADER = path.join(import.meta.dirname, "test", "css-loader", "index.cjs");

function runWebpackCompiler(compiler: webpack.Compiler): Promise<webpack.Stats> {
  return new Promise((resolve, reject) => {
    compiler.run((error, stats) => {
      compiler.close((closeError) => {
        if (error || closeError) {
          reject(error ?? closeError);
          return;
        }

        if (!stats) {
          reject(new Error("Webpack did not return stats"));
          return;
        }

        resolve(stats);
      });
    });
  });
}

void test("webpack define injection supports a Webpack 4 compiler shape", (t) => {
  const appliedDefinitions: Array<Record<string, string>> = [];

  class FakeDefinePlugin {
    definitions: Record<string, string>;

    constructor(definitions: Record<string, string>) {
      this.definitions = definitions;
    }

    apply() {
      appliedDefinitions.push(this.definitions);
    }
  }

  const compiler = {
    options: {
      plugins: [
        {
          definitions: {
            __VUE_OPTIONS_API__: JSON.stringify(true),
          },
        },
      ],
    },
  } as unknown as webpack.Compiler;

  injectWebpackVueDefines(compiler, true, 4, FakeDefinePlugin);

  t.assert.deepStrictEqual(appliedDefinitions, [
    {
      __VUE_PROD_DEVTOOLS__: JSON.stringify(false),
      __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: JSON.stringify(false),
    },
  ]);
});

void test("webpack bundles a basic SFC", async (t) => {
  const outputPath = prepareOutputDir("webpack");
  const outputFile = path.join(outputPath, "bundle.js");
  const compiler = webpack({
    mode: "development",
    devtool: false,
    target: ["web", "es2022"],
    entry: resolveFixturePath("basic", "entry.ts"),
    externals: {
      vue: "vue",
    },
    output: {
      path: outputPath,
      filename: "bundle.js",
      clean: true,
    },
    infrastructureLogging: {
      level: "error",
    },
    module: {
      rules: [{ test: /\.css$/, use: [CSS_LOADER] }],
    },
    plugins: [vize({ isProduction: true })],
    resolve: {
      extensions: [".ts", ".js", ".vue"],
    },
  });

  const stats = await runWebpackCompiler(compiler);
  const info = stats.toJson({
    all: false,
    errors: true,
    assets: true,
  });

  if (stats.hasErrors()) {
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const code = fs.readFileSync(outputFile, "utf8");
  t.assert.match(code, /Hello from Vize unplugin/);
  // The block went through the host's `.css` rule, not a run-time injection.
  t.assert.doesNotMatch(code, /__vize_css__/);
  t.assert.match(code, /\.root\[data-v-[0-9a-f]{8}\]/);
  t.assert.snapshot(normalizeSnapshot(code));
});

void test("webpack without a .css rule fails on the style block instead of dropping it", async (t) => {
  const outputPath = prepareOutputDir("webpack-no-css-rule");
  const compiler = webpack({
    mode: "development",
    devtool: false,
    entry: resolveFixturePath("basic", "entry.ts"),
    externals: { vue: "vue" },
    output: { path: outputPath, filename: "bundle.js", clean: true },
    infrastructureLogging: { level: "error" },
    plugins: [vize({ isProduction: true })],
    resolve: { extensions: [".ts", ".js", ".vue"] },
  });

  const stats = await runWebpackCompiler(compiler);
  const errors = stats.toJson({ all: false, errors: true }).errors ?? [];
  t.assert.equal(errors.length, 1);
  t.assert.match(errors[0].moduleName ?? "", /App\.vue\.__vize_style_0\.data-v-[0-9a-f]{8}\.css$/);
  t.assert.match(errors[0].message, /Module parse failed[\s\S]*You may need an additional loader/);
});

void test("webpack: a JSX <style scoped> block goes through the host's .css rule (#11520)", async (t) => {
  const outputPath = prepareOutputDir("webpack-jsx-style");
  // A .jsx copy: webpack's experiments.typescript refuses any .tsx resource.
  const entry = path.join(outputPath, "ScopedApp.jsx");
  fs.copyFileSync(resolveFixturePath("jsx", "ScopedApp.tsx"), entry);
  const compiler = webpack({
    mode: "development",
    devtool: false,
    entry,
    externals: { vue: "vue" },
    output: { path: path.join(outputPath, "dist"), filename: "bundle.js", clean: true },
    infrastructureLogging: { level: "error" },
    module: { rules: [{ test: /\.css$/, use: [CSS_LOADER] }] },
    plugins: [vize({ isProduction: true })],
  });

  const stats = await runWebpackCompiler(compiler);
  t.assert.deepEqual(stats.toJson({ all: false, errors: true }).errors, []);
  const code = fs.readFileSync(path.join(outputPath, "dist", "bundle.js"), "utf8");
  t.assert.doesNotMatch(code, /__vize_css__/);
  t.assert.match(code, /ScopedApp\.jsx\.__vize_style_0\.css/);
  t.assert.match(
    code,
    /__WEBPACK_DEFAULT_EXPORT__ = \("\.box\[data-v-[0-9a-f]+\]\{ color: red \}"\)/,
  );
});
