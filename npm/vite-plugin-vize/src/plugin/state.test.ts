import assert from "node:assert/strict";

import {
  diffPrecompileFiles,
  hasFileMetadataChanged,
  type PrecompileFileMetadata,
} from "./state.js";

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

console.log("✅ vite-plugin-vize state tests passed!");
