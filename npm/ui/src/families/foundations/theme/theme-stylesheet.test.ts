import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import { test } from "vite-plus/test";

import {
  themeCascadeLayerOrder,
  themeDensityScales,
  themePresets,
  themeTokens,
  themeTokensForPack,
} from "./theme.ts";

const stylesheet = await readFile(path.resolve("dist/style.css"), "utf8");
const sourcePresetStylesheets = new Map(
  await Promise.all(
    themePresets.map(async (name) => [
      name,
      await readFile(
        path.resolve(`src/families/foundations/theme/theme-preset-${name}.css`),
        "utf8",
      ),
    ]),
  ),
);
const opinionatedThemePresets = themePresets.filter((name) => name !== "headless");
const semanticColorTokens = themeTokensForPack("color");
const feedbackToneTokens = [
  "color-neutral",
  "color-neutral-contrast",
  "color-info",
  "color-info-contrast",
  "color-success",
  "color-success-contrast",
  "color-warning",
  "color-warning-contrast",
  "color-danger",
  "color-danger-contrast",
] as const;
const elevationTokens = themeTokensForPack("elevation");

const layerStarts = new Map(
  themeCascadeLayerOrder.map((layer) => [layer, stylesheet.indexOf(`@layer ${layer}{`)]),
);

/** Shipped rules of one vize layer block. */
function layerBlock(layer: (typeof themeCascadeLayerOrder)[number]): string {
  const start = layerStarts.get(layer);
  assert.ok(start !== undefined && start >= 0, `dist/style.css must ship @layer ${layer}`);
  const followers = [...layerStarts.values()].filter((index) => index > start);
  return stylesheet.slice(start, followers.length === 0 ? undefined : Math.min(...followers));
}

/** Value of one custom property in the packaged stylesheet's first definition. */
function shippedToken(name: string): string {
  const pattern = new RegExp(`--vize-ui-${name.replaceAll("-", "\\-")}:([^;}]+)[;}]`);
  const match = pattern.exec(stylesheet);
  assert.ok(match?.[1], `dist/style.css must define --vize-ui-${name}`);
  return match[1];
}

/** Normalized comparison form, tolerant of minifier whitespace and zero trims. */
function normalizeCss(value: string): string {
  return value.replaceAll(/\s+/g, "").replaceAll("0.", ".");
}

test("ships the documented cascade layer order", () => {
  const indexes = themeCascadeLayerOrder.map((layer) => {
    const index = layerStarts.get(layer) ?? -1;
    assert.ok(index >= 0, `dist/style.css must ship @layer ${layer}`);
    assert.equal(
      stylesheet.indexOf(`@layer ${layer}{`, index + 1),
      -1,
      `@layer ${layer} must merge into a single block`,
    );
    return index;
  });

  assert.deepEqual(
    indexes,
    [...indexes].sort((left, right) => left - right),
    "layer blocks must ship in ascending priority order",
  );
  assert.equal(indexes[0], 0, "the token layer must lead the stylesheet");
});

test("ships layered zero-specificity theme tokens matching the mirrors", () => {
  assert.match(layerBlock("vize.tokens"), /^@layer vize\.tokens\{:where\(:root,:host\)\{/);

  for (const [token, value] of Object.entries(themeTokens)) {
    assert.equal(normalizeCss(shippedToken(token)), normalizeCss(value), `token ${token}`);
  }
});

test("keeps the headless default free of visual opinion", () => {
  const tokens = layerBlock("vize.tokens");
  const headlessPreset = shippedPresetRule("headless");

  assert.match(tokens, /--vize-ui-color-canvas:Canvas[;}]/);
  assert.match(tokens, /--vize-ui-color-text:CanvasText[;}]/);
  assert.match(tokens, /--vize-ui-color-info:LinkText[;}]/);
  assert.match(tokens, /--vize-ui-color-success:CanvasText[;}]/);
  assert.match(tokens, /--vize-ui-color-warning:CanvasText[;}]/);
  assert.match(tokens, /--vize-ui-elevation-raised:none[;}]/);
  assert.doesNotMatch(tokens, /oklch\(/, "headless defaults must stay on the system palette");
  assert.match(headlessPreset, /color-scheme:light dark/);
  assert.doesNotMatch(
    headlessPreset,
    /--vize-ui-/,
    "the explicit headless preset must not add visual token opinion",
  );
});

test("ships semantic feedback tone tokens through every theme surface", () => {
  const defaults = layerBlock("vize.tokens");

  for (const token of feedbackToneTokens) {
    assert.ok(semanticColorTokens.includes(token), `${token} must belong to the color pack`);
    assert.match(defaults, new RegExp(`--vize-ui-${token}:`), `${token} must have a default`);
    assert.equal(normalizeCss(shippedToken(token)), normalizeCss(themeTokens[token]), token);
  }

  for (const name of opinionatedThemePresets) {
    const rule = shippedPresetRule(name);
    for (const token of feedbackToneTokens) {
      assert.match(rule, new RegExp(`--vize-ui-${token}:`), `${name} must ship ${token}`);
    }
  }
});

test("ships density scopes that retune the shared factor", () => {
  const tokens = layerBlock("vize.tokens");

  for (const [scale, factor] of Object.entries(themeDensityScales)) {
    const pattern = new RegExp(
      `:where\\(\\[data-vize-density\\]\\):where\\(\\[data-vize-density=${scale}\\]\\)` +
        `\\{--vize-ui-density:([^;}]+)\\}`,
    );
    const match = pattern.exec(tokens);
    assert.ok(match?.[1], `the ${scale} density scope must ship`);
    assert.equal(normalizeCss(match[1]), normalizeCss(factor));
  }

  // Space and control sizes resolve through the factor, never literal steps.
  assert.equal(shippedToken("space-md"), "calc(.75rem * var(--vize-ui-density))");
  assert.equal(shippedToken("size-control-md"), "calc(2.25rem * var(--vize-ui-density))");
});

function shippedPresetRule(name: (typeof themePresets)[number]): string {
  const preset = layerBlock("vize.preset");
  const pattern = new RegExp(
    `:where\\(\\[data-vize-theme~=${name}\\],:host\\(\\[data-vize-theme~=${name}\\]\\)\\)` +
      "\\{([^}]+)\\}",
  );
  const match = pattern.exec(preset);
  assert.ok(match?.[1], `the ${name} preset must ship`);
  return match[1];
}

function sourcePresetStylesheet(name: (typeof themePresets)[number]): string {
  const source = sourcePresetStylesheets.get(name);
  assert.ok(source, `src/families/foundations/theme/theme-preset-${name}.css must exist`);
  return source;
}

test("scopes published presets to their opt-in attributes", () => {
  const preset = layerBlock("vize.preset");

  for (const name of opinionatedThemePresets) {
    const rule = shippedPresetRule(name);
    assert.match(rule, /color-scheme:light dark/);
    assert.match(rule, /--vize-ui-color-accent:/);
    assert.match(rule, /--vize-ui-elevation-raised:/);
  }
  assert.match(shippedPresetRule("high-contrast"), /--vize-ui-border-width-thin:2px/);
  assert.match(shippedPresetRule("midnight"), /--vize-ui-z-overlay:1400/);
  assert.match(shippedPresetRule("paper"), /--vize-ui-type-leading-normal:1\.6/);
  assert.match(shippedPresetRule("play"), /--vize-ui-radius-lg:1\.25rem/);
  assert.match(shippedPresetRule("signal"), /--vize-ui-opacity-muted:\.82/);
  assert.doesNotMatch(
    preset,
    /:where\(:root,:host\)/,
    "presets must never assign tokens outside their opt-in scope",
  );
});

test("keeps every opinionated preset on the complete semantic color matrix", () => {
  for (const name of opinionatedThemePresets) {
    const source = sourcePresetStylesheet(name);

    // source-contract: preset layer ownership is authored before CSS floor lowering.
    assert.match(source, /@layer\s+vize\.preset/);
    // source-contract: preset scoping is authored before CSS floor lowering.
    assert.match(source, new RegExp(`data-vize-theme~="${name}"`));
    // source-contract: color-scheme pairing is authored before CSS floor lowering.
    assert.match(source, /color-scheme:\s*light dark;/);

    for (const token of semanticColorTokens) {
      assert.match(
        source,
        new RegExp(`--vize-ui-${token}:\\s*light-dark\\(`),
        `${name} must ship scheme-aware ${token}`,
      );
    }

    for (const token of elevationTokens) {
      // source-contract: elevation completeness is authored source policy.
      assert.match(source, new RegExp(`--vize-ui-${token}:`), `${name} must assign ${token}`);
    }
  }

  const headless = sourcePresetStylesheet("headless");
  assert.match(headless, /color-scheme:\s*light dark;/);
  assert.doesNotMatch(headless, /--vize-ui-/, "headless must remain a no-token preset");
});

test("lowers preset color schemes to the declared floor", () => {
  const preset = layerBlock("vize.preset");

  // light-dark() is newer than the floor and must ship as the lowered
  // custom-property pair driven by the color-scheme media flip.
  assert.doesNotMatch(stylesheet, /light-dark\(/);
  assert.match(
    preset,
    /--vize-ui-color-canvas:var\(--lightningcss-light,oklch\(98\.5% \.003 255\)\)var\(--lightningcss-dark,oklch\(15\.5% \.012 255\)\)/,
  );
  for (const name of opinionatedThemePresets) {
    assert.match(
      preset,
      new RegExp(
        `@media \\(prefers-color-scheme:(?:dark|light)\\)\\{:where\\(\\[data-vize-theme~=${name}\\]`,
      ),
    );
  }

  // Relative color and color-mix() derivations resolve to literals at build
  // time, so the floor never sees the newer syntax.
  assert.doesNotMatch(stylesheet, /oklch\(from/);
  assert.doesNotMatch(preset, /color-mix\(/);
  assert.match(
    preset,
    /--vize-ui-color-border:var\(--lightningcss-light,oklch\(88\.5% \.003 255\)\)/,
  );
  assert.match(preset, /--vize-ui-color-text-muted:var\(--lightningcss-light,oklab\(/);
});

test("keeps the forced-colors policy above the complete visual token surface", () => {
  const policy = layerBlock("vize.policy");
  const presetLayerIndex = layerStarts.get("vize.preset");
  const policyLayerIndex = layerStarts.get("vize.policy");

  assert.ok(
    presetLayerIndex !== undefined &&
      policyLayerIndex !== undefined &&
      presetLayerIndex < policyLayerIndex,
    "forced-colors policy must ship after every preset",
  );

  for (const token of [...semanticColorTokens, ...elevationTokens]) {
    assert.match(policy, new RegExp(`--vize-ui-${token}:`), `policy must cover ${token}`);
  }
  assert.match(policy, /--vize-ui-focus-ring-color:Highlight[;}]/);
});

test("stands down to system colors under forced colors", () => {
  const policy = layerBlock("vize.policy");

  assert.match(
    policy,
    /@media \(forced-colors:active\)\{:where\(:root,:host,\[data-vize-theme\]\)\{/,
  );
  assert.match(policy, /--vize-ui-color-accent:Highlight[;}]/);
  assert.match(policy, /--vize-ui-color-accent-contrast:HighlightText[;}]/);
  assert.match(policy, /--vize-ui-color-border:ButtonBorder[;}]/);
  assert.match(policy, /--vize-ui-color-info:Highlight[;}]/);
  assert.match(policy, /--vize-ui-color-info-contrast:HighlightText[;}]/);
  assert.match(policy, /--vize-ui-color-success:CanvasText[;}]/);
  assert.match(policy, /--vize-ui-color-warning:CanvasText[;}]/);
  assert.match(policy, /--vize-ui-elevation-floating:none[;}]/);
  assert.match(policy, /--vize-ui-color-danger-contrast:Canvas[;}]/);
  assert.match(policy, /--vize-ui-focus-ring-color:Highlight[;}]/);
  assert.match(
    layerBlock("vize.preset"),
    /@media \(forced-colors:active\)\{:where\(\[data-vize-theme~=high-contrast\]/,
  );
});
