import { test } from "node:test";
import { build } from "esbuild";
import "./test/setup.ts";
import vize from "./esbuild.ts";
import { createFakePreprocessorRoot } from "./test/fake-preprocessors.ts";
import { normalizeSnapshot, resolveFixturePath } from "./test/helpers.ts";

void test("esbuild bundles a basic SFC", async (t) => {
  const result = await build({
    entryPoints: [resolveFixturePath("basic", "entry.ts")],
    bundle: true,
    write: false,
    format: "esm",
    platform: "browser",
    external: ["vue"],
    plugins: [vize({ isProduction: true })],
  });

  const jsAsset = result.outputFiles.find(
    (file) => !file.path.endsWith(".css") && !file.path.endsWith(".map"),
  );
  t.assert.ok(jsAsset, "expected a JavaScript output file");
  t.assert.match(jsAsset.text, /Hello from Vize unplugin/);
  t.assert.snapshot(normalizeSnapshot(jsAsset.text));
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
