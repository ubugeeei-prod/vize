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

function resolveDistLoaderPath(): string {
  return path.join(packageRoot, "dist", "loader", "index.mjs");
}

function resolvePackageLoaderPath(name: "scope-loader" | "style-loader"): string {
  return path.join(packageRoot, "dist", "loader", `${name}.mjs`);
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
    resolveLoader: {
      alias: {
        "@vizejs/rspack-plugin/scope-loader": resolvePackageLoaderPath("scope-loader"),
        "@vizejs/rspack-plugin/style-loader": resolvePackageLoaderPath("style-loader"),
      },
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
              loader: resolveDistLoaderPath(),
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

void test("rspack omits script setup imports used only in TypeScript positions", async (t) => {
  const compiler = rspack({
    mode: "development",
    devtool: false,
    context: resolveFixturePath("type-only-import-runtime", "."),
    entry: {
      main: resolveFixturePath("type-only-import-runtime", "entry.ts"),
    },
    output: {
      path: prepareOutputDir("type-only-import-runtime"),
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
    resolveLoader: {
      alias: {
        "@vizejs/rspack-plugin/scope-loader": resolvePackageLoaderPath("scope-loader"),
        "@vizejs/rspack-plugin/style-loader": resolvePackageLoaderPath("style-loader"),
      },
    },
    module: {
      rules: [
        {
          test: /\.ts$/,
          loader: "builtin:swc-loader",
          options: {
            jsc: { parser: { syntax: "typescript" } },
          },
        },
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
        {
          test: /\.vue$/,
          use: [
            {
              loader: resolveDistLoaderPath(),
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

  const bundle = Object.values(stats.compilation.assets)
    .map((asset) => normalizeSnapshot(asset.source().toString()))
    .join("\n");

  t.assert.ok(!bundle.includes("BadgeType"), bundle);
});
