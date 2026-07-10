import assert from "node:assert/strict";

import type { CompiledModule } from "../types.ts";
import type { VizePluginState } from "./state.ts";
import { loadHook } from "./load.ts";

const emptyState: VizePluginState = {
  cache: new Map(),
  ssrCache: new Map(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map(),
  isProduction: true,
  root: "/project",
  clientViteBase: "/",
  serverViteBase: "/",
  server: null,
  filter: () => false,
  scanPatterns: null,
  precompileBatchSize: 128,
  ignorePatterns: [],
  mergedOptions: { exclude: [/node_modules/] },
  initialized: true,
  dynamicImportAliasRules: [],
  cssAliasRules: [],
  extractCss: true,
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

const dependencyStyleLoad = loadHook(
  emptyState,
  "/project/node_modules/pkg/ProsePre.vue?vue=&type=style&index=0&lang=css.css",
  { ssr: false },
);

assert.equal(
  dependencyStyleLoad,
  null,
  "uncached dependency SFC styles should fall through to Nuxt/Vite style handling",
);

const ssrStylePath = "/src/DeferredDemo.vue";
const ssrStyleLoad = loadHook(
  {
    ...emptyState,
    cache: new Map([
      [
        ssrStylePath,
        compiledStyle("clientstyle", "export default { props: { options: { type: Object } } }"),
      ],
    ]),
    ssrCache: new Map([
      [ssrStylePath, compiledStyle("ssrstyle", ".deferred-demo-loading { height: 350px; }")],
    ]),
  },
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

console.log("✅ vite-plugin-vize style load tests passed!");

function compiledStyle(scopeId: string, content: string): CompiledModule {
  return {
    code: `export default {}`,
    scopeId,
    hasScoped: false,
    styles: [{ content, lang: "css", scoped: false, module: false, index: 0 }],
  };
}
