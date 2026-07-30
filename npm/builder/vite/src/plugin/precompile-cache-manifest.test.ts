/**
 * Unit-level tests for the pre-compile cache manifest.
 *
 * The staleness tests that drive `compileAll` end to end live in
 * `./precompile-cache.test.ts`; this file exercises the cache object directly.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  PRECOMPILE_CACHE_DIR,
  PRECOMPILE_CACHE_ENV,
  createDisabledPrecompileCache,
  isPersistablePrecompileModule,
  isPrecompileCacheDisabledByEnv,
  openPrecompileCache,
} from "./precompile-cache.ts";
import { computePrecompileCacheKey } from "./precompile-cache-key.ts";
import type { CompiledModule } from "../types.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const testRoot = path.join(
  path.resolve(__dirname, "../../../.."),
  "target",
  "vize-tests",
  "tests",
  "vite-plugin-vize",
  "precompile-cache-manifest",
);
fs.rmSync(testRoot, { recursive: true, force: true });
fs.mkdirSync(testRoot, { recursive: true });

assert.equal(isPrecompileCacheDisabledByEnv({ [PRECOMPILE_CACHE_ENV]: "0" }), true);
assert.equal(isPrecompileCacheDisabledByEnv({ [PRECOMPILE_CACHE_ENV]: "false" }), true);
assert.equal(isPrecompileCacheDisabledByEnv({}), false);
assert.equal(isPrecompileCacheDisabledByEnv({ [PRECOMPILE_CACHE_ENV]: "1" }), false);

const module: CompiledModule = {
  code: "export default {}",
  scopeId: "data-v-1",
  hasScoped: false,
  styles: [],
  macroArtifacts: [],
  dependencies: [],
};

assert.equal(isPersistablePrecompileModule(module), true);
assert.equal(isPersistablePrecompileModule({ ...module, dependencies: ["/x.js"] }), false);

const disabled = createDisabledPrecompileCache();
disabled.set("/a.vue", "hash", module);
assert.equal(disabled.file, null);
assert.equal(disabled.get("/a.vue", "hash"), undefined);
assert.equal(disabled.flush(), false, "a disabled cache never reports a write");

const root = path.join(testRoot, "unit");
fs.mkdirSync(root, { recursive: true });
const options = { ssr: false, vapor: false };
const cache = openPrecompileCache({ root, compileOptions: options });
assert.equal(cache.flush(), false, "an untouched cache must not write");
cache.set("/a.vue", "hash-a", module);
assert.equal(cache.flush(), true);
assert.equal(cache.flush(), false, "a clean cache must not rewrite");

const reopened = openPrecompileCache({ root, compileOptions: options });
assert.deepEqual(reopened.get("/a.vue", "hash-a"), module);
assert.equal(reopened.get("/a.vue", "hash-b"), undefined, "a different hash must miss");
assert.equal(reopened.get("/b.vue", "hash-a"), undefined, "an unknown path must miss");

// A module with `src` dependencies is refused, and evicts any prior entry.
reopened.set("/a.vue", "hash-a", { ...module, dependencies: ["/dep.js"] });
assert.equal(reopened.get("/a.vue", "hash-a"), undefined);

// Different compile options must land in a different manifest.
assert.notEqual(
  computePrecompileCacheKey({ ssr: false, vapor: true }),
  computePrecompileCacheKey(options),
);
const other = openPrecompileCache({ root, compileOptions: { ssr: false, vapor: true } });
assert.notEqual(other.file, cache.file);
assert.equal(other.get("/a.vue", "hash-a"), undefined);

// Key order inside the options object must not change the key.
assert.equal(
  computePrecompileCacheKey({ ssr: false, vapor: true }),
  computePrecompileCacheKey({ vapor: true, ssr: false }),
);

// Atomic writes must not leave temp files behind.
assert.deepEqual(
  fs.readdirSync(path.join(root, PRECOMPILE_CACHE_DIR)).filter((name) => name.endsWith(".tmp")),
  [],
);

// An unwritable cache directory must not fail the build.
const readonlyRoot = path.join(testRoot, "readonly");
fs.mkdirSync(path.join(readonlyRoot, "node_modules"), { recursive: true });
fs.writeFileSync(path.join(readonlyRoot, "node_modules", ".vize"), "not a directory\n");
const diagnostics: string[] = [];
const blocked = openPrecompileCache({
  root: readonlyRoot,
  compileOptions: options,
  onDiagnostic: (message) => diagnostics.push(message),
});
blocked.set("/a.vue", "hash-a", module);
assert.equal(blocked.flush(), false, "an unwritable manifest must report failure, not throw");
assert.equal(diagnostics.length, 1, "the write failure must be surfaced as a diagnostic");

console.log("✅ vite-plugin-vize precompile cache manifest tests passed!");
