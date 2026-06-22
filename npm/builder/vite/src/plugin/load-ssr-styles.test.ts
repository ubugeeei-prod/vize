import assert from "node:assert/strict";

import type { VizePluginState } from "./state.ts";
import { loadHook } from "./load.ts";

// Verify that the SSR cache is preferred over the client cache when loading
// Vue style requests in an SSR context, and that Nuxt/Vite CSS suffix style
// request IDs (e.g. `.css?inline&used.css.css?inline`) are handled correctly.

const ssrStylePath = "/src/DeferredDemo.vue";
const ssrStyleState: VizePluginState = {
  cache: new Map([
    [
      ssrStylePath,
      {
        code: `export default { __name: "DeferredDemoClient" }`,
        scopeId: "clientstyle",
        hasScoped: false,
        styles: [
          {
            content: "export default { props: { options: { type: Object } } }",
            lang: "css",
            scoped: false,
            module: false,
            index: 0,
          },
        ],
      },
    ],
  ]),
  ssrCache: new Map([
    [
      ssrStylePath,
      {
        code: `export default { __name: "DeferredDemoSsr" }`,
        scopeId: "ssrstyle",
        hasScoped: false,
        styles: [
          {
            content: ".deferred-demo-loading { height: 350px; }",
            lang: "css",
            scoped: false,
            module: false,
            index: 0,
          },
        ],
      },
    ],
  ]),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map(),
  isProduction: false,
  root: "/src",
  clientViteBase: "/",
  serverViteBase: "/",
  server: {} as never,
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
};

const ssrStyleLoad = loadHook(
  ssrStyleState,
  "/src/DeferredDemo.vue?vue=&type=style&index=0&lang=css.css?inline&used.css.css?inline",
  { ssr: true },
);
assert.ok(
  ssrStyleLoad && typeof ssrStyleLoad === "object",
  "SSR style requests with CSS suffixes should load as code objects",
);
assert.equal(
  ssrStyleLoad.code,
  ".deferred-demo-loading { height: 350px; }",
  "SSR style requests should read style blocks from the SSR cache before the client cache",
);

console.log("✅ vite-plugin-vize load SSR style cache tests passed!");
