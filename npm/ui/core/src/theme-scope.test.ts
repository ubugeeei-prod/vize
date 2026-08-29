import assert from "node:assert/strict";
import { runInNewContext } from "node:vm";
import { test } from "vite-plus/test";

import { themeDensityAttribute, themePresetAttribute } from "./theme.ts";
import {
  applyThemeScope,
  createThemeBootstrapScript,
  themeScopeAttributes,
  themeScopeStorageKeys,
} from "./theme-scope.ts";
import type { ThemeDensityScale, ThemePresetName } from "./theme-scope.ts";

function runBootstrapScript(script: string): void {
  runInNewContext(script, {
    document,
    globalThis: { localStorage },
  });
}

test("normalizes nested theme scope attributes and restores imperative scopes", () => {
  assert.deepEqual(themeScopeAttributes(), {});
  assert.deepEqual(
    themeScopeAttributes({
      presets: ["atelier", "paper", "atelier", "high-contrast"],
      density: "comfortable",
    }),
    {
      [themePresetAttribute]: "atelier paper high-contrast",
      [themeDensityAttribute]: "comfortable",
    },
  );
  assert.deepEqual(
    themeScopeAttributes({
      presets: ["headless", "signal", "signal"],
      density: "compact",
    }),
    {
      [themePresetAttribute]: "headless signal",
      [themeDensityAttribute]: "compact",
    },
  );

  const element = document.createElement("section");
  element.setAttribute(themePresetAttribute, "paper");
  const restore = applyThemeScope(element, { presets: ["midnight", "signal"], density: "compact" });
  assert.equal(element.getAttribute(themePresetAttribute), "midnight signal");
  assert.equal(element.getAttribute(themeDensityAttribute), "compact");
  restore();
  assert.equal(element.getAttribute(themePresetAttribute), "paper");
  assert.equal(element.hasAttribute(themeDensityAttribute), false);

  assert.throws(
    () => themeScopeAttributes({ presets: ["atelier", "bogus" as ThemePresetName] }),
    /VIZE_UI_THEME_SCOPE/,
  );
  assert.throws(
    () => themeScopeAttributes({ density: "dense" as ThemeDensityScale }),
    /VIZE_UI_THEME_SCOPE/,
  );
  assert.throws(
    () => applyThemeScope(null as unknown as Element, { presets: "atelier" }),
    /VIZE_UI_THEME_SCOPE/,
  );
});

test("creates a storage-backed no-flash theme bootstrap script", () => {
  const root = document.documentElement;
  root.removeAttribute(themePresetAttribute);
  root.removeAttribute(themeDensityAttribute);
  localStorage.clear();
  localStorage.setItem(themeScopeStorageKeys.presets, "paper signal signal");
  localStorage.setItem(themeScopeStorageKeys.density, "compact");

  const script = createThemeBootstrapScript({
    fallback: { presets: "atelier", density: "comfortable" },
  });
  runBootstrapScript(script);
  assert.equal(root.getAttribute(themePresetAttribute), "paper signal");
  assert.equal(root.getAttribute(themeDensityAttribute), "compact");

  root.removeAttribute(themePresetAttribute);
  root.removeAttribute(themeDensityAttribute);
  localStorage.setItem(themeScopeStorageKeys.presets, "atelier bogus");
  localStorage.setItem(themeScopeStorageKeys.density, "dense");
  runBootstrapScript(script);
  assert.equal(root.getAttribute(themePresetAttribute), "atelier");
  assert.equal(root.getAttribute(themeDensityAttribute), "comfortable");

  assert.throws(
    () => createThemeBootstrapScript({ storageKeys: { presets: " " } }),
    /VIZE_UI_THEME_SCOPE/,
  );

  root.removeAttribute(themePresetAttribute);
  root.removeAttribute(themeDensityAttribute);
  localStorage.clear();
});
