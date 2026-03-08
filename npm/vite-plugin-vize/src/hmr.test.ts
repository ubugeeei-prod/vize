import assert from "node:assert/strict";

import type { CompiledModule } from "./types.js";
import { detectHmrUpdateType, hasHmrChanges } from "./hmr.js";

const baseModule: CompiledModule = {
  code: "export default {}",
  scopeId: "scope1234",
  hasScoped: false,
  templateHash: "template-a",
  styleHash: "style-a",
  scriptHash: "script-a",
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

console.log("✅ vite-plugin-vize hmr tests passed!");
