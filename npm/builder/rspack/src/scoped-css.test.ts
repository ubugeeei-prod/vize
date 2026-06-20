import { test } from "node:test";
import { rspack } from "@rspack/core";
import "./test/setup.ts";
import { VizePlugin } from "./plugin/index.ts";
import {
  normalizeSnapshot,
  packageLoaderAliases,
  prepareOutputDir,
  resolveFixturePath,
} from "./test/helpers.ts";

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

function createScopedCompiler(fixtureName: string, outputName: string): ReturnType<typeof rspack> {
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
    resolveLoader: {
      alias: packageLoaderAliases,
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
              loader: "@vizejs/rspack-plugin/loader",
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
}

function extractAssets(stats: Awaited<ReturnType<typeof runCompiler>>): Record<string, string> {
  return Object.fromEntries(
    Object.entries(stats.compilation.assets)
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([name, asset]) => [name, normalizeSnapshot(asset.source().toString())]),
  );
}

function getCssAsset(assets: Record<string, string>): string | undefined {
  return Object.entries(assets).find(([name]) => name.endsWith(".css"))?.[1];
}

function getJsBundle(assets: Record<string, string>): string | undefined {
  return Object.entries(assets).find(([name]) => name.endsWith(".js"))?.[1];
}

// ---------------------------------------------------------------------------
// Test: basic scoped CSS — selectors, pseudo-classes, pseudo-elements, comma
// ---------------------------------------------------------------------------

void test("scoped: basic selectors, :hover, ::before, comma groups", async (t) => {
  const compiler = createScopedCompiler("scoped-basic", "scoped-basic");
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const assets = extractAssets(stats);
  const css = getCssAsset(assets);

  t.assert.ok(css, "should produce a CSS asset");
  t.assert.snapshot(JSON.stringify(assets, null, 2));
});

// ---------------------------------------------------------------------------
// Test: :deep(), :global(), :slotted()
// ---------------------------------------------------------------------------

void test("scoped: :deep(), :global(), :slotted() semantics", async (t) => {
  const compiler = createScopedCompiler("scoped-deep-global-slotted", "scoped-deep-global-slotted");
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const assets = extractAssets(stats);
  const css = getCssAsset(assets);

  t.assert.ok(css, "should produce a CSS asset");
  t.assert.snapshot(JSON.stringify(assets, null, 2));
});

// ---------------------------------------------------------------------------
// Test: nested @media, @supports, @keyframes
// ---------------------------------------------------------------------------

void test("scoped: @media, @supports, @keyframes preserved and selectors scoped inside", async (t) => {
  const compiler = createScopedCompiler("scoped-at-rules", "scoped-at-rules");
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const assets = extractAssets(stats);
  const css = getCssAsset(assets);

  t.assert.ok(css, "should produce a CSS asset");
  t.assert.snapshot(JSON.stringify(assets, null, 2));
});

// ---------------------------------------------------------------------------
// Test: v-bind() CSS variables extraction
// ---------------------------------------------------------------------------

void test("scoped: v-bind() replaced with CSS variables", async (t) => {
  const compiler = createScopedCompiler("scoped-v-bind", "scoped-v-bind");
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const assets = extractAssets(stats);
  const css = getCssAsset(assets);

  t.assert.ok(css, "should produce a CSS asset");
  t.assert.snapshot(JSON.stringify(assets, null, 2));
});

// ---------------------------------------------------------------------------
// Test: multiple style blocks — scoped + module + unscoped coexist
// ---------------------------------------------------------------------------

void test("scoped: multiple style blocks (scoped + module + unscoped) coexist", async (t) => {
  const compiler = createScopedCompiler("scoped-multi-blocks", "scoped-multi-blocks");
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const assets = extractAssets(stats);
  const css = getCssAsset(assets);
  const js = getJsBundle(assets);

  t.assert.ok(css, "should produce a CSS asset");
  t.assert.ok(js, "should produce a JS bundle");

  t.assert.snapshot(JSON.stringify(assets, null, 2));
});
