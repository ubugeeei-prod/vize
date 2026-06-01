import assert from "node:assert/strict";
import path from "node:path";

import { createPostTransformPlugin, transformScopedPreprocessorCss } from "./compat.ts";
import type { VizePluginState } from "./state.ts";

function createState(overrides: Partial<VizePluginState> = {}): VizePluginState {
  return {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    root: "/src",
    clientViteBase: "/",
    serverViteBase: "/",
    server: null,
    filter: () => true,
    scanPatterns: ["**/*.vue"],
    precompileBatchSize: 128,
    ignorePatterns: [],
    mergedOptions: {},
    initialized: true,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {},
    serverViteDefine: {},
    logger: {
      log() {},
      info() {},
      warn() {},
      error() {},
    } as never,
    ...overrides,
  };
}

const virtualSfcId = path.join("/virtual", "Card.setup.ts");
const virtualSfcSource = `
<script setup lang="ts">
const msg = 'hello'
</script>

<template>
  <div class="card">{{ msg }}</div>
</template>

<style>
.card { color: rebeccapurple; }
</style>
`;

{
  const state = createState();
  const plugin = createPostTransformPlugin(state);
  const result = await plugin.transform?.(virtualSfcSource, virtualSfcId, {
    ssr: true,
  });

  assert.ok(result && typeof result === "object", "SSR virtual SFC transforms should succeed");
  assert.doesNotMatch(
    result.code,
    /__vize_css__/,
    "SSR post-transforms should not inject client CSS runtime exports",
  );
  assert.doesNotMatch(
    result.code,
    /document\.createElement/,
    "SSR post-transforms should stay free of document-based side effects",
  );
  assert.equal(
    result.map,
    null,
    "SSR post-transforms should not allocate discarded OXC sourcemaps",
  );
}

{
  const transformed = transformScopedPreprocessorCss(
    ".rrevdjwu > .group + .group { color: red; }",
    "\0/src/MkSuperMenu.vue?vue=&type=style&index=0&scoped=data-v-menu&lang=scss.scss",
  );

  assert.equal(
    transformed,
    ".rrevdjwu > .group + .group[data-v-menu] { color: red; }",
    "Scoped preprocessor CSS should be scoped after preprocessing, matching Vue selector placement",
  );
}

{
  const transformed = transformScopedPreprocessorCss(
    ".rrevdjwu > .group + .group { color: red; }",
    "/src/MkSuperMenu.vue?vue=&type=style&index=0&scoped=data-v-menu&lang=scss.scss",
  );

  assert.equal(
    transformed,
    ".rrevdjwu > .group + .group[data-v-menu] { color: red; }",
    "CSS-visible style IDs should still receive scoped preprocessor post-processing",
  );
}

{
  const state = createState({
    isProduction: true,
    extractCss: true,
  });
  const plugin = createPostTransformPlugin(state);
  const result = await plugin.transform?.(virtualSfcSource, virtualSfcId, {
    ssr: false,
  });

  assert.ok(
    result && typeof result === "object",
    "Production virtual SFC transforms should succeed",
  );
  assert.equal(
    state.collectedCss.has(virtualSfcId),
    false,
    "Production virtual SFC transforms should let Vite collect emitted style imports",
  );
  assert.match(
    result.code,
    /import ".*Card\.setup\.ts\?vue=&type=style&index=0&lang=css";/,
    "Production virtual SFC transforms should emit a virtual style import",
  );
}

console.log("✅ vite-plugin-vize compat tests passed!");
