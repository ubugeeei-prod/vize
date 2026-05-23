import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import {
  DEFAULT_PRECOMPILE_BATCH_MAX_BYTES,
  DEFAULT_PRECOMPILE_BATCH_SIZE,
  DEFAULT_PRECOMPILE_IGNORE_PATTERNS,
  clearBuildCaches,
  chunkPrecompileFiles,
  compileAll,
  diffPrecompileFiles,
  getCompileOptionsForRequest,
  hasFileMetadataChanged,
  normalizePrecompileBatchSize,
  syncCollectedCssForFile,
  type PrecompileFileMetadata,
  type VizePluginState,
} from "./state.ts";
import type { CompiledModule } from "../types.ts";

const previousMetadata = new Map<string, PrecompileFileMetadata>([
  ["/src/unchanged.vue", { mtimeMs: 10, size: 100 }],
  ["/src/changed.vue", { mtimeMs: 10, size: 100 }],
  ["/src/removed.vue", { mtimeMs: 10, size: 100 }],
]);

const currentMetadata = new Map<string, PrecompileFileMetadata>([
  ["/src/unchanged.vue", { mtimeMs: 10, size: 100 }],
  ["/src/changed.vue", { mtimeMs: 20, size: 100 }],
  ["/src/new.vue", { mtimeMs: 30, size: 50 }],
]);

assert.equal(
  hasFileMetadataChanged(
    previousMetadata.get("/src/unchanged.vue"),
    currentMetadata.get("/src/unchanged.vue")!,
  ),
  false,
);
assert.equal(
  hasFileMetadataChanged(
    previousMetadata.get("/src/changed.vue"),
    currentMetadata.get("/src/changed.vue")!,
  ),
  true,
);
assert.equal(hasFileMetadataChanged(undefined, currentMetadata.get("/src/new.vue")!), true);

const diff = diffPrecompileFiles(
  ["/src/unchanged.vue", "/src/changed.vue", "/src/new.vue"],
  currentMetadata,
  previousMetadata,
);
assert.deepEqual(diff.changedFiles, ["/src/changed.vue", "/src/new.vue"]);
assert.deepEqual(diff.deletedFiles, ["/src/removed.vue"]);

assert.equal(
  normalizePrecompileBatchSize(undefined),
  DEFAULT_PRECOMPILE_BATCH_SIZE,
  "Missing precompile batch size should use the memory-safe default",
);
assert.equal(
  normalizePrecompileBatchSize(3.8),
  3,
  "Precompile batch size should be normalized to a whole-file count",
);
assert.equal(
  normalizePrecompileBatchSize(0.5),
  1,
  "Fractional precompile batch sizes should still compile at least one file per batch",
);
assert.deepEqual(
  chunkPrecompileFiles(["a.vue", "b.vue", "c.vue", "d.vue", "e.vue"], 2),
  [["a.vue", "b.vue"], ["c.vue", "d.vue"], ["e.vue"]],
  "Precompile chunking should cap the number of SFCs handed to native compilation",
);
assert.deepEqual(
  chunkPrecompileFiles(["a.vue", "b.vue", "c.vue"], 10, {
    maxBytes: 10,
    metadata: new Map<string, PrecompileFileMetadata>([
      ["a.vue", { mtimeMs: 1, size: 4 }],
      ["b.vue", { mtimeMs: 1, size: 4 }],
      ["c.vue", { mtimeMs: 1, size: 9 }],
    ]),
  }),
  [["a.vue", "b.vue"], ["c.vue"]],
  "Precompile chunking should cap the total source bytes retained per native batch",
);
assert.equal(
  DEFAULT_PRECOMPILE_BATCH_MAX_BYTES,
  32 * 1024 * 1024,
  "Default precompile byte cap should leave headroom in Node heap",
);

const brokenPrecompileRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-precompile-fail-"));
fs.writeFileSync(path.join(brokenPrecompileRoot, "Broken.vue"), `<template><div></template>`);

const brokenPrecompileState: VizePluginState = {
  cache: new Map(),
  ssrCache: new Map(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map(),
  isProduction: false,
  root: brokenPrecompileRoot,
  clientViteBase: "/",
  serverViteBase: "/",
  server: null,
  filter: () => true,
  scanPatterns: ["**/*.vue"],
  precompileBatchSize: DEFAULT_PRECOMPILE_BATCH_SIZE,
  ignorePatterns: [],
  mergedOptions: {},
  initialized: true,
  dynamicImportAliasRules: [],
  cssAliasRules: [],
  extractCss: false,
  clientViteDefine: {},
  serverViteDefine: {},
  logger: {
    log() {},
    info() {},
    warn() {},
    error() {},
  } as never,
};

await assert.rejects(
  () => compileAll(brokenPrecompileState),
  /Pre-compilation failed for 1 file\(s\)[\s\S]*Broken\.vue/,
  "Pre-compilation errors should fail buildStart instead of continuing with a partial cache",
);
assert.equal(
  brokenPrecompileState.cache.has(path.join(brokenPrecompileRoot, "Broken.vue")),
  false,
  "Failed pre-compilation must not leave invalid output in the cache",
);

assert.ok(
  DEFAULT_PRECOMPILE_IGNORE_PATTERNS.includes(".nuxt/**"),
  "Nuxt build artifacts should be ignored by default during pre-compilation",
);

assert.deepEqual(
  getCompileOptionsForRequest(
    {
      isProduction: false,
      mergedOptions: { vapor: true },
    },
    false,
  ),
  {
    sourceMap: true,
    ssr: false,
    vapor: true,
    customRenderer: false,
    vueParserQuirks: false,
  },
  "Client requests should keep Vapor enabled when the plugin is configured for it",
);

assert.deepEqual(
  getCompileOptionsForRequest(
    {
      isProduction: true,
      mergedOptions: { vapor: true },
    },
    true,
  ),
  {
    sourceMap: false,
    ssr: true,
    vapor: false,
    customRenderer: false,
    vueParserQuirks: false,
  },
  "SSR requests should continue to use the VDOM compiler while client builds hydrate with Vapor",
);

const cssState = {
  isProduction: true,
  collectedCss: new Map<string, string>(),
  cssAliasRules: [],
};

const plainCssModule: CompiledModule = {
  code: "export default {}",
  css: ".card { color: tomato; }",
  scopeId: "plaincss",
  hasScoped: false,
  macroArtifacts: [],
  styles: [
    {
      content: ".card { color: tomato; }",
      lang: "css",
      scoped: false,
      module: false,
      index: 0,
    },
  ],
};

syncCollectedCssForFile(cssState, "/src/Card.vue", plainCssModule);
assert.equal(
  cssState.collectedCss.get("/src/Card.vue"),
  ".card { color: tomato; }",
  "Production CSS collection should retain plain CSS modules",
);

const delegatedCssModule: CompiledModule = {
  code: "export default {}",
  css: ".button { color: red; }",
  scopeId: "delegatedcss",
  hasScoped: false,
  macroArtifacts: [],
  styles: [
    {
      content: ".button { color: red; }",
      lang: "css",
      scoped: false,
      module: "buttonStyles",
      index: 0,
    },
  ],
};

syncCollectedCssForFile(cssState, "/src/Button.vue", delegatedCssModule);
assert.equal(
  cssState.collectedCss.has("/src/Button.vue"),
  false,
  "Delegated CSS modules should stay out of the extracted plain CSS bundle",
);

const retainedBuildState = {
  cache: new Map([["/src/App.vue", plainCssModule]]),
  ssrCache: new Map([["/src/App.vue", plainCssModule]]),
  collectedCss: new Map([["/src/App.vue", ".app{}"]]),
  precompileMetadata: new Map([["/src/App.vue", { mtimeMs: 1, size: 100 }]]),
  pendingHmrUpdateTypes: new Map([["/src/App.vue", "template-only" as const]]),
};

clearBuildCaches(retainedBuildState);
assert.equal(retainedBuildState.cache.size, 0, "build cache should be released after bundling");
assert.equal(
  retainedBuildState.ssrCache.size,
  0,
  "SSR build cache should be released after bundling",
);
assert.equal(
  retainedBuildState.collectedCss.size,
  0,
  "collected CSS cache should be released after CSS emission",
);
assert.equal(
  retainedBuildState.precompileMetadata.size,
  0,
  "precompile metadata should be released after build",
);
assert.equal(
  retainedBuildState.pendingHmrUpdateTypes.size,
  0,
  "build-only HMR bookkeeping should be released after build",
);

console.log("✅ vite-plugin-vize state tests passed!");
