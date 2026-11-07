// #11522: a `<style src>` file in another directory resolves its relative
// `@import`, `@use` and `url()` from its own directory on every host, as when
// the host imports that file directly.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";
import { pathToFileURL } from "node:url";
import { build as esbuild } from "esbuild";
import webpack from "webpack";
import vizeEsbuild from "./esbuild.ts";
import { parseVueRequest } from "./request.ts";
import { vizeUnplugin } from "./unplugin.ts";
import vizeWebpack from "./webpack.ts";
import { packageRoot, workspaceRoot } from "./test/helpers.ts";

// A sass stand-in that resolves `@use` from the importing file's `url`, as
// Sass does; css-loader is pinned for the sibling rspack package.
const standIns = path.join(packageRoot, "../shared/test-preprocessors.cjs");
const { installStandIns } = createRequire(import.meta.url)(standIns) as {
  installStandIns: (root: string) => void;
};
const cssLoader = createRequire(path.join(packageRoot, "../rspack/package.json")).resolve(
  "css-loader",
);

const SFC =
  '<template><p class="a b s">x</p></template>\n<style src="../styles/a.css"></style>\n<style scoped lang="scss" src="../styles/s.scss"></style>\n';

function project(t: TestContext, extra: Record<string, string> = {}): string {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-style-src-dir-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const files: Record<string, string> = {
    "package.json": '{ "private": true, "type": "module" }\n',
    "src/styles/b.css": ".b { color: blue; }\n",
    "src/styles/img.svg": '<svg xmlns="http://www.w3.org/2000/svg"/>\n',
    "src/styles/_vars.scss": "$c: green;\n",
    "src/styles/a.css": '@import "./b.css";\n.a { background: url(./img.svg); }\n',
    "src/styles/s.scss": '@use "./vars" as v;\n.s { color: v.$c; background: url(./img.svg); }\n',
    "src/components/Foo.vue": SFC,
    "src/index.ts": 'export { default as Foo } from "./components/Foo.vue";\n',
    ...extra,
  };
  for (const [file, content] of Object.entries(files)) {
    fs.mkdirSync(path.dirname(path.join(root, file)), { recursive: true });
    fs.writeFileSync(path.join(root, file), content);
  }
  installStandIns(root);
  fs.writeFileSync(path.join(root, "package.json"), files["package.json"]);
  return root;
}

const missingUse = { "src/styles/s.scss": '@use "./missing" as v;\n.s { color: v.$c; }\n' };

void test("#11522 the style module sits beside its src file and names its SFC", (t) => {
  const root = project(t);
  const plugin = vizeUnplugin.raw({ isProduction: true, root }, { framework: "rollup" }) as {
    transform(this: unknown, code: string, id: string): Promise<{ code: string }>;
    transformInclude(id: string): boolean;
    load(this: unknown, id: string): Promise<{ code: string }>;
  };
  const sfc = path.join(root, "src/components/Foo.vue");
  return (async () => {
    const watched: string[] = [];
    const ctx = { warn() {}, addWatchFile: (file: string) => watched.push(file) };
    const { code } = await plugin.transform.call(ctx, SFC, sfc);
    const ids = [...code.matchAll(/^import "([^"]+)";$/gm)].map((match) => match[1]);
    const css = path.join(root, "src/styles/a.css");
    assert.equal(
      ids[0],
      `${css}.__vize_style_0?vue&type=style&index=0&owner=${encodeURIComponent(sfc)}&lang.css`,
    );
    // Not an SFC to the plugin, and not a file on disk: the owner comes from the query.
    assert.equal(plugin.transformInclude(ids[0]), false);
    assert.equal(parseVueRequest(ids[0]).filename, sfc);
    fs.writeFileSync(`${css}.__vize_style_0`, ".decoy { color: red; }");
    watched.length = 0;
    const loaded = await plugin.load.call(ctx, ids[0]);
    assert.match(loaded.code, /@import "\.\/b\.css";\n\.a \{ background: url\(\.\/img\.svg\); \}/);
    assert.doesNotMatch(loaded.code, /decoy/);
    // The real src file is watched for rebuilds.
    assert.deepEqual(watched, [css]);
  })();
});

void test("#11522 tsdown compiles a src block's scss `@use` from the src file's directory", (t) => {
  const requireFromUnplugin = createRequire(
    path.join(workspaceRoot, "npm/builder/unplugin/package.json"),
  );
  const tsdown = requireFromUnplugin.resolve("tsdown");
  const cli = requireFromUnplugin.resolve("tsdown/run");
  const config = (entry: string, plugins: string) =>
    `import Vize from ${JSON.stringify(pathToFileURL(path.join(import.meta.dirname, "rolldown.ts")).href)};
import { defineConfig } from ${JSON.stringify(pathToFileURL(tsdown).href)};
export default defineConfig({ deps: { neverBundle: ["vue"] }, dts: false, entry: { index: ${JSON.stringify(entry)} }, format: "esm", plugins: ${plugins} });
`;
  const run = (root: string) => {
    const result = spawnSync(process.execPath, [cli, "--config", "tsdown.config.ts"], {
      cwd: root,
      encoding: "utf8",
      timeout: 60_000,
    });
    const css = path.join(root, "dist", "style.css");
    return { result, css: fs.existsSync(css) ? fs.readFileSync(css, "utf8") : "" };
  };

  const root = project(t, {
    "tsdown.config.ts": config("src/index.ts", "[Vize({ isProduction: true })]"),
  });
  const { result, css } = run(root);
  assert.equal(result.status, 0, `${result.stdout}${result.stderr}`);
  assert.match(
    css,
    /\.s\[data-v-[0-9a-f]{8}\]\s*\{\s*color: green;\s*background: url\(\.\/img\.svg\);?\s*\}/,
  );
  // As for the file imported directly, the url stays relative to the src file;
  // @tsdown/css passes a virtual stylesheet's @import through (as with
  // unplugin-vue), so it is still the authored one.
  assert.match(css, /@import "\.\/b\.css";\n\.a \{ background: url\(\.\/img\.svg\); \}/);

  // Negative control: an import missing beside the src file still fails.
  const missing = project(t, {
    ...missingUse,
    "tsdown.config.ts": config("src/index.ts", "[Vize({ isProduction: true })]"),
  });
  const failed = run(missing).result;
  assert.notEqual(failed.status, 0);
  assert.match(`${failed.stdout}${failed.stderr}`, /Can't find stylesheet to import/);
});

void test("#11522 esbuild resolves a src block's @import, @use and url() from its file", async (t) => {
  const build = (root: string) =>
    esbuild({
      entryPoints: [path.join(root, "src/index.ts")],
      bundle: true,
      write: false,
      outdir: path.join(root, "dist"),
      format: "esm",
      external: ["vue"],
      loader: { ".svg": "file" },
      logLevel: "silent",
      plugins: [vizeEsbuild({ isProduction: true, root })],
    });
  const root = project(t);
  const result = await build(root);
  const css = result.outputFiles.find((file) => file.path.endsWith(".css"))?.text ?? "";
  const asset = result.outputFiles.find((file) => file.path.endsWith(".svg"));
  assert.ok(asset, result.outputFiles.map((file) => file.path).join(","));
  const url = `url("./${path.basename(asset.path)}")`;
  assert.match(css, /\.b \{\n {2}color: blue;\n\}/);
  assert.ok(css.includes(`.a {\n  background: ${url};\n}`), css);
  assert.match(css, /\.s\[data-v-[0-9a-f]{8}\] \{\n {2}color: green;/);
  assert.ok(css.includes(`  background: ${url};\n}`), css);

  // Negative control: an import missing beside the src file still fails.
  await assert.rejects(build(project(t, missingUse)), /Can't find stylesheet to import/);
});

void test("#11522 webpack resolves a src block's @import, @use and url() from its file", async (t) => {
  const build = (root: string, plugin: boolean, entry: string) =>
    new Promise<{ bundle: string; errors: string[] }>((resolve, reject) => {
      fs.writeFileSync(
        path.join(root, "sass-loader.cjs"),
        // A sass-loader stand-in, resolving from the resource as sass-loader does.
        `const sass = require(${JSON.stringify(path.join(root, "node_modules/sass"))});\nconst { pathToFileURL } = require("node:url");\nmodule.exports = function (source) { return sass.compileString(source, { url: pathToFileURL(this.resourcePath) }).css; };\n`,
      );
      const output = path.join(root, plugin ? "dist" : "dist-control");
      const compiler = webpack({
        mode: "production",
        devtool: false,
        optimization: { minimize: false },
        context: root,
        entry: path.join(root, entry),
        externals: { vue: "vue" },
        output: { path: output, filename: "bundle.js", assetModuleFilename: "[name][ext]" },
        infrastructureLogging: { level: "error" },
        module: {
          rules: [
            { test: /\.css$/, use: [{ loader: cssLoader, options: { exportType: "string" } }] },
            {
              test: /\.scss$/,
              use: [
                { loader: cssLoader, options: { exportType: "string" } },
                path.join(root, "sass-loader.cjs"),
              ],
            },
            { test: /\.svg$/, type: "asset/resource" },
          ],
        },
        plugins: plugin ? [vizeWebpack({ isProduction: true, root })] : [],
        resolve: { extensions: [".ts", ".js", ".vue"] },
      });
      compiler.run((error, stats) =>
        compiler.close(() => {
          if (error || !stats) return reject(error);
          const errors = (stats.toJson({ all: false, errors: true }).errors ?? []).map(
            (item) => item.message,
          );
          const file = path.join(output, "bundle.js");
          resolve({ bundle: fs.existsSync(file) ? fs.readFileSync(file, "utf8") : "", errors });
        }),
      );
    });
  // The control imports the same files directly.
  const root = project(t, {
    "src/control.js": 'import "./styles/a.css";\nimport "./styles/s.scss";\n',
  });
  const control = await build(root, false, "src/control.js");
  const actual = await build(root, true, "src/index.ts");
  assert.deepEqual(actual.errors, []);
  assert.deepEqual(control.errors, []);
  for (const { bundle } of [control, actual]) {
    assert.match(bundle, /\.b \{ color: blue; \}/);
    assert.match(bundle, /\.a \{ background: url\(\$\{___CSS_LOADER_URL_REPLACEMENT_0___\}\); \}/);
    assert.match(bundle, /__webpack_require__\.p \+ "img\.svg"/);
  }
  assert.match(actual.bundle, /\.s\[data-v-[0-9a-f]{8}\]\{color: green;/);

  // Negative control: an import missing beside the src file still fails.
  const missing = await build(project(t, missingUse), true, "src/index.ts");
  assert.match(missing.errors.join("\n"), /Can't find stylesheet to import/);
});
