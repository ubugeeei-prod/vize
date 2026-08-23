import assert from "node:assert/strict";
import { test } from "node:test";

import { isolationTsconfigPaths } from "../../tools/fixtures/typecheck-dependency-prepare.mjs";

/**
 * Vize's `--tsconfig` and vue-tsc's baseline config can differ (Nuxt Volt).
 * Isolation used to walk only the baseline, so packages declared only on
 * Vize's config still escaped into Vize's `node_modules` (#4461).
 */

test("isolation walks the baseline tsconfig and Vize's tsconfig when they differ", () => {
  assert.deepEqual(
    isolationTsconfigPaths({
      tsconfig: "apps/volt/tsconfig.json",
      typecheckPerformance: { baseline: { tsconfig: "apps/volt/.nuxt/tsconfig.json" } },
    }),
    ["apps/volt/.nuxt/tsconfig.json", "apps/volt/tsconfig.json"],
  );
});

test("isolation does not walk the same tsconfig twice", () => {
  assert.deepEqual(isolationTsconfigPaths({ tsconfig: "tsconfig.json" }), ["tsconfig.json"]);
});

test("isolation follows the baseline tsconfig when Vize's is omitted", () => {
  assert.deepEqual(
    isolationTsconfigPaths({
      typecheckPerformance: { baseline: { tsconfig: ".nuxt/tsconfig.app.json" } },
    }),
    [".nuxt/tsconfig.app.json"],
  );
});
