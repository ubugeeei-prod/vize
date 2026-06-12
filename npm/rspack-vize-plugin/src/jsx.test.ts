import { test } from "node:test";
import path from "node:path";
import { rspack } from "@rspack/core";
import "./test/setup.ts";
import { VizePlugin } from "./plugin/index.ts";
import { packageRoot, prepareOutputDir, resolveFixturePath } from "./test/helpers.ts";

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

/** Build a fixture routing `.jsx`/`.tsx` through the dist JSX loader. */
function createJsxCompiler(
  fixtureName: string,
  outputName: string,
  vapor: boolean,
): ReturnType<typeof rspack> {
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
    infrastructureLogging: {
      level: "error",
    },
    resolve: {
      extensions: ["...", ".ts", ".js", ".jsx", ".tsx"],
    },
    module: {
      rules: [
        {
          test: /\.ts$/,
          loader: "builtin:swc-loader",
          options: { jsc: { parser: { syntax: "typescript" } } },
        },
        // Route `.jsx`/`.tsx` Vue components through the Vize JSX loader.
        {
          test: /\.[jt]sx$/,
          use: [
            {
              loader: path.join(packageRoot, "dist", "loader", "jsx-loader.mjs"),
              options: { vapor },
            },
          ],
        },
      ],
    },
    plugins: [new VizePlugin({ vapor })],
  });
}

function bundleSource(stats: Awaited<ReturnType<typeof runCompiler>>): string {
  return Object.values(stats.compilation.assets)
    .map((asset) => asset.source().toString())
    .join("\n");
}

void test("jsx: a .jsx Vue component compiles to VDOM render code", async (t) => {
  const compiler = createJsxCompiler("jsx-vdom", "jsx-vdom", false);
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const bundle = bundleSource(stats);
  t.assert.ok(
    bundle.includes("_createElementBlock"),
    "VDOM bundle should contain _createElementBlock",
  );
});

void test("jsx: a .jsx <style scoped> block emits scope-rewritten CSS into the bundle", async (t) => {
  // The JSX loader emits the component's `<style scoped>` CSS through the same
  // inline-style injection the integrations use for plain SFC CSS, so the
  // rewritten CSS (scope id baked into the selector) lands in the bundle and the
  // `data-v-<hash>` attribute is injected at runtime (#1495, #1533).
  const compiler = createJsxCompiler("jsx-scoped", "jsx-scoped", false);
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const bundle = bundleSource(stats);
  t.assert.ok(
    bundle.includes("__vize_css__"),
    "bundle should carry the inline-style injection for the scoped CSS",
  );
  const scopeMatch = bundle.match(/data-v-[0-9a-f]+/);
  t.assert.ok(scopeMatch, "the emitted CSS should carry a data-v- scope id");
  t.assert.ok(
    bundle.includes(`.jsx-scoped-box[${scopeMatch[0]}]`),
    "the emitted CSS should apply the scope id to the component selector",
  );
});

void test("jsx: a vapor .jsx component compiles to template render code", async (t) => {
  const compiler = createJsxCompiler("jsx-vapor", "jsx-vapor", true);
  const stats = await runCompiler(compiler);

  if (stats.hasErrors()) {
    const info = stats.toJson({ all: false, errors: true });
    throw new Error(JSON.stringify(info.errors, null, 2));
  }

  const bundle = bundleSource(stats);
  // The vapor path emits a `_template(...)` call importing `template` from
  // `vue`; rspack rewrites the externalized import (e.g.
  // `(0,vue__rspack_import_0.template)(...)`). The vapor output never goes
  // through the VDOM `_createElementBlock` block helper, and it inlines the
  // component markup into a static template string.
  t.assert.ok(bundle.includes(".template)"), "vapor bundle should contain a template() call");
  t.assert.ok(
    bundle.includes("jsx-vapor-app") && bundle.includes("hello vapor jsx"),
    "vapor bundle should inline the component markup into a template string",
  );
  t.assert.ok(
    !bundle.includes("_createElementBlock"),
    "vapor bundle should not contain VDOM _createElementBlock",
  );
});
