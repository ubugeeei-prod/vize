import { test } from "node:test";
import path from "node:path";
import { rspack } from "@rspack/core";
import "./test/setup.ts";
import { VizePlugin } from "./plugin/index.js";
import {
  normalizeSnapshot,
  packageRoot,
  prepareOutputDir,
  resolveFixturePath,
} from "./test/helpers.js";

function runCompiler(compiler: ReturnType<typeof rspack>) {
  return new Promise<NonNullable<Parameters<Parameters<typeof compiler.run>[0]>[1]>>(
    (resolve, reject) => {
      compiler!.run((error, stats) => {
        compiler!.close((closeError) => {
          if (error || closeError) {
            reject(error ?? closeError);
            return;
          }

          if (!stats) {
            reject(new Error("Rspack did not return stats"));
            return;
          }

          resolve(stats);
        });
      });
    },
  );
}

void test("rspack builds a Vue SFC with auto-inject mode", async (t) => {
  const compiler = rspack({
    mode: "development",
    devtool: false,
    context: resolveFixturePath("basic", "."),
    entry: {
      main: resolveFixturePath("basic", "entry.ts"),
    },
    output: {
      path: prepareOutputDir("integration"),
      filename: "bundle.js",
      clean: true,
    },
    externals: {
      vue: "vue",
    },
    experiments: {
      css: true,
    },
    infrastructureLogging: {
      level: "error",
    },
    resolve: {
      extensions: ["...", ".ts", ".js", ".vue"],
    },
    module: {
      rules: [
        // TypeScript support
        {
          test: /\.ts$/,
          loader: "builtin:swc-loader",
          options: {
            jsc: { parser: { syntax: "typescript" } },
          },
        },
        // TypeScript post-processing for .vue files
        {
          test: /\.vue$/,
          resourceQuery: { not: [/type=/] },
          enforce: "post" as const,
          loader: "builtin:swc-loader",
          options: {
            jsc: { parser: { syntax: "typescript" } },
          },
          type: "javascript/auto",
        },
        // Simple .vue rule — VizePlugin auto-injects style sub-request handling
        {
          test: /\.vue$/,
          use: [
            {
              loader: path.join(packageRoot, "dist", "loader", "index.mjs"),
            },
          ],
        },
      ],
    },
    plugins: [
      new VizePlugin({
        css: {
          native: true,
        },
      }),
    ],
  });

  const stats = await runCompiler(compiler);
  const info = stats.toJson({
    all: false,
    errors: true,
    assets: true,
  });

  if (stats.hasErrors()) {
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const assets = Object.fromEntries(
    Object.entries(stats.compilation.assets)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([name, asset]) => [name, normalizeSnapshot(asset.source().toString())]),
  );

  t.assert.snapshot(JSON.stringify(assets, null, 2));
});
