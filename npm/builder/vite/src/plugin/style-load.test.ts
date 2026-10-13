import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { compileSfc, transformViteCssVarsForPipeline } from "@vizejs/native";

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

const official = createRequire(import.meta.url)("vue/compiler-sfc");
const originalHead = fs.readFileSync(
  new URL(
    "../../../../../crates/vize_atelier_sfc/tests/fixtures/fix-history/production-extracted-css-vars.input.txt",
    import.meta.url,
  ),
  "utf8",
);
const headPath = "/project/app/pages/timetable/_components/TimetableHead.vue";
const scopeId = "86318631";
const variables = (css: string) => [...css.matchAll(/var\(--([\w-]+)\)/g)].map((match) => match[1]);

for (const isProduction of [false, true]) {
  const result = compileSfc(originalHead, {
    filename: headPath,
    scopeId,
    isProd: isProduction,
    inlineTemplate: true,
  });
  assert.deepEqual(result.errors, []);
  const stock = official.compileStyle({
    source: result.styles[0].content,
    filename: headPath,
    id: isProduction ? headPath : scopeId,
    isProd: isProduction,
  });
  assert.deepEqual(stock.errors, []);
  const expectedNames = variables(stock.code);
  assert.deepEqual(variables(result.css ?? ""), expectedNames);
  const cached = {
    ...result,
    scopeId,
    styles: result.styles.map((block) => ({ ...block, lang: block.lang ?? null })),
  };
  // Raw and Vite filesystem requests share the resolved filename.
  for (const path of [headPath, `/@fs${headPath}`]) {
    const loaded = loadHook(
      { ...emptyState, isProduction, cache: new Map([[headPath, cached]]) },
      `${path}?vue&type=style&index=0&lang=css`,
      { ssr: false },
    );
    assert.ok(loaded && typeof loaded === "object", path);
    assert.deepEqual(variables(loaded.code), expectedNames);
  }
  const defaultStyle = transformViteCssVarsForPipeline(result.styles[0].content, scopeId);
  assert.deepEqual(variables(defaultStyle), ["86318631-textColor", "86318631-backgroundColor"]);
}

function compiledStyle(scopeId: string, content: string): CompiledModule {
  return {
    code: `export default {}`,
    scopeId,
    hasScoped: false,
    styles: [{ content, lang: "css", scoped: false, module: false, index: 0 }],
  };
}
