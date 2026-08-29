import assert from "node:assert/strict";
import { test } from "vite-plus/test";
import { defineComponent, h, ref } from "vue";

import { mountInteraction } from "./testing/mount.ts";
import {
  setThemeTokens,
  themeDensityAttribute,
  themeDensityScales,
  themePresetAttribute,
  themePresets,
  themeTokenPackNames,
  themeTokenProperty,
  themeTokens,
  themeTokensForPack,
  themeTokenVar,
} from "./theme.ts";

test("applies and restores token overrides on a real element", () => {
  const element = document.createElement("section");
  document.body.append(element);
  element.style.setProperty("--vize-ui-color-accent", "rebeccapurple");

  const restore = setThemeTokens(element, {
    "color-accent": "oklch(0.6 0.2 300)",
    "radius-md": "0.75rem",
    density: "0.9",
  });
  assert.equal(element.style.getPropertyValue("--vize-ui-color-accent"), "oklch(0.6 0.2 300)");
  assert.equal(element.style.getPropertyValue("--vize-ui-radius-md"), "0.75rem");
  assert.equal(element.style.getPropertyValue("--vize-ui-density"), "0.9");

  restore();
  assert.equal(element.style.getPropertyValue("--vize-ui-color-accent"), "rebeccapurple");
  assert.equal(element.style.getPropertyValue("--vize-ui-radius-md"), "");
  assert.equal(element.style.getPropertyValue("--vize-ui-density"), "");
  element.remove();

  assert.equal(themeTokenProperty("color-canvas"), "--vize-ui-color-canvas");
  assert.equal(themeTokenProperty("density"), "--vize-ui-density");
  assert.equal(themeTokenVar("focus-ring-color"), "var(--vize-ui-focus-ring-color)");
});

test("rejects unknown tokens and empty override values", () => {
  const element = document.createElement("div");
  assert.throws(
    () => themeTokenProperty("color-bogus" as Parameters<typeof themeTokenProperty>[0]),
    /VIZE_UI_THEME_TOKEN/,
  );
  assert.throws(() => setThemeTokens(element, { "color-accent": "  " }), /VIZE_UI_THEME_TOKEN/);
  assert.throws(
    () => setThemeTokens(null as unknown as HTMLElement, { "color-accent": "red" }),
    /VIZE_UI_THEME_TOKEN/,
  );
});

test("publishes the token contract and scope constants", () => {
  assert.equal(themePresetAttribute, "data-vize-theme");
  assert.equal(themeDensityAttribute, "data-vize-density");
  assert.deepEqual(
    [...themePresets],
    ["headless", "atelier", "midnight", "paper", "play", "signal", "high-contrast"],
  );
  assert.deepEqual(Object.keys(themeDensityScales).sort(), ["comfortable", "compact"]);

  // The record is frozen and every name round-trips through the helpers.
  assert.ok(Object.isFrozen(themeTokens));
  for (const name of Object.keys(themeTokens)) {
    assert.equal(
      themeTokenProperty(name as keyof typeof themeTokens),
      `--vize-ui-${name}`,
      `token ${name} must map onto its custom property`,
    );
  }
});

test("scopes presets and densities in a mounted consumer", () => {
  const density = ref<"compact" | "comfortable">("compact");
  const Consumer = defineComponent({
    name: "ThemeScopeProbe",
    setup() {
      return () =>
        h(
          "section",
          {
            [themePresetAttribute]: "headless atelier midnight paper play signal high-contrast",
            [themeDensityAttribute]: density.value,
          },
          [h("output", { "data-accent": themeTokenVar("color-accent") }, "Themed")],
        );
    },
  });

  const handle = mountInteraction(Consumer);
  const root = handle.root();
  assert.equal(
    root.getAttribute("data-vize-theme"),
    "headless atelier midnight paper play signal high-contrast",
  );
  assert.equal(root.getAttribute("data-vize-density"), "compact");
  assert.equal(
    root.querySelector("output")?.getAttribute("data-accent"),
    "var(--vize-ui-color-accent)",
  );

  const restore = setThemeTokens(root, { "space-md": "1rem" });
  assert.equal(root.style.getPropertyValue("--vize-ui-space-md"), "1rem");
  restore();
  assert.equal(root.style.getPropertyValue("--vize-ui-space-md"), "");
  handle.unmount();
});

test("publishes independent token packs without gaps or overlap", () => {
  assert.ok(Object.isFrozen(themeTokenPackNames));

  const assigned = new Map<string, string>();
  for (const pack of themeTokenPackNames) {
    const tokens = themeTokensForPack(pack);
    assert.ok(Object.isFrozen(tokens), `pack ${pack} must be immutable`);
    assert.ok(tokens.length > 0, `pack ${pack} must publish at least one token`);
    for (const token of tokens) {
      assert.equal(assigned.get(token), undefined, `token ${token} must be in only one pack`);
      assigned.set(token, pack);
    }
  }

  assert.deepEqual([...assigned.keys()].sort(), Object.keys(themeTokens).sort());
  assert.deepEqual(themeTokensForPack("color"), [
    "color-canvas",
    "color-surface",
    "color-text",
    "color-text-muted",
    "color-accent",
    "color-accent-contrast",
    "color-border",
    "color-danger",
  ]);
  assert.deepEqual(themeTokensForPack("density"), ["density"]);
  assert.throws(
    () => themeTokensForPack("paint" as Parameters<typeof themeTokensForPack>[0]),
    /VIZE_UI_THEME_TOKEN/,
  );
});
