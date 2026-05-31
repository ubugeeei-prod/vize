import assert from "node:assert/strict";

import type { VizePluginState } from "./state.ts";
import { handleGenerateBundleHook, VIZE_COMPONENTS_CSS_FILE } from "./hmr.ts";

function createState(): VizePluginState {
  return {
    extractCss: true,
    collectedCss: new Map([
      ["/src/App.vue", ".app { color: red; }"],
      ["/src/Page.vue", ".page { color: blue; }"],
    ]),
    logger: {
      log() {},
    },
  } as VizePluginState;
}

{
  const emitted: Array<{ type: "asset"; fileName: string; source: string }> = [];
  const existingCss = new Set(["assets/app.css"]);
  const dynamicCss = new Set<string>();
  const bundle = {
    "assets/index.js": {
      type: "chunk",
      isEntry: true,
      isDynamicEntry: false,
      viteMetadata: {
        importedCss: existingCss,
      },
    },
    "assets/lazy.js": {
      type: "chunk",
      isEntry: false,
      isDynamicEntry: true,
      viteMetadata: {
        importedCss: dynamicCss,
      },
    },
    "assets/shared.js": {
      type: "chunk",
      isEntry: false,
      isDynamicEntry: false,
    },
    "assets/logo.svg": {
      type: "asset",
    },
  } satisfies Parameters<typeof handleGenerateBundleHook>[2];

  handleGenerateBundleHook(createState(), (file) => emitted.push(file), bundle);

  assert.deepEqual(emitted, [
    {
      type: "asset",
      fileName: VIZE_COMPONENTS_CSS_FILE,
      source: ".app { color: red; }\n\n.page { color: blue; }",
    },
  ]);
  assert.equal(existingCss.has("assets/app.css"), true);
  assert.equal(existingCss.has(VIZE_COMPONENTS_CSS_FILE), true);
  assert.equal(dynamicCss.has(VIZE_COMPONENTS_CSS_FILE), true);
  assert.equal("viteMetadata" in bundle["assets/shared.js"], false);
}

{
  const emitted: Array<{ type: "asset"; fileName: string; source: string }> = [];
  const bundle = {
    "assets/index.js": {
      type: "chunk",
      isEntry: true,
      isDynamicEntry: false,
    },
  } satisfies Parameters<typeof handleGenerateBundleHook>[2];

  handleGenerateBundleHook(createState(), (file) => emitted.push(file), bundle);

  const entryChunk = bundle["assets/index.js"] as {
    viteMetadata?: { importedCss?: Set<string> };
  };
  assert.equal(emitted.length, 1);
  assert.deepEqual(entryChunk.viteMetadata?.importedCss, new Set([VIZE_COMPONENTS_CSS_FILE]));
}

console.log("✅ vite-plugin-vize plugin hmr tests passed!");
