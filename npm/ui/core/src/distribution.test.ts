import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

const packageDirectory = new URL("../", import.meta.url);

void test("preserves required accessibility CSS in the public entry", async () => {
  const manifest = JSON.parse(
    await readFile(new URL("package.json", packageDirectory), "utf8"),
  ) as {
    exports: Record<string, string | { default: string; import: string; types: string }>;
    sideEffects: string[];
  };

  assert.deepEqual(manifest.sideEffects, ["./dist/*.css"]);
  assert.equal(manifest.exports["./style.css"], "./dist/style.css");

  const entry = await readFile(new URL("dist/index.mjs", packageDirectory), "utf8");
  assert.match(entry, /import\s*["']\.\/style\.css["'];?/);

  const style = await readFile(new URL("dist/style.css", packageDirectory), "utf8");
  assert.match(style, /data-vize-ui=["']?visually-hidden["']?/);
});
