import assert from "node:assert/strict";
import test from "node:test";

import {
  DEFAULT_NUXT_BRIDGE_OPTIONS,
  NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
  resolveNuxtBridgeOptions,
  resolveNuxtCompilerOptions,
  resolveNuxtDevOptions,
  resolveNuxtMuseaOptions,
  resolveNuxtUnoCssOptions,
} from "./options.ts";

void test("resolveNuxtCompilerOptions merges an internal baseURL with buildAssetsDir into devUrlBase", () => {
  const resolved = resolveNuxtCompilerOptions("/repo/app", "/2025/docs/", "_nuxt", true);
  assert.notEqual(resolved, false, "compiler should stay enabled");
  assert.deepEqual(
    resolved,
    {
      devUrlBase: "/2025/docs/_nuxt/",
      exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
      root: "/repo/app",
      scanPatterns: [],
    },
    "an internal baseURL should prefix the normalized buildAssetsDir",
  );
});

void test("resolveNuxtCompilerOptions defaults buildAssetsDir only when undefined, not when empty", () => {
  assert.deepEqual(
    resolveNuxtCompilerOptions("/repo/app", "/", undefined, true),
    {
      devUrlBase: "/_nuxt/",
      exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
      root: "/repo/app",
      scanPatterns: [],
    },
    "undefined buildAssetsDir should fall back to /_nuxt/",
  );

  assert.deepEqual(
    resolveNuxtCompilerOptions("/repo/app", "/", "", true),
    {
      devUrlBase: "/",
      exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
      root: "/repo/app",
      scanPatterns: [],
    },
    "an empty buildAssetsDir normalizes to a bare / instead of /_nuxt/",
  );
});

void test("resolveNuxtCompilerOptions treats Vue 0.11 as host-compiler legacy", () => {
  assert.deepEqual(
    resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", true, {
      supportsViteCompiler: true,
      vueVersion: 0.11,
    }),
    {
      compatibility: {
        vueVersion: 0.11,
        hostCompiler: true,
      },
      devUrlBase: "/_nuxt/",
      exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
      root: "/repo/app",
      scanPatterns: [],
      vueVersion: 0.11,
    },
    "the oldest legacy Vue version should enable host-compiler mode",
  );
});

void test("resolveNuxtCompilerOptions treats the legacy string Vue version as host-compiler", () => {
  assert.deepEqual(
    resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", true, {
      supportsViteCompiler: true,
      vueVersion: "legacy",
    }),
    {
      compatibility: {
        vueVersion: "legacy",
        hostCompiler: true,
      },
      devUrlBase: "/_nuxt/",
      exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
      root: "/repo/app",
      scanPatterns: [],
      vueVersion: "legacy",
    },
    "the legacy string Vue version should enable host-compiler mode",
  );
});

void test("resolveNuxtCompilerOptions disables Vite for Nuxt 2 without a Vite builder", () => {
  assert.equal(
    resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", true, {
      supportsViteCompiler: false,
      nuxtVersion: 2,
    }),
    false,
    "Nuxt 2 without supportsViteCompiler and without forceViteCompiler should not register Vite",
  );
});

void test("resolveNuxtCompilerOptions keeps the Takumi exclude when overrides.exclude is an empty array", () => {
  assert.deepEqual(
    resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", { exclude: [] }),
    {
      devUrlBase: "/_nuxt/",
      exclude: [NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE],
      root: "/repo/app",
      scanPatterns: [],
    },
    "an empty user exclude array still merges the Takumi OG-image exclude",
  );
});

void test("resolveNuxtCompilerOptions drops the Takumi exclude when customRenderer is true", () => {
  const resolved = resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", {
    customRenderer: true,
  });
  assert.deepEqual(
    resolved,
    {
      customRenderer: true,
      devUrlBase: "/_nuxt/",
      root: "/repo/app",
      scanPatterns: [],
    },
    "customRenderer without a user exclude should omit exclude entirely",
  );
  assert.equal(
    Object.prototype.hasOwnProperty.call(resolved, "exclude"),
    false,
    "no exclude key should be present for a custom renderer",
  );
});

void test("resolveNuxtCompilerOptions with empty overrides preserves the bare Takumi exclude default", () => {
  assert.deepEqual(
    resolveNuxtCompilerOptions("/repo/app", "/", "/_nuxt/", {}),
    {
      devUrlBase: "/_nuxt/",
      exclude: NUXT_OG_IMAGE_RENDERER_SFC_EXCLUDE,
      root: "/repo/app",
      scanPatterns: [],
    },
    "empty overrides should not clobber the default Takumi exclude (kept as the bare regex)",
  );
});

void test("resolveNuxtBridgeOptions returns every flag false when bridge is false", () => {
  assert.deepEqual(
    resolveNuxtBridgeOptions(false),
    {
      autoImports: false,
      components: false,
      i18n: false,
      stableInjectedKeys: false,
    },
    "bridge=false should disable all four transform bridges",
  );
});

void test("resolveNuxtBridgeOptions returns enabled defaults for true, null, and undefined", () => {
  const expected = { ...DEFAULT_NUXT_BRIDGE_OPTIONS };
  assert.deepEqual(resolveNuxtBridgeOptions(true), expected, "bridge=true uses defaults");
  assert.deepEqual(resolveNuxtBridgeOptions(null), expected, "bridge=null uses defaults");
  assert.deepEqual(resolveNuxtBridgeOptions(undefined), expected, "bridge=undefined uses defaults");
});

void test("resolveNuxtBridgeOptions merges a partial object and treats explicit undefined as default", () => {
  assert.deepEqual(
    resolveNuxtBridgeOptions({ autoImports: false }),
    {
      autoImports: false,
      components: true,
      i18n: true,
      stableInjectedKeys: true,
    },
    "a single false flag should merge while the rest stay defaulted to true",
  );

  assert.deepEqual(
    resolveNuxtBridgeOptions({ i18n: undefined }),
    { ...DEFAULT_NUXT_BRIDGE_OPTIONS },
    "an explicitly undefined flag should fall back to its default",
  );
});

void test("resolveNuxtUnoCssOptions resolves false, true/null, and originalSource variants", () => {
  assert.equal(resolveNuxtUnoCssOptions(false), false, "false disables the UnoCSS bridge");
  assert.deepEqual(resolveNuxtUnoCssOptions(true), { originalSource: {} }, "true enables defaults");
  assert.deepEqual(resolveNuxtUnoCssOptions(null), { originalSource: {} }, "null enables defaults");
  assert.deepEqual(
    resolveNuxtUnoCssOptions({ originalSource: false }),
    { originalSource: false },
    "originalSource=false is preserved verbatim",
  );
  assert.deepEqual(
    resolveNuxtUnoCssOptions({ originalSource: true }),
    { originalSource: {} },
    "originalSource=true normalizes to an empty options object",
  );
  assert.deepEqual(
    resolveNuxtUnoCssOptions({ originalSource: null }),
    { originalSource: {} },
    "originalSource=null normalizes to an empty options object",
  );
});

void test("resolveNuxtUnoCssOptions passes an originalSource object through, including maxBytes 0", () => {
  assert.deepEqual(
    resolveNuxtUnoCssOptions({ originalSource: { maxBytes: 0 } }),
    { originalSource: { maxBytes: 0 } },
    "a falsy-but-present maxBytes:0 should survive the passthrough",
  );
});

void test("resolveNuxtDevOptions defaults for null, undefined, and empty object", () => {
  const expected = { stylesheetLinks: true };
  assert.deepEqual(resolveNuxtDevOptions(null), expected, "null yields defaults");
  assert.deepEqual(resolveNuxtDevOptions(undefined), expected, "undefined yields defaults");
  assert.deepEqual(resolveNuxtDevOptions({}), expected, "empty object yields defaults");
});

void test("resolveNuxtDevOptions honors explicit stylesheetLinks and spreads unknown fields through", () => {
  assert.deepEqual(
    resolveNuxtDevOptions({ stylesheetLinks: false }),
    { stylesheetLinks: false },
    "explicit stylesheetLinks:false should be honored",
  );

  // Observed behavior: unknown fields are NOT stripped despite the Required<VizeNuxtDevOptions>
  // return type; the implementation spreads `...dev` so extra keys leak through at runtime.
  assert.deepEqual(
    resolveNuxtDevOptions({ stylesheetLinks: true, unknownField: 123 } as never),
    { stylesheetLinks: true, unknownField: 123 },
    "extra unknown fields are spread through unchanged",
  );
});

void test("resolveNuxtMuseaOptions resolves true, false/null, and object passthrough", () => {
  assert.deepEqual(resolveNuxtMuseaOptions(true), {}, "true enables Musea with empty defaults");
  assert.equal(resolveNuxtMuseaOptions(false), false, "false disables Musea");
  assert.equal(resolveNuxtMuseaOptions(null), false, "null disables Musea");
  assert.deepEqual(
    resolveNuxtMuseaOptions({ include: ["**/*.art.vue"] }),
    { include: ["**/*.art.vue"] },
    "an object should pass through unchanged",
  );
});
