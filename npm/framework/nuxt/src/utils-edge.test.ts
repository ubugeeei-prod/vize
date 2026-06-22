import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";

import {
  NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
  buildNuxtCompilerOptions,
  buildNuxtDevAssetBase,
  isVizeGeneratedVueModuleId,
  isVizeJsxModuleId,
  isVizeVirtualVueModuleId,
  normalizeNuxtInjectedKeysForVizeVirtualModule,
  normalizeVizeGeneratedVueModuleId,
  normalizeVizeVirtualVueModuleId,
  preserveExplicitVueImportsFromNuxtAutoImports,
  stabilizeNuxtInjectedKeysForVizeVirtualModule,
} from "./utils.ts";

function stableNuxtKey(normalizedId: string, index: number): string {
  return createHash("sha256")
    .update(normalizedId)
    .update(":")
    .update(String(index))
    .digest("base64url")
    .slice(0, 10);
}

void test("buildNuxtDevAssetBase keeps the default /_nuxt/ asset base", () => {
  assert.equal(buildNuxtDevAssetBase("/", "/_nuxt/"), "/_nuxt/");
});

void test("buildNuxtDevAssetBase treats an empty baseURL like the root base", () => {
  // "" normalizes to "/", so the asset dir is returned unprefixed.
  assert.equal(buildNuxtDevAssetBase("", "/_nuxt/"), "/_nuxt/");
});

void test("buildNuxtDevAssetBase collapses an empty buildAssetsDir under the root base", () => {
  // "" buildAssetsDir normalizes to "/", and root base "/" short-circuits to that.
  assert.equal(buildNuxtDevAssetBase("/", ""), "/");
});

void test("buildNuxtDevAssetBase merges a baseURL with no trailing slash before /_nuxt/", () => {
  assert.equal(buildNuxtDevAssetBase("/docs", "/_nuxt/"), "/docs/_nuxt/");
});

void test("buildNuxtDevAssetBase does NOT collapse internal double slashes", () => {
  // normalizeUrlPrefix only guards the leading/trailing slash; interior "//" survive.
  // This is surprising for a URL builder and is recorded as a possible bug.
  assert.equal(buildNuxtDevAssetBase("//docs//", "//_nuxt//"), "//docs///_nuxt//");
});

void test("buildNuxtCompilerOptions sets defaults and prepends the Takumi exclude", () => {
  assert.deepEqual(buildNuxtCompilerOptions("/root"), {
    devUrlBase: "/_nuxt/",
    handleNodeModulesVue: false,
    root: "/root",
    scanPatterns: [],
    exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
  });
});

void test("buildNuxtCompilerOptions merges a user exclude without double-merging the Takumi default", () => {
  const a = /\.a\.vue$/;
  const b = /\.b\.vue$/;
  const result = buildNuxtCompilerOptions("/root", "/", "/_nuxt/", { exclude: [a, b] });
  // Takumi default is prepended exactly once, followed by the user's patterns.
  assert.deepEqual(result.exclude, [NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE, a, b]);
});

void test("buildNuxtCompilerOptions with customRenderer:true and no exclude omits the exclude key", () => {
  const result = buildNuxtCompilerOptions("/root", "/", "/_nuxt/", { customRenderer: true });
  assert.equal("exclude" in result, false);
  assert.deepEqual(result, {
    devUrlBase: "/_nuxt/",
    handleNodeModulesVue: false,
    root: "/root",
    scanPatterns: [],
    customRenderer: true,
  });
});

void test("isVizeVirtualVueModuleId requires a NUL prefix while generated does not", () => {
  assert.equal(isVizeVirtualVueModuleId("\0/App.vue.ts"), true);
  assert.equal(isVizeGeneratedVueModuleId("\0/App.vue.ts"), true);
  // Without the NUL byte it is a generated id but not a virtual one.
  assert.equal(isVizeVirtualVueModuleId("/App.vue.ts"), false);
  assert.equal(isVizeGeneratedVueModuleId("/App.vue.ts"), true);
});

void test("the .vue.tsx extension is neither virtual nor generated", () => {
  // The regex anchors ".vue.ts" to "?" or end-of-string, so the trailing "x" excludes it.
  assert.equal(isVizeVirtualVueModuleId("\0/App.vue.tsx"), false);
  assert.equal(isVizeGeneratedVueModuleId("/App.vue.tsx"), false);
});

void test("ids without .vue are rejected and a query after .vue.ts is accepted", () => {
  assert.equal(isVizeVirtualVueModuleId("\0/App.ts"), false);
  assert.equal(isVizeGeneratedVueModuleId("/App.ts"), false);
  assert.equal(isVizeVirtualVueModuleId("\0/App.vue.ts?vue"), true);
  assert.equal(isVizeGeneratedVueModuleId("/App.vue.ts?vue"), true);
});

void test("isVizeJsxModuleId matches in-place .jsx and .tsx component modules", () => {
  // Raw JSX/TSX Vue components are compiled in place: the underlying Vite
  // plugin keeps the original id (no `.vue.ts` virtual), so the Nuxt bridge
  // must recognize them through this predicate to apply auto-imports etc.
  assert.equal(isVizeJsxModuleId("/components/Foo.jsx"), true);
  assert.equal(isVizeJsxModuleId("/components/Foo.tsx"), true);
  // A plain `?vue` dev-server query suffix still matches.
  assert.equal(isVizeJsxModuleId("/components/Foo.tsx?vue"), true);
  // Such modules are NOT seen as `.vue` virtual/generated modules.
  assert.equal(isVizeGeneratedVueModuleId("/components/Foo.tsx"), false);
  assert.equal(isVizeVirtualVueModuleId("\0/components/Foo.tsx"), false);
});

void test("isVizeJsxModuleId rejects non-JSX ids and asset-import queries", () => {
  assert.equal(isVizeJsxModuleId("/App.vue.ts"), false);
  assert.equal(isVizeJsxModuleId("/App.ts"), false);
  assert.equal(isVizeJsxModuleId("/App.js"), false);
  // `.jsx`/`.tsx` referenced as raw/url/worker assets are not component modules.
  assert.equal(isVizeJsxModuleId("/icon.tsx?raw"), false);
  assert.equal(isVizeJsxModuleId("/icon.tsx?url"), false);
  assert.equal(isVizeJsxModuleId("/worker.jsx?worker"), false);
  assert.equal(isVizeJsxModuleId("/worker.jsx?sharedworker"), false);
});

void test("normalizeVizeVirtualVueModuleId strips the NUL and the optional vize-ssr prefix", () => {
  assert.equal(normalizeVizeVirtualVueModuleId("\0/App.vue.ts"), "/App.vue");
  assert.equal(normalizeVizeVirtualVueModuleId("\0vize-ssr:/App.vue.ts"), "/App.vue");
});

void test("normalizeVizeVirtualVueModuleId preserves the query string but only strips the suffix .ts", () => {
  assert.equal(normalizeVizeVirtualVueModuleId("\0/App.vue.ts?vue"), "/App.vue?vue");
  // A ".ts" living inside the query must NOT be stripped.
  assert.equal(normalizeVizeVirtualVueModuleId("\0/App.vue.ts?v=1.ts"), "/App.vue?v=1.ts");
});

void test("normalizeVizeGeneratedVueModuleId handles the /@id/__x00__ and __x00__ wrappers", () => {
  assert.equal(normalizeVizeGeneratedVueModuleId("/@id/__x00__/App.vue.ts?vue"), "/App.vue?vue");
  assert.equal(normalizeVizeGeneratedVueModuleId("__x00__/App.vue.ts"), "/App.vue");
  assert.equal(normalizeVizeGeneratedVueModuleId("/App.vue.ts?vue&vize"), "/App.vue?vue&vize");
  // A ".ts" inside the query is preserved here too.
  assert.equal(normalizeVizeGeneratedVueModuleId("/App.vue.ts?v=1.ts"), "/App.vue?v=1.ts");
});

void test("normalizeNuxtInjectedKeysForVizeVirtualModule replaces every marker with a deterministic 10-char key", () => {
  const code = "a('$x1' /* nuxt-injected */); b('$x2' /* nuxt-injected */)";
  const id = "\0/repo/App.vue.ts";
  const normalizedId = "/repo/App.vue";

  const out = normalizeNuxtInjectedKeysForVizeVirtualModule(code, id);
  const expected = `a('$${stableNuxtKey(normalizedId, 1)}' /* nuxt-injected */); b('$${stableNuxtKey(normalizedId, 2)}' /* nuxt-injected */)`;
  assert.equal(out, expected);

  const keys = [...out.matchAll(/'\$([^']+)'/g)].map((m) => m[1]!);
  assert.deepEqual(
    keys.map((k) => k.length),
    [10, 10],
  );

  // Same id + index pair is stable across repeated calls.
  assert.equal(out, normalizeNuxtInjectedKeysForVizeVirtualModule(code, id));
});

void test("normalizeNuxtInjectedKeysForVizeVirtualModule leaves code without markers untouched", () => {
  const code = "const x = doThing();";
  assert.equal(normalizeNuxtInjectedKeysForVizeVirtualModule(code, "\0/repo/App.vue.ts"), code);
});

void test("stabilizeNuxtInjectedKeysForVizeVirtualModule injects a stable key into an unkeyed useFetch", () => {
  const id = "\0/repo/App.vue.ts";
  const normalizedId = "/repo/App.vue";
  const out = stabilizeNuxtInjectedKeysForVizeVirtualModule("useFetch('/api')", id);
  assert.equal(out, `useFetch('/api', '$${stableNuxtKey(normalizedId, 1)}' /* nuxt-injected */)`);
});

void test("stabilizeNuxtInjectedKeysForVizeVirtualModule does not double-inject an already-keyed call", () => {
  const id = "\0/repo/App.vue.ts";
  const normalizedId = "/repo/App.vue";
  // Already-keyed: no extra argument is appended, but the key is re-stabilized.
  const out = stabilizeNuxtInjectedKeysForVizeVirtualModule(
    "useFetch('/api', {}, '$abc' /* nuxt-injected */)",
    id,
  );
  assert.equal(
    out,
    `useFetch('/api', {}, '$${stableNuxtKey(normalizedId, 1)}' /* nuxt-injected */)`,
  );
  // Exactly one nuxt-injected marker remains.
  assert.equal(out.match(/nuxt-injected/g)?.length, 1);
});

void test("preserveExplicitVueImportsFromNuxtAutoImports restores a plain vue import moved to #imports", () => {
  const original = `import { ref } from "vue";\nconst x = ref(0);`;
  const injected = `import { ref, useState } from "#imports";\nconst x = ref(0);\nconst s = useState();`;
  assert.equal(
    preserveExplicitVueImportsFromNuxtAutoImports(original, injected),
    `import { ref } from "vue";\nimport { useState } from "#imports";\nconst x = ref(0);\nconst s = useState();`,
  );
});

void test("preserveExplicitVueImportsFromNuxtAutoImports restores an aliased import by its local name", () => {
  const original = `import { ref as _ref } from "vue";\nconst x = _ref(0);`;
  const injected = `import { ref as _ref, useState } from "#imports";\nconst x = _ref(0);\nconst s = useState();`;
  assert.equal(
    preserveExplicitVueImportsFromNuxtAutoImports(original, injected),
    `import { ref as _ref } from "vue";\nimport { useState } from "#imports";\nconst x = _ref(0);\nconst s = useState();`,
  );
});

void test("preserveExplicitVueImportsFromNuxtAutoImports drops the type modifier when restoring a type-only original specifier", () => {
  const original = `import { type Ref, ref } from "vue";\nconst x = ref(0);`;
  const injected = `import { Ref, ref, useState } from "#imports";\nconst x = ref(0);`;
  // The original "type Ref" is matched by local "Ref"; its restored raw drops the "type " prefix.
  assert.equal(
    preserveExplicitVueImportsFromNuxtAutoImports(original, injected),
    `import { Ref, ref } from "vue";\nimport { useState } from "#imports";\nconst x = ref(0);`,
  );
});

void test("preserveExplicitVueImportsFromNuxtAutoImports is a no-op when nothing needs restoring", () => {
  // No explicit vue import in the original => injected code is returned unchanged.
  const noVue = `import { ref } from "#imports";\nconst x = ref(0);`;
  assert.equal(preserveExplicitVueImportsFromNuxtAutoImports(`const x = 1;`, noVue), noVue);

  // Original has a vue import, but the injected code never moved it into #imports.
  const original = `import { ref } from "vue";`;
  const injected = `import { useState } from "#imports";\nconst s = useState();`;
  assert.equal(preserveExplicitVueImportsFromNuxtAutoImports(original, injected), injected);
});
