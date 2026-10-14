// `v-bind()` in delegated style blocks (a preprocessor block forces every
// block of the SFC through the bundler) must become the CSS variables the
// compiled `useCssVars` call sets, scoped or not.
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

const FIXTURE = "delegated-v-bind";
const CSS_LOADER = createRequire(path.join(packageRoot, "..", "rspack", "package.json")).resolve(
  "css-loader",
);

function assertVarsMatch(t: TestContext, css: string, js: string) {
  const used = [...css.matchAll(/var\(--([\w-]+)\)/g)].map((match) => match[1]);
  // The key of the `useCssVars` getter object, however the bundler prints it.
  const set = [...js.matchAll(/useCssVars\)?\(\(?_ctx\)? => \(\{\s*"([\w-]+)": accent/g)].map(
    (match) => match[1],
  );
  t.assert.equal(set.length, 1, `no useCssVars call in:\n${js}`);
  // Both the scoped scss block and the plain global block read the same var.
  t.assert.deepEqual(used, [set[0], set[0]]);
  t.assert.ok(!css.includes("v-bind("), `v-bind() left in CSS:\n${css}`);
}

void test("rollup: delegated blocks resolve v-bind() to the useCssVars names", async (t: TestContext) => {
  for (const isProduction of [true, false]) {
    const styles: string[] = [];
    let js = "";
    const bundle = await rollup({
      input: resolveFixturePath(FIXTURE, "entry.ts"),
      external: ["vue"],
      plugins: [
        vizeRollup({ isProduction, root: createFakePreprocessorRoot("rollup-delegated-v-bind") }),
        {
          name: "collect",
          transform(code, id) {
            if (id.includes("type=style")) {
              styles.push(code);
              return { code: "export default undefined;", map: null };
            }
            if (id.endsWith("App.vue")) js = code;
            return null;
          },
        },
      ],
    });
    await bundle.close();
    assertVarsMatch(t, styles.join("\n"), js);
  }
});

void test("webpack: delegated style files resolve v-bind() to the useCssVars names", async (t: TestContext) => {
  const outputPath = prepareOutputDir("webpack-delegated-v-bind");
  const cssLoader = { loader: CSS_LOADER, options: { exportType: "string" } };
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
        { test: /\.css$/, use: [cssLoader] },
        {
          test: /\.scss$/,
          use: [cssLoader, resolveFixturePath("scoped-preprocessor", "fake-scss-loader.cjs")],
        },
      ],
    },
    plugins: [vizeWebpack({ isProduction: true })],
    resolve: { extensions: [".ts", ".js", ".vue"] },
  });
  const stats = await new Promise<webpack.Stats>((resolve, reject) => {
    compiler.run((error, result) => {
      compiler.close(() => (error || !result ? reject(error) : resolve(result)));
    });
  });
  if (stats.hasErrors()) {
    throw new Error(JSON.stringify(stats.toJson({ all: false, errors: true }).errors, null, 2));
  }
  const bundle = fs.readFileSync(path.join(outputPath, "bundle.js"), "utf8");
  const css = [...bundle.matchAll(/___CSS_LOADER_EXPORT___\.push\(\[module\.id, `([^`]*)`/g)]
    .map((match) => match[1])
    .join("\n");
  assertVarsMatch(t, css, bundle);
});
