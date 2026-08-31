import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  PRECOMPILE_CACHE_RELATIVE_DIR,
  assertPrecompileCacheCold,
  clearPrecompileCache,
  precompileCacheDir,
  readPrecompileCacheEntries,
} from "../../tools/benchmarks/scripts/vite-precompile-cache.mjs";

function withTempRoot(fn: (root: string) => void): void {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-vite-cache-"));
  try {
    fn(root);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
}

function writeManifests(root: string, names: string[]): void {
  const dir = precompileCacheDir(root);
  fs.mkdirSync(dir, { recursive: true });
  for (const name of names) {
    fs.writeFileSync(path.join(dir, name), "manifest");
  }
}

test("the pre-compile manifest directory matches the plugin's own layout", () => {
  assert.equal(
    PRECOMPILE_CACHE_RELATIVE_DIR,
    path.join("node_modules", ".vize", "vite-precompile"),
  );
  assert.equal(
    precompileCacheDir(path.join(path.sep, "work")),
    path.join(path.sep, "work", "node_modules", ".vize", "vite-precompile"),
  );
});

test("a root that has never been built reports no manifests", () => {
  withTempRoot((root) => {
    assert.deepEqual(readPrecompileCacheEntries(root), []);
    assert.deepEqual(clearPrecompileCache(root), []);
    assert.equal(assertPrecompileCacheCold(root, "@vizejs/vite-plugin"), undefined);
  });
});

test("clearing removes every manifest and leaves the rest of node_modules alone", () => {
  withTempRoot((root) => {
    writeManifests(root, ["b1c2.vpc", "a0f9.vpc"]);
    const sibling = path.join(root, "node_modules", ".vize", "oxc-dumps");
    fs.mkdirSync(sibling, { recursive: true });
    fs.writeFileSync(path.join(sibling, "keep.ts"), "keep");

    assert.deepEqual(readPrecompileCacheEntries(root), ["a0f9.vpc", "b1c2.vpc"]);
    assert.deepEqual(clearPrecompileCache(root), ["a0f9.vpc", "b1c2.vpc"]);
    assert.deepEqual(readPrecompileCacheEntries(root), []);
    assert.equal(fs.existsSync(precompileCacheDir(root)), false);
    assert.deepEqual(fs.readdirSync(sibling), ["keep.ts"]);
  });
});

// Regression guard for the defect this module exists to prevent: `tools/benchmarks/scripts/vite.ts`
// ran a warmup build and then the measured build in the same working directory,
// so after the persistent pre-compile cache landed (#3372) the measured Vize
// build restored all of its modules from the warmup's manifest while
// `@vitejs/plugin-vue` compiled from scratch. Measured output on the pre-fix
// harness, 300 SFCs: "0 recompiled, 300 restored from disk".
test("measuring against a warm manifest is refused with the manifests named", () => {
  withTempRoot((root) => {
    writeManifests(root, ["b1c2.vpc", "a0f9.vpc"]);

    assert.throws(
      () => assertPrecompileCacheCold(root, "@vizejs/vite-plugin"),
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.match(error.message, /Refusing to measure/);
        assert.ok(error.message.includes("@vizejs/vite-plugin"));
        assert.ok(error.message.includes(precompileCacheDir(root)));
        assert.ok(error.message.includes("a0f9.vpc, b1c2.vpc"));
        return true;
      },
    );

    clearPrecompileCache(root);
    assert.equal(assertPrecompileCacheCold(root, "@vizejs/vite-plugin"), undefined);
  });
});
