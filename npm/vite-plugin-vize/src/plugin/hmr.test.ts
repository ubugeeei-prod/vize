import assert from "node:assert/strict";

import type { VizePluginState } from "./state.ts";
import {
  handleGenerateBundleHook,
  resolveComponentsCssFileName,
  VIZE_COMPONENTS_CSS_BASENAME,
  VIZE_COMPONENTS_CSS_FILE,
} from "./hmr.ts";

function createState(): VizePluginState {
  return {
    extractCss: true,
    componentsCssFileName: VIZE_COMPONENTS_CSS_FILE,
    collectedCss: new Map([
      ["/src/App.vue", ".app { color: red; }"],
      ["/src/Page.vue", ".page { color: blue; }"],
    ]),
    logger: {
      log() {},
    },
  } as VizePluginState;
}

assert.equal(
  resolveComponentsCssFileName(undefined),
  VIZE_COMPONENTS_CSS_FILE,
  "default Vite builds should keep the existing assets/ output path",
);
assert.equal(
  resolveComponentsCssFileName("_nuxt"),
  `_nuxt/${VIZE_COMPONENTS_CSS_BASENAME}`,
  "Nuxt client builds should emit component CSS under build.assetsDir",
);
assert.equal(
  resolveComponentsCssFileName("."),
  VIZE_COMPONENTS_CSS_BASENAME,
  "assetless builds should place component CSS beside chunks",
);

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
  const cssFileName = "_nuxt/vize-components.css";
  const bundle = {
    "_nuxt/index.js": {
      type: "chunk",
      isEntry: true,
      isDynamicEntry: false,
    },
  } satisfies Parameters<typeof handleGenerateBundleHook>[2];
  const state = {
    ...createState(),
    componentsCssFileName: cssFileName,
  };

  handleGenerateBundleHook(state, (file) => emitted.push(file), bundle);

  const entryChunk = bundle["_nuxt/index.js"] as {
    viteMetadata?: { importedCss?: Set<string> };
  };
  assert.equal(emitted[0]?.fileName, cssFileName);
  assert.deepEqual(entryChunk.viteMetadata?.importedCss, new Set([cssFileName]));
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

{
  const emitted: Array<{ type: "asset"; fileName: string; source: string }> = [];
  const bundle = {
    "assets/index.js": {
      type: "chunk",
      isEntry: true,
      isDynamicEntry: false,
    },
  } satisfies Parameters<typeof handleGenerateBundleHook>[2];
  const state = {
    ...createState(),
    extractCss: false,
  };

  handleGenerateBundleHook(state, (file) => emitted.push(file), bundle);

  assert.equal(emitted.length, 0, "SSR builds should not emit or attach extracted CSS");
  assert.equal(
    "viteMetadata" in bundle["assets/index.js"],
    false,
    "SSR chunks should not reference client-only extracted CSS",
  );
}

console.log("✅ vite-plugin-vize plugin hmr tests passed!");
