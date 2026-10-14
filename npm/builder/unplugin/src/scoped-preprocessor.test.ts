// Real-bundler order checks for `<style scoped lang="scss">`: the preprocessor
// runs first and scoping second, as in Vue and Vite. The fixture's stand-in
// preprocessor (test/fixtures/scoped-preprocessor/fake-scss.cjs) flattens
// nesting the way Sass does, so scoping before it would yield descendant
// selectors such as `[data-v-x] .root` and leave `:deep()` unresolved.
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test, type TestContext } from "node:test";
import webpack from "webpack";
import { rollup } from "rollup";
import "./test/setup.ts";
import vizeWebpack from "./webpack.ts";
import vizeRollup from "./rollup.ts";
import { createFakePreprocessorRoot } from "./test/fake-preprocessors.ts";
import { packageRoot, prepareOutputDir, resolveFixturePath } from "./test/helpers.ts";

const FIXTURE = "scoped-preprocessor";
// css-loader is pinned for the sibling rspack package; reuse it rather than
// adding a devDependency here.
const CSS_LOADER = createRequire(path.join(packageRoot, "..", "rspack", "package.json")).resolve(
  "css-loader",
);

function assertVueScopedCss(t: TestContext, css: string) {
  const scope = css.match(/\[(data-v-[0-9a-f]+)\]/)?.[1];
  t.assert.ok(scope, `no scope attribute in:\n${css}`);
  // Root-element rule: the attribute sits on the root's own compound selector.
  t.assert.ok(css.includes(`.root[${scope}]`), `root rule not scoped in place:\n${css}`);
  t.assert.ok(!css.includes(`[${scope}] .root`), `root rule became a descendant:\n${css}`);
  // Descendant rule: only the last compound carries the attribute.
  t.assert.ok(css.includes(`.root .title[${scope}]`), `descendant rule mis-scoped:\n${css}`);
  // :deep() moves the attribute before the deep part and disappears.
  t.assert.ok(css.includes(`.root[${scope}] .inner`), `:deep() not resolved:\n${css}`);
  t.assert.ok(!css.includes(":deep("), `:deep() left in output:\n${css}`);
  t.assert.ok(!css.includes("$accent"), `preprocessor did not run:\n${css}`);
  // A plain CSS sibling block is scoped too.
  t.assert.ok(css.includes(`.plain[${scope}]`), `plain scoped block not scoped:\n${css}`);
}

function runWebpack(compiler: webpack.Compiler): Promise<webpack.Stats> {
  return new Promise((resolve, reject) => {
    compiler.run((error, stats) => {
      compiler.close((closeError) => {
        if (error || closeError || !stats) {
          reject(error ?? closeError ?? new Error("webpack returned no stats"));
          return;
        }
        resolve(stats);
      });
    });
  });
}

void test("webpack: host preprocessor loader runs before scoping and css-loader", async (t: TestContext) => {
  const outputPath = prepareOutputDir("webpack-scoped-preprocessor");
  const compiler = webpack({
    mode: "development",
    devtool: false,
    target: ["web", "es2022"],
    entry: resolveFixturePath(FIXTURE, "entry.ts"),
    externals: { vue: "vue" },
    output: { path: outputPath, filename: "bundle.js", clean: true },
    infrastructureLogging: { level: "error" },
    module: {
      rules: [
        {
          test: /\.css$/,
          use: [{ loader: CSS_LOADER, options: { exportType: "string" } }],
        },
        {
          test: /\.scss$/,
          use: [
            { loader: CSS_LOADER, options: { exportType: "string" } },
            resolveFixturePath(FIXTURE, "fake-scss-loader.cjs"),
          ],
        },
      ],
    },
    plugins: [vizeWebpack({ isProduction: true })],
    resolve: { extensions: [".ts", ".js", ".vue"] },
  });

  const stats = await runWebpack(compiler);
  if (stats.hasErrors()) {
    throw new Error(JSON.stringify(stats.toJson({ all: false, errors: true }).errors, null, 2));
  }
  const bundle = fs.readFileSync(path.join(outputPath, "bundle.js"), "utf8");
  assertVueScopedCss(t, bundle);
});

void test("rollup: the project's preprocessor runs before scoping", async (t: TestContext) => {
  const root = createFakePreprocessorRoot("rollup-scoped-preprocessor");

  const styles: string[] = [];
  const styleFiles: string[] = [];
  const bundle = await rollup({
    input: resolveFixturePath(FIXTURE, "entry.ts"),
    external: ["vue"],
    plugins: [
      vizeRollup({ isProduction: true, root }),
      {
        name: "collect-css",
        transform(code, id) {
          if (!id.includes("type=style")) return null;
          styleFiles.push(id.split("?")[0]);
          styles.push(code);
          return { code: "export default undefined;", map: null };
        },
      },
    ],
  });
  await bundle.close();

  t.assert.equal(styles.length, 2);
  for (const file of styleFiles) {
    t.assert.ok(file.endsWith(".css"), `style id is not plain CSS: ${file}`);
  }
  assertVueScopedCss(t, styles.join("\n"));
});
