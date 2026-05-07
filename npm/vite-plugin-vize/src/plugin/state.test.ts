import assert from "node:assert/strict";

import {
  diffPrecompileFiles,
  getCompileOptionsForRequest,
  hasFileMetadataChanged,
  syncCollectedCssForFile,
  type PrecompileFileMetadata,
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

console.log("✅ vite-plugin-vize state tests passed!");
