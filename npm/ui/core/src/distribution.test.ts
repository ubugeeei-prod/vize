import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { test } from "node:test";

const packageDirectory = new URL("../", import.meta.url);

const publicEntries = [
  ".",
  "./button",
  "./checkbox",
  "./controllable-state",
  "./primitive",
  "./visually-hidden",
] as const;

void test("every public entry exposes generated JavaScript and declarations", async () => {
  const manifest = JSON.parse(
    await readFile(new URL("package.json", packageDirectory), "utf8"),
  ) as {
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
    await assert.doesNotReject(
      readFile(new URL(target.import.replace(/^\.\//, ""), packageDirectory)),
    );
    await assert.doesNotReject(
      readFile(new URL(target.types.replace(/^\.\//, ""), packageDirectory)),
    );
  }
});

void test("required accessibility CSS follows its component entry", async () => {
  const entry = await readFile(new URL("dist/visually-hidden.mjs", packageDirectory), "utf8");
  const componentPath = entry.match(/from ["'](\.\/visually-hidden-[^"']+)["']/)?.[1];
  assert.ok(componentPath, "component chunk must be reachable from its public entry");

  const component = await readFile(
    new URL(`dist/${componentPath.replace(/^\.\//, "")}`, packageDirectory),
    "utf8",
  );
  assert.match(component, /import\s*["']\.\/style\.css["'];?/);

  const style = await readFile(new URL("dist/style.css", packageDirectory), "utf8");
  assert.match(style, /data-vize-ui=["']?visually-hidden["']?/);
});
