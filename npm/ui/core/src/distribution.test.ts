import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import { test } from "vite-plus/test";

import { uiFamilyCatalog } from "./family-catalog.ts";

const publicEntries = [
  ".",
  "./catalog",
  "./theme-scope",
  ...uiFamilyCatalog.map((entry) => entry.packageSubpath),
] as const;

test("every public entry exposes generated JavaScript and declarations", async () => {
  const manifest = JSON.parse(await readFile(path.resolve("package.json"), "utf8")) as {
    exports: Record<string, string | { default: string; import: string; types: string }>;
    sideEffects: string[];
  };

  assert.deepEqual(manifest.sideEffects, ["./dist/*.css"]);
  assert.equal(manifest.exports["./style.css"], "./dist/style.css");

  for (const entry of publicEntries) {
    const target = manifest.exports[entry];
    assert.equal(typeof target, "object", `${entry} must be a conditional export`);
    if (typeof target !== "object") continue;

    assert.equal(target.import, target.default);
    await assert.doesNotReject(readFile(path.resolve(target.import)));
    await assert.doesNotReject(readFile(path.resolve(target.types)));
  }
});

test("required accessibility CSS follows its component entry", async () => {
  const entry = await readFile(path.resolve("dist/visually-hidden.mjs"), "utf8");
  const componentPath = entry.match(/from ["'](\.\/visually-hidden-[^"']+)["']/)?.[1];
  assert.ok(componentPath, "component chunk must be reachable from its public entry");

  const component = await readFile(path.resolve("dist", componentPath), "utf8");
  assert.match(component, /import\s*["']\.\/style\.css["'];?/);

  const style = await readFile(path.resolve("dist/style.css"), "utf8");
  assert.match(style, /data-vize-ui=["']?visually-hidden["']?/);
});
