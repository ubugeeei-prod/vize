import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { build } from "esbuild";
import "./test/setup.ts";
import vize from "./esbuild.ts";
import { createFakePreprocessorRoot } from "./test/fake-preprocessors.ts";
import { normalizeSnapshot, prepareOutputDir, resolveFixturePath } from "./test/helpers.ts";

void test("esbuild bundles a basic SFC", async (t) => {
  const result = await build({
    entryPoints: [resolveFixturePath("basic", "entry.ts")],
    bundle: true,
    write: false,
    format: "esm",
    platform: "browser",
    outdir: "out",
    external: ["vue"],
    plugins: [vize({ isProduction: true })],
  });

  const jsAsset = result.outputFiles.find(
    (file) => !file.path.endsWith(".css") && !file.path.endsWith(".map"),
  );
  t.assert.ok(jsAsset, "expected a JavaScript output file");
  t.assert.match(jsAsset.text, /Hello from Vize unplugin/);
  t.assert.doesNotMatch(jsAsset.text, /document\.createElement\("style"\)/);
  const css = result.outputFiles.find((file) => file.path.endsWith(".css"))?.text ?? "";
  t.assert.match(css, /\.root\[data-v-[0-9a-f]{8}\] \{\n  color: tomato;/);
  t.assert.snapshot(normalizeSnapshot(jsAsset.text));
});

void test("esbuild without bundle injects plain CSS, which nothing would import", async (t) => {
  const result = await build({
    entryPoints: [resolveFixturePath("basic", "App.vue")],
    bundle: false,
    write: false,
    format: "esm",
    outdir: "out",
    plugins: [vize({ isProduction: true })],
  });

  t.assert.deepEqual(
    result.outputFiles.map((file) => file.path.endsWith(".css")),
    [false],
  );
  const js = result.outputFiles[0].text;
  t.assert.match(js, /document\.createElement\("style"\)/);
  t.assert.match(js, /\.root\[data-v-[0-9a-f]{8}\]\{\\n  color: tomato;/);
  t.assert.doesNotMatch(js, /type=style/);
});

void test("esbuild loads delegated style blocks: preprocessed, then scoped", async (t) => {
  const result = await build({
    entryPoints: [resolveFixturePath("scoped-preprocessor", "entry.ts")],
    bundle: true,
    write: false,
    format: "esm",
    platform: "browser",
    outdir: "out",
    external: ["vue"],
    plugins: [vize({ isProduction: true, root: createFakePreprocessorRoot("esbuild-styles") })],
  });

  const css = result.outputFiles.find((file) => file.path.endsWith(".css"))?.text ?? "";
  const scope = /\[(data-v-[0-9a-f]+)\]/.exec(css)?.[1];
  t.assert.ok(scope, `no scoped CSS in:\n${css}`);
  t.assert.deepEqual(
    css
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.includes("{")),
    [
      `.root[${scope}] {`,
      `.root .title[${scope}] {`,
      `.root[${scope}] .inner {`,
      `.plain[${scope}] {`,
    ],
  );
});

// #11513: esbuild has no preprocessor pipeline that can see a plugin-namespace
// module, so an unscoped preprocessor block must reach it already compiled.
void test("esbuild compiles an unscoped preprocessor block before its CSS loader", async (t) => {
  const root = createFakePreprocessorRoot("esbuild-unscoped-scss");
  const app = path.join(root, "App.vue");
  fs.writeFileSync(
    app,
    '<template><p class="a">x</p></template>\n<style lang="scss">\n$c: red;\n.a {\n  color: $c;\n}\n</style>\n',
  );
  const options = {
    entryPoints: [app],
    bundle: true,
    write: false,
    format: "esm" as const,
    outdir: "out",
    external: ["vue"],
    logLevel: "silent" as const,
  };
  const result = await build({ ...options, plugins: [vize({ isProduction: true, root })] });
  t.assert.deepEqual(
    result.warnings.map((warning) => warning.text),
    [],
  );
  const css = result.outputFiles.find((file) => file.path.endsWith(".css"))?.text ?? "";
  t.assert.match(css, /\.a \{\n  color: red;\n\}/);
  t.assert.doesNotMatch(css, /\$c/);

  // Negative control: with no preprocessor installed, the build still fails.
  const bare = prepareOutputDir("esbuild-unscoped-scss-bare");
  fs.writeFileSync(path.join(bare, "package.json"), "{}\n");
  fs.copyFileSync(app, path.join(bare, "App.vue"));
  await t.assert.rejects(
    build({
      ...options,
      entryPoints: [path.join(bare, "App.vue")],
      plugins: [vize({ isProduction: true, root: bare })],
    }),
    /<style lang="scss"> needs sass or sass-embedded installed/,
  );
});

// #11521: esbuild without `bundle` never follows an import, so a block it
// cannot inject must fail instead of leaving a dangling style import.
void test("esbuild without bundle injects CSS modules and rejects preprocessor blocks", async (t) => {
  const root = prepareOutputDir("esbuild-unbundled-blocks");
  const app = path.join(root, "App.vue");
  const options = {
    entryPoints: [app],
    bundle: false,
    write: false,
    format: "esm" as const,
    outdir: path.join(root, "out"),
    logLevel: "silent" as const,
    plugins: [vize({ isProduction: true, root })],
  };
  fs.writeFileSync(
    app,
    '<template><p :class="$style.a">x</p></template>\n<style module>\n.a { color: green; }\n</style>\n',
  );
  const result = await build(options);
  const js = result.outputFiles[0].text;
  t.assert.doesNotMatch(js, /type=style/);
  const local = /"a": "([\w-]+)"/.exec(js)?.[1];
  t.assert.ok(local, js);
  t.assert.match(js, new RegExp(`__vize_css__ = "\\.${local} \\{`));

  // Negative control: nothing here compiles Sass, so the block is an error.
  fs.writeFileSync(
    app,
    '<template><p class="a">x</p></template>\n<style lang="scss">\n$c: red;\n.a { color: $c; }\n</style>\n',
  );
  await t.assert.rejects(build(options), /<style lang="scss"> needs a bundler CSS pipeline/);
});
