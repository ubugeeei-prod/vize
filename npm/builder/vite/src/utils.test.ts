/**
 * Unit tests for vite-plugin-vize utils
 *
 * Run with: npx tsx src/utils.test.ts
 *
 * These tests cover various edge cases and bug fixes.
 */

import assert from "node:assert";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import os from "node:os";
import path from "node:path";
import { snapshot, test } from "node:test";
import { resolveConfigExport } from "../../../cli/src/config.ts";
import { createFilter, generateOutput } from "./utils/index.ts";
import { resolveCssImports } from "./utils/css.ts";

snapshot.setDefaultSnapshotSerializers([
  (value) => (typeof value === "string" ? value : JSON.stringify(value, null, 2)),
]);
snapshot.setResolveSnapshotPath(() =>
  fileURLToPath(new URL("./utils.test.ts.snapshot", import.meta.url)),
);

// =============================================================================
// Test: Shared config RegExp filters
// =============================================================================

const regexpConfig = await resolveConfigExport({
  vite: {
    include: [/\.vue$/],
    exclude: [/node_modules/],
  },
});
const regexpInclude = Array.isArray(regexpConfig.vite?.include)
  ? regexpConfig.vite.include[0]
  : regexpConfig.vite?.include;
const regexpExclude = Array.isArray(regexpConfig.vite?.exclude)
  ? regexpConfig.vite.exclude[0]
  : regexpConfig.vite?.exclude;
assert.ok(regexpInclude instanceof RegExp, "Shared config should preserve include RegExp values");
assert.ok(regexpExclude instanceof RegExp, "Shared config should preserve exclude RegExp values");

const regexpFilter = createFilter(regexpConfig.vite?.include, regexpConfig.vite?.exclude);
assert.equal(regexpFilter("/src/App.vue"), true, "RegExp include should match Vue files");
assert.equal(
  regexpFilter("/src/node_modules/App.vue"),
  false,
  "RegExp exclude should filter node_modules",
);

// =============================================================================
// Test: Non-script-setup SFC _sfc_main duplication fix
// =============================================================================

/**
 * Simulates the generateOutput logic for detecting _sfc_main
 */
function hasSfcMainDefined(code: string): boolean {
  return /\bconst\s+_sfc_main\s*=/.test(code);
}

function hasExportDefault(code: string): boolean {
  return /^export default /m.test(code);
}

// Test 1: Script setup component (should NOT have _sfc_main defined)
const scriptSetupCode = `
import { openBlock as _openBlock } from "vue"
export default {
  __name: "Component",
  setup() { return {} }
}
`;

assert.strictEqual(
  hasSfcMainDefined(scriptSetupCode),
  false,
  "Script setup component should not have _sfc_main pre-defined",
);
assert.strictEqual(
  hasExportDefault(scriptSetupCode),
  true,
  "Script setup component should have export default",
);

// Test 2: Non-script-setup component (should have _sfc_main defined)
const nonScriptSetupCode = `
import { openBlock as _openBlock } from "vue"
const __default__ = { name: 'MyComponent' }
const _sfc_main = __default__
export default _sfc_main
`;

assert.strictEqual(
  hasSfcMainDefined(nonScriptSetupCode),
  true,
  "Non-script-setup component should have _sfc_main pre-defined",
);
assert.strictEqual(
  hasExportDefault(nonScriptSetupCode),
  true,
  "Non-script-setup component should have export default",
);

// Test 3: Variation with different spacing
const variationCode = `const  _sfc_main   =  __default__`;
assert.strictEqual(
  hasSfcMainDefined(variationCode),
  true,
  "Should detect _sfc_main with various whitespace",
);

// Test 3b: Template-only SFCs need a default export shim
const templateOnlyCode = `
export function render() {
  return null
}
const _sfc_main = {}
_sfc_main.render = render
export default _sfc_main
`;
assert.strictEqual(
  hasExportDefault(templateOnlyCode),
  true,
  "Template-only components should still expose a default export",
);

// Test 3b-ssr: SSR template-only SFCs need an ssrRender default export shim
void test("generateOutput snapshots SSR template-only default export shim", (t) => {
  const output = generateOutput(
    {
      code: `
export function ssrRender() {
  return null
}
`,
      scopeId: "ssrtemplate",
      hasScoped: false,
      styles: [],
    },
    {
      isProduction: true,
      isDev: false,
    },
  );
  t.assert.snapshot(output);
});

// Test 3c: Inline-template script setup must not claim template-only HMR
void test("generateOutput snapshots inline-template full-reload HMR downgrade", (t) => {
  const output = generateOutput(
    {
      code: `
export default {
  __name: "InlineOnly",
  setup() {
    return (_ctx, _cache) => null
  }
}
`,
      scopeId: "inlinehmr",
      hasScoped: false,
      styles: [],
    },
    {
      isProduction: false,
      isDev: true,
      hmrUpdateType: "template-only",
    },
  );
  t.assert.snapshot(output);
});

// Test 3d: Components with standalone render may keep template-only HMR
void test("generateOutput snapshots standalone render template-only HMR", (t) => {
  const output = generateOutput(
    {
      code: `
export function render() {
  return null
}
const _sfc_main = {}
_sfc_main.render = render
export default _sfc_main
`,
      scopeId: "separatehmr",
      hasScoped: false,
      styles: [],
    },
    {
      isProduction: false,
      isDev: true,
      hmrUpdateType: "template-only",
    },
  );
  t.assert.snapshot(output);
});

// Test 3e: CSS Modules bindings should be injected even when export default has no semicolon
void test("generateOutput snapshots CSS Modules binding injection", (t) => {
  const output = generateOutput(
    {
      code: `
const _sfc_main = { name: "ModuleButton" }
export default _sfc_main
`,
      scopeId: "cssmodule",
      hasScoped: false,
      styles: [
        {
          content: ".root { color: red; }",
          lang: "css",
          scoped: false,
          module: "buttonStyles",
          index: 0,
        },
      ],
    },
    {
      isProduction: false,
      isDev: false,
      filePath: "/src/ModuleButton.vue",
    },
  );
  t.assert.snapshot(output);
});

// Test 3f: delegated SFC styles should load after imported child components
void test("generateOutput snapshots delegated style import order", (t) => {
  const output = generateOutput(
    {
      code: `
import { openBlock as _openBlock } from "vue"
const _hoisted_1 = {}
import Child from "./Child.vue"
const _sfc_main = { name: "Parent" }
export default _sfc_main
`,
      scopeId: "styleorder",
      hasScoped: false,
      styles: [
        {
          content: ".subscribe { position: absolute; }",
          lang: "css",
          scoped: false,
          module: true,
          index: 0,
        },
      ],
    },
    {
      isProduction: false,
      isDev: false,
      filePath: "/src/Parent.vue",
    },
  );
  t.assert.snapshot(output);
});

// Test 3g: production extracted plain CSS should flow through Vite's CSS pipeline
void test("generateOutput snapshots production extracted plain CSS", (t) => {
  const output = generateOutput(
    {
      code: `
import { openBlock as _openBlock } from "vue"
const _sfc_main = { name: "ProdStyles" }
export default _sfc_main
`,
      scopeId: "prodstyles",
      hasScoped: false,
      styles: [
        {
          content: ".root { color: red; }",
          lang: "css",
          scoped: false,
          module: false,
          index: 0,
        },
      ],
      css: ".root { color: red; }",
    },
    {
      isProduction: true,
      isDev: false,
      extractCss: true,
      filePath: "/src/ProdStyles.vue",
    },
  );
  t.assert.snapshot(output);
});

// Test 3h: production scoped CSS extraction should still delegate plain CSS to Vite
void test("generateOutput snapshots production scoped CSS extraction", (t) => {
  const output = generateOutput(
    {
      code: `
const _sfc_main = { name: "PlainCss" }
export default _sfc_main
`,
      scopeId: "plaincss",
      hasScoped: true,
      styles: [
        {
          content: ".root { color: red; }",
          lang: "css",
          scoped: true,
          module: false,
          index: 0,
        },
      ],
    },
    {
      isProduction: true,
      isDev: false,
      extractCss: true,
      filePath: "/src/PlainCss.vue",
    },
  );

  t.assert.snapshot(output);
});

// =============================================================================
// Test: Query parameter preservation in relative imports
// =============================================================================

function splitPathAndQuery(id: string): [string, string] {
  const [pathPart, queryPart] = id.split("?");
  const querySuffix = queryPart ? `?${queryPart}` : "";
  return [pathPart, querySuffix];
}

// Test 4: Import with ?inline query
const [path1, query1] = splitPathAndQuery("./style.css?inline");
assert.strictEqual(path1, "./style.css", "Path should be extracted");
assert.strictEqual(query1, "?inline", "Query should be preserved");

// Test 5: Import with ?raw query
const [path2, query2] = splitPathAndQuery("./data.json?raw");
assert.strictEqual(path2, "./data.json", "Path should be extracted");
assert.strictEqual(query2, "?raw", "Query should be preserved");

// Test 6: Import without query
const [path3, query3] = splitPathAndQuery("./component.vue");
assert.strictEqual(path3, "./component.vue", "Path should be unchanged");
assert.strictEqual(query3, "", "No query suffix");

// Test 7: Import with multiple query params
const [path4, query4] = splitPathAndQuery("./file.txt?raw&inline");
assert.strictEqual(path4, "./file.txt", "Path should be extracted");
assert.strictEqual(query4, "?raw&inline", "All query params preserved");

// =============================================================================
// Test: Already-resolved path detection
// =============================================================================

function isAlreadyResolved(id: string): boolean {
  return id.includes("/dist/") || id.includes("/lib/") || id.includes("/es/");
}

// Test 8: dist path
assert.strictEqual(
  isAlreadyResolved("/node_modules/some-pkg/dist/index.mjs"),
  true,
  "Should detect /dist/ path as resolved",
);

// Test 9: lib path
assert.strictEqual(
  isAlreadyResolved("/node_modules/some-pkg/lib/index.js"),
  true,
  "Should detect /lib/ path as resolved",
);

// Test 10: es path (ESM build)
assert.strictEqual(
  isAlreadyResolved("/node_modules/some-pkg/es/index.mjs"),
  true,
  "Should detect /es/ path as resolved",
);

// Test 11: Regular package import
assert.strictEqual(
  isAlreadyResolved("lodash-es"),
  false,
  "Package name should not be detected as resolved",
);

// Test 12: Relative import
assert.strictEqual(
  isAlreadyResolved("./components/Button.vue"),
  false,
  "Relative import should not be detected as resolved",
);

// =============================================================================
// Test: scopeId generation
// =============================================================================

function generateScopeId(filename: string): string {
  // Simplified hash function for testing
  let hash = 0;
  for (let i = 0; i < filename.length; i++) {
    const char = filename.charCodeAt(i);
    hash = (hash << 5) - hash + char;
    hash = hash & hash; // Convert to 32bit integer
  }
  return Math.abs(hash).toString(16).substring(0, 8);
}

// Test 13: Different files should have different scope IDs
const scope1 = generateScopeId("src/components/Button.vue");
const scope2 = generateScopeId("src/components/Input.vue");
assert.notStrictEqual(scope1, scope2, "Different files should have different scope IDs");

// Test 14: Same file should have same scope ID
const scope3 = generateScopeId("src/components/Button.vue");
assert.strictEqual(scope1, scope3, "Same file should have same scope ID");

// =============================================================================
// Test: resolveCssImports — @custom-media resolution
// =============================================================================

/**
 * Minimal version of resolveCssImports that handles @custom-media and :deep()
 * without filesystem access (for unit testing the pure transformation logic).
 */
function resolveCssTransforms(css: string): string {
  const customMedia = new Map<string, string>();

  // Parse @custom-media definitions
  const cmRe = /@custom-media\s+(--[\w-]+)\s+(.+?)\s*;/g;
  let m: RegExpExecArray | null;
  while ((m = cmRe.exec(css)) !== null) {
    customMedia.set(m[1], m[2]);
  }

  // Remove @custom-media definitions
  let result = css.replace(/^@custom-media\s+[^;]+;\s*$/gm, "");

  // Replace (--name) in @media rules
  for (const [name, query] of customMedia) {
    const escaped = name.replace(/[-/\\^$*+?.()|[\]{}]/g, "\\$&");
    result = result.replace(new RegExp(`\\(${escaped}\\)`, "g"), query);
  }

  // Unwrap :deep()
  result = result.replace(/:deep\(([^()]*(?:\([^()]*\))*[^()]*)\)/g, "$1");

  // Clean up excessive blank lines
  result = result.replace(/\n{3,}/g, "\n\n");

  return result;
}

// Test 15: @custom-media resolution
void test("resolveCssTransforms snapshots @custom-media resolution", (t) => {
  const css = `@custom-media --mobile (max-width: 768px);
.foo { color: red; }
@media (--mobile) { .foo { font-size: 12px; } }`;
  const result = resolveCssTransforms(css);
  t.assert.snapshot(result);
});

// Test 16: Multiple @custom-media definitions
void test("resolveCssTransforms snapshots multiple @custom-media definitions", (t) => {
  const css = `@custom-media --mobile (max-width: 768px);
@custom-media --desktop (min-width: 1024px);
@media (--mobile) { .a { color: red; } }
@media (--desktop) { .b { color: blue; } }`;
  const result = resolveCssTransforms(css);
  t.assert.snapshot(result);
});

// Test 17: :deep() unwrapping
void test("resolveCssTransforms snapshots :deep() unwrapping", (t) => {
  const css = `.parent :deep(.child) { color: red; }`;
  const result = resolveCssTransforms(css);
  t.assert.snapshot(result);
});

// Test 18: :deep() with nested parens
void test("resolveCssTransforms snapshots nested :deep() unwrapping", (t) => {
  const css = `.parent :deep(.child:nth-child(2)) { color: red; }`;
  const result = resolveCssTransforms(css);
  t.assert.snapshot(result);
});

// Test 19: CSS without @custom-media passes through unchanged
{
  const css = `.foo { color: red; }\n@media (max-width: 768px) { .foo { font-size: 12px; } }`;
  const result = resolveCssTransforms(css);
  assert.strictEqual(result, css, "CSS without @custom-media should pass through unchanged");
}

// Test 20: dev asset URLs honor configured base path
void test("resolveCssImports snapshots dev asset URLs with base path", (t) => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-"));
  const assetPath = path.join(tempDir, "assets", "noise.png");
  fs.mkdirSync(path.dirname(assetPath), { recursive: true });
  fs.writeFileSync(assetPath, "");

  const result = resolveCssImports(
    `.hero { background-image: url("~/assets/noise.png"); }`,
    path.join(tempDir, "Component.vue"),
    [{ find: "~/", replacement: `${tempDir}/` }],
    true,
    "/_nuxt/",
  );

  t.assert.snapshot(result.replace(assetPath.replace(/\\/g, "/"), "<asset>"));
});

// Test 21: CSS aliases should not match package prefixes
void test("resolveCssImports snapshots alias and package-like URLs", (t) => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-alias-"));
  const srcAssetPath = path.join(tempDir, "src", "assets", "logo.svg");
  const packageAssetPath = path.join(tempDir, "node_modules", "@scope", "icons", "logo.svg");
  fs.mkdirSync(path.dirname(srcAssetPath), { recursive: true });
  fs.mkdirSync(path.dirname(packageAssetPath), { recursive: true });
  fs.writeFileSync(srcAssetPath, "");
  fs.writeFileSync(packageAssetPath, "");

  const result = resolveCssImports(
    `
.local { background-image: url("@/assets/logo.svg"); }
.package { background-image: url("@scope/icons/logo.svg"); }
`,
    path.join(tempDir, "Component.vue"),
    [{ find: "@", replacement: path.join(tempDir, "src") }],
    true,
  );

  t.assert.snapshot(result.replace(srcAssetPath.replace(/\\/g, "/"), "<src-asset>"));
});

// Test 22: CSS aliases should support RegExp find rules
void test("resolveCssImports snapshots RegExp alias URL resolution", (t) => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-css-regexp-alias-"));
  const assetPath = path.join(tempDir, "src", "assets", "logo.svg");
  fs.mkdirSync(path.dirname(assetPath), { recursive: true });
  fs.writeFileSync(assetPath, "");

  const result = resolveCssImports(
    `.logo { background-image: url("@app/assets/logo.svg"); }`,
    path.join(tempDir, "Component.vue"),
    [{ find: /^@app\/(.+)$/, replacement: path.join(tempDir, "src", "$1") }],
    true,
  );

  t.assert.snapshot(result.replace(assetPath.replace(/\\/g, "/"), "<asset>"));
});

// =============================================================================
// Test: applyDefineReplacements
// =============================================================================

function applyDefineReplacements(code: string, defines: Record<string, string>): string {
  const sortedKeys = Object.keys(defines).sort((a, b) => b.length - a.length);
  let result = code;
  for (const key of sortedKeys) {
    if (!result.includes(key)) continue;
    const escaped = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const re = new RegExp(escaped + "(?![\\w$.])", "g");
    result = result.replace(re, defines[key]);
  }
  return result;
}

// Test 20: Basic define replacement
void test("applyDefineReplacements snapshots basic define replacement", (t) => {
  const code = `if (import.meta.vfFeatures.photoSection) { show(); }`;
  const defines = { "import.meta.vfFeatures.photoSection": "true" };
  const result = applyDefineReplacements(code, defines);
  t.assert.snapshot(result);
});

// Test 21: Should not replace partial matches
void test("applyDefineReplacements snapshots partial-match preservation", (t) => {
  const code = `import.meta.vfFeatures.photoSectionEnabled`;
  const defines = { "import.meta.vfFeatures.photoSection": "true" };
  const result = applyDefineReplacements(code, defines);
  t.assert.snapshot(result);
});

// Test 22: Longest key matched first
void test("applyDefineReplacements snapshots longest key precedence", (t) => {
  const code = `import.meta.env.MODE + import.meta.env.MODE_DETAIL`;
  const defines = {
    "import.meta.env.MODE": '"production"',
    "import.meta.env.MODE_DETAIL": '"full"',
  };
  const result = applyDefineReplacements(code, defines);
  t.assert.snapshot(result);
});

// Test 23: No replacement when key not present
{
  const code = `const x = 42;`;
  const defines = { "import.meta.vfFeatures.foo": "true" };
  const result = applyDefineReplacements(code, defines);
  assert.strictEqual(result, code, "Should not modify code when key is absent");
}

// =============================================================================
// Test: isBuiltinDefine
// =============================================================================

const BUILTIN_DEFINE_PREFIXES = [
  "import.meta.server",
  "import.meta.client",
  "import.meta.dev",
  "import.meta.test",
  "import.meta.prerender",
  "import.meta.env",
  "import.meta.hot",
  "__VUE_",
  "__NUXT_",
  "process.env",
];

function isBuiltinDefine(key: string): boolean {
  return BUILTIN_DEFINE_PREFIXES.some(
    (prefix) => key === prefix || key.startsWith(prefix + ".") || key.startsWith(prefix + "_"),
  );
}

// Test 24: Built-in define detection
assert.strictEqual(isBuiltinDefine("import.meta.env"), true, "import.meta.env is builtin");
assert.strictEqual(
  isBuiltinDefine("import.meta.env.MODE"),
  true,
  "import.meta.env.MODE is builtin",
);
assert.strictEqual(isBuiltinDefine("import.meta.server"), true, "import.meta.server is builtin");
assert.strictEqual(isBuiltinDefine("import.meta.client"), true, "import.meta.client is builtin");
assert.strictEqual(isBuiltinDefine("import.meta.hot"), true, "import.meta.hot is builtin");
// Note: __VUE_OPTIONS_API__ starts with __VUE_O, not __VUE__, so it is NOT matched
// by the current prefix + "_" check. Only __VUE__ (double underscore) would match.
assert.strictEqual(
  isBuiltinDefine("__VUE_OPTIONS_API__"),
  false,
  "__VUE_OPTIONS_API__ does not match __VUE__ prefix",
);
assert.strictEqual(
  isBuiltinDefine("__VUE__SOMETHING"),
  true,
  "__VUE__ (double underscore) is builtin",
);
assert.strictEqual(
  isBuiltinDefine("__NUXT__SOMETHING"),
  true,
  "__NUXT__ (double underscore) is builtin",
);
assert.strictEqual(isBuiltinDefine("process.env"), true, "process.env is builtin");
assert.strictEqual(
  isBuiltinDefine("process.env.NODE_ENV"),
  true,
  "process.env.NODE_ENV is builtin",
);

// Test 25: Non-builtin defines
assert.strictEqual(
  isBuiltinDefine("import.meta.vfFeatures"),
  false,
  "Custom define is not builtin",
);
assert.strictEqual(
  isBuiltinDefine("import.meta.vfFeatures.photoSection"),
  false,
  "Custom nested define is not builtin",
);
assert.strictEqual(isBuiltinDefine("MY_CUSTOM_FLAG"), false, "Custom flag is not builtin");

// =============================================================================
// All tests passed
// =============================================================================

console.log("✅ All vite-plugin-vize utils tests passed!");
