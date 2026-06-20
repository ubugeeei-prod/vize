import assert from "node:assert/strict";

import type { CompiledModule } from "./types.ts";
import { detectHmrUpdateType, hasHmrChanges } from "./hmr.ts";

const baseModule: CompiledModule = {
  code: "export default {}",
  scopeId: "scope1234",
  hasScoped: false,
  templateHash: "template-a",
  styleHash: "style-a",
  scriptHash: "script-a",
  macroArtifacts: [],
  styles: [],
};

const templateOnlyModule: CompiledModule = {
  ...baseModule,
  templateHash: "template-b",
};
assert.equal(hasHmrChanges(baseModule, templateOnlyModule), true);
assert.equal(detectHmrUpdateType(baseModule, templateOnlyModule), "template-only");

const styleOnlyModule: CompiledModule = {
  ...baseModule,
  styleHash: "style-b",
};
assert.equal(hasHmrChanges(baseModule, styleOnlyModule), true);
assert.equal(detectHmrUpdateType(baseModule, styleOnlyModule), "style-only");

const fullReloadModule: CompiledModule = {
  ...baseModule,
  scriptHash: "script-b",
};
assert.equal(hasHmrChanges(baseModule, fullReloadModule), true);
assert.equal(detectHmrUpdateType(baseModule, fullReloadModule), "full-reload");

const unchangedModule: CompiledModule = {
  ...baseModule,
};
assert.equal(hasHmrChanges(baseModule, unchangedModule), false);
assert.equal(
  detectHmrUpdateType(baseModule, unchangedModule),
  "full-reload",
  "Callers must short-circuit no-op updates before generating HMR output",
);

const generatedCodeChangedModule: CompiledModule = {
  ...baseModule,
  code: "export default { name: 'Changed' }",
};
assert.equal(
  hasHmrChanges(baseModule, generatedCodeChangedModule),
  true,
  "HMR must react when generated code changes even if native section hashes stay stable",
);
assert.equal(
  detectHmrUpdateType(baseModule, generatedCodeChangedModule),
  "full-reload",
  "Generated code changes outside section hashes should conservatively reload the component",
);

const cssChangedWithoutStyleHashModule: CompiledModule = {
  ...baseModule,
  css: ".root { color: blue; }",
};
assert.equal(
  hasHmrChanges(baseModule, cssChangedWithoutStyleHashModule),
  true,
  "HMR must react when compiled CSS changes even if the style hash is missing or unchanged",
);
assert.equal(
  detectHmrUpdateType(baseModule, cssChangedWithoutStyleHashModule),
  "style-only",
  "CSS-only fallback changes should keep the style-only HMR path",
);

const styleMetadataChangedModule: CompiledModule = {
  ...baseModule,
  styles: [
    {
      content: ".root { color: red; }",
      src: null,
      lang: null,
      scoped: false,
      module: true,
      index: 0,
    },
  ],
};
assert.equal(
  hasHmrChanges(baseModule, styleMetadataChangedModule),
  true,
  "HMR must react when style metadata changes the generated module shape",
);
assert.equal(
  detectHmrUpdateType(baseModule, styleMetadataChangedModule),
  "full-reload",
  "Style pipeline metadata changes can affect imports or CSS modules and should reload",
);

console.log("✅ vite-plugin-vize hmr tests passed!");
