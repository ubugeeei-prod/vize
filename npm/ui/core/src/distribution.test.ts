import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import { test } from "vite-plus/test";

import { uiFamilyCatalog } from "./family-catalog.ts";
import { themePresets } from "./theme.ts";
import type {
  ThemePresetName,
  ThemePresetStylesheetSpecifier,
  ThemeTokenStylesheetSpecifier,
} from "./theme.ts";

const publicEntries = [
  ".",
  "./catalog",
  "./theme-scope",
  ...uiFamilyCatalog.map((entry) => entry.packageSubpath),
] as const;

const themeTokenStylesheet = "./theme.css" satisfies ThemeTokenStylesheetSpecifier;
function themePresetStylesheet<Name extends ThemePresetName>(
  name: Name,
): ThemePresetStylesheetSpecifier<Name> {
  return `./theme-preset-${name}.css`;
}

const publicThemeCssEntries = [
  themeTokenStylesheet,
  ...themePresets.map((name) => themePresetStylesheet(name)),
] as const;

test("every public entry exposes generated JavaScript and declarations", async () => {
  const manifest = JSON.parse(await readFile(path.resolve("package.json"), "utf8")) as {
    exports: Record<string, string | { default: string; import: string; types: string }>;
    sideEffects: string[];
  };

  assert.deepEqual(manifest.sideEffects, ["./dist/*.css"]);
  assert.equal(manifest.exports["./style.css"], "./dist/style.css");

  for (const entry of publicThemeCssEntries) {
    const target = manifest.exports[entry];
    assert.equal(typeof target, "string", `${entry} must be a CSS-only export`);
    if (typeof target !== "string") continue;

    assert.equal(target, `./dist/${entry.slice(2)}`);
    assert.doesNotMatch(target.slice("./dist/".length), /\//, `${entry} must match sideEffects`);
    await assert.doesNotReject(readFile(path.resolve(target)));
  }

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

test("theme CSS entrypoints are lowered CSS assets without runtime entry shims", async () => {
  const manifest = JSON.parse(await readFile(path.resolve("package.json"), "utf8")) as {
    exports: Record<string, string | { default: string; import: string; types: string }>;
  };
  const tokenTarget = manifest.exports[themeTokenStylesheet];
  assert.equal(typeof tokenTarget, "string");
  if (typeof tokenTarget !== "string") return;

  const tokenStylesheet = await readFile(path.resolve(tokenTarget), "utf8");
  assert.match(tokenStylesheet, /^@layer vize\.tokens,vize\.ui,vize\.preset,vize\.policy;/);
  assert.match(tokenStylesheet, /@layer vize\.tokens\{/);
  assert.match(tokenStylesheet, /@layer vize\.policy\{/);
  assert.doesNotMatch(tokenStylesheet, /@layer vize\.preset\{/);

  for (const name of themePresets) {
    const entry = themePresetStylesheet(name);
    const target = manifest.exports[entry];
    assert.equal(typeof target, "string", `${entry} must be a CSS-only export`);
    if (typeof target !== "string") continue;

    const stylesheet = await readFile(path.resolve(target), "utf8");
    assert.match(stylesheet, /^@layer vize\.tokens,vize\.ui,vize\.preset,vize\.policy;/);
    assert.match(stylesheet, /@layer vize\.preset\{/);
    assert.match(stylesheet, new RegExp(`data-vize-theme~=${name}`));
    assert.doesNotMatch(stylesheet, /@layer vize\.tokens\{/);
    assert.doesNotMatch(stylesheet, /\b(?:import|export)\b/);
  }
});
