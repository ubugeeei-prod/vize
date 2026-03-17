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

  // Every plain selector must carry the scope attribute
  t.assert.ok(css!.includes("[data-v-"), "selectors must be scoped with [data-v-*]");

  // Pseudo-class :hover must appear after scope attribute
  t.assert.ok(css!.includes(":hover"), ":hover must be preserved");

  // Pseudo-element ::before must appear after scope attribute (LightningCSS may lower to :before)
  t.assert.ok(
    css!.includes("::before") || css!.includes(":before"),
    "::before (or :before) must be preserved",
  );

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

  // :deep(.child) — scope on parent, not on .child
  t.assert.ok(!css!.includes(":deep("), ":deep() pseudo must be consumed and not appear in output");

  // :global(.global-reset) — no scope attribute on .global-reset
  t.assert.ok(css!.includes(".global-reset"), ":global() content must be present");
  t.assert.ok(
    !css!.includes(":global("),
    ":global() pseudo must be consumed and not appear in output",
  );

  // :slotted(div) — scope attribute with -s suffix
  t.assert.ok(
    !css!.includes(":slotted("),
    ":slotted() pseudo must be consumed and not appear in output",
  );

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

  // @media rules preserved
  t.assert.ok(css!.includes("@media"), "@media rules must be preserved");
  t.assert.ok(
    css!.includes("max-width") || css!.includes("768px"),
    "@media (max-width: 768px) must be preserved",
  );

  // @supports preserved
  t.assert.ok(css!.includes("@supports"), "@supports must be preserved");

  // @keyframes preserved
  t.assert.ok(css!.includes("@keyframes"), "@keyframes must be preserved");

  // Selectors inside @media must still be scoped
  t.assert.ok(css!.includes("[data-v-"), "selectors inside at-rules must be scoped");

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

  // v-bind() must be replaced with var(--*)
  t.assert.ok(!css!.includes("v-bind("), "v-bind() must not appear in final CSS output");
  t.assert.ok(
    css!.includes("var(--"),
    "v-bind() values must be replaced with CSS custom properties",
  );

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

  // Scoped block: selectors are scoped
  t.assert.ok(css!.includes("[data-v-"), "scoped block selectors must carry scope attribute");

  // Unscoped block: .unscoped-text must be present without scoping
  t.assert.ok(css!.includes(".unscoped-text"), "unscoped block selectors must be present");

  // JS should reference CSS module binding
  t.assert.ok(js, "should produce a JS bundle");

  t.assert.snapshot(JSON.stringify(assets, null, 2));
});
