import { test } from "node:test";
import fs from "node:fs";
import path from "node:path";
import webpack from "webpack";
import "./test/setup.ts";
import vize from "./webpack.ts";
import { injectWebpackVueDefines } from "./unplugin.ts";
import { normalizeSnapshot, prepareOutputDir, resolveFixturePath } from "./test/helpers.ts";

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
  t.assert.snapshot(normalizeSnapshot(code));
});
