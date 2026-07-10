import { test } from "node:test";
import { build } from "esbuild";
import "./test/setup.ts";
import vize from "./esbuild.ts";
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
