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
      compiler.run((error, stats) => {
        compiler.close((closeError) => {
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

function createVaporCompiler(fixtureName: string, outputName: string): ReturnType<typeof rspack> {
  return rspack({
    mode: "development",
    devtool: false,
    context: resolveFixturePath(fixtureName, "."),
    entry: {
      main: resolveFixturePath(fixtureName, "entry.ts"),
    },
    output: {
      path: prepareOutputDir(outputName),
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
              options: {
                vapor: true,
              },
            },
          ],
        },
      ],
    },
    plugins: [
      new VizePlugin({
        vapor: true,
        css: {
          native: true,
        },
      }),
    ],
  });
}

function extractAssets(stats: Awaited<ReturnType<typeof runCompiler>>): Record<string, string> {
  return Object.fromEntries(
    Object.entries(stats.compilation.assets)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([name, asset]) => [name, normalizeSnapshot(asset.source().toString())]),
  );
}

void test("vapor: script setup SFC compiles successfully", async (t) => {
  const compiler = createVaporCompiler("vapor-script-setup", "vapor-script-setup");
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const assets = extractAssets(stats);
  const jsBundle = Object.values(assets).find((v) => v.includes("vapor"));
  t.assert.ok(jsBundle, "bundle should contain vapor-related output");
  t.assert.snapshot(JSON.stringify(assets, null, 2));
});

void test("vapor: scoped style coexists with vapor mode", async (t) => {
  const compiler = createVaporCompiler("vapor-scoped-style", "vapor-scoped-style");
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const assets = extractAssets(stats);

  // Verify vapor output exists
  const jsBundle = Object.values(assets).find((v) => v.includes("vapor"));
  t.assert.ok(jsBundle, "bundle should contain vapor-related output");

  // Verify scoped style is present (CSS asset or inline)
  const hasStyle = Object.values(assets).some(
    (v) => v.includes("scoped-app") || v.includes("data-v-"),
  );
  t.assert.ok(hasStyle, "bundle should contain scoped style output");

  t.assert.snapshot(JSON.stringify(assets, null, 2));
});

void test("vapor: HMR injection does not break __vapor marker", async (t) => {
  // The dev-mode build (with HMR) should preserve the __vapor flag
  const compiler = createVaporCompiler("vapor-script-setup", "vapor-hmr");
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const assets = extractAssets(stats);
  const jsBundle = Object.values(assets).find((v) => v.includes("__vapor"));
  t.assert.ok(jsBundle, "HMR build should preserve __vapor marker in output");
});
