/**
 * `ModuleOutputInfo` now carries the two facts `insertBeforeSfcMainDefaultExport`
 * used to recover by parsing the module a second time (#3425): the end offset of
 * the `export default ...` statement, and whether its declaration is the
 * `_sfc_main` identifier. `generateOutput` already analyzed the module, so the
 * second parse was pure duplicated work -- 60 of 300 modules on the bench corpus
 * take a full oxc parse, and those same 60 were parsed twice.
 *
 * These fixtures are the output shapes the Rust SFC emitter can produce, so the
 * two new fields are pinned against every branch a real build takes rather than
 * against a synthetic module.
 */

import assert from "node:assert/strict";

import {
  analyzeModuleOutput,
  insertBeforeSfcMainDefaultExport,
  type ModuleOutputInfo,
} from "./utils/module-output.ts";

/** Emitted module shapes, named for the emitter path that produces them. */
const SHAPES: Record<string, string> = {
  // Template-only client build: `export function render`, no default export.
  templateOnlyClient: "export function render(_ctx, _cache) {\n  return null;\n}\n",
  // Template-only vapor: render is exported AND attached to `_sfc_main`.
  templateOnlyVapor:
    "export function render(_ctx) {\n  return null;\n}\nconst _sfc_main = { __vapor: true };\n_sfc_main.render = render;\nexport default _sfc_main",
  // Template-only SSR: `ssrRender` is a bare function, never exported.
  templateOnlySsr:
    "function ssrRender(_ctx, _push) {\n  _push('<div></div>');\n}\nconst _sfc_main = {};\n_sfc_main.ssrRender = ssrRender;\nexport default _sfc_main",
  // `<script>` + `<template>`: the render function is renamed and attached.
  scriptWithTemplate:
    "const _sfc_main = {\n  name: 'Foo'\n};\nfunction _sfc_render(_ctx) {\n  return null;\n}\n_sfc_main.render = _sfc_render;\nexport default _sfc_main",
  // `<script>` with no `<template>`.
  scriptOnly: "const _sfc_main = {\n  name: 'Foo'\n};\nexport default _sfc_main",
  // Template compile error fallback: the script alone, with no default export.
  templateErrorFallback: "const _sfc_main = {\n  name: 'Foo'\n};\n",
  // `<script setup>`, plain JS -- the shape the string fast path handles.
  scriptSetupPlain: "export default {\n  setup() {\n    return {};\n  }\n}",
  // `<script setup>`, TS -- also fast path, but with an import above it.
  scriptSetupDefineComponent:
    "import { defineComponent as _defineComponent } from 'vue';\nexport default /*@__PURE__*/_defineComponent({\n  setup() {\n    return {};\n  }\n})",
  // `export { render }` forces the full parse and is not an `_sfc_main` shape.
  namedRenderExportObject:
    "export { render };\nfunction render(_ctx) {\n  return null;\n}\nexport default { name: 'Foo' }",
};

const EXPECTED: Record<string, ModuleOutputInfo> = {
  templateOnlyClient: {
    hasDefaultExport: false,
    hasSfcMainDefined: false,
    hasNamedRenderExport: true,
    hasNamedSsrRenderExport: false,
    defaultExportKeywordEnd: null,
    defaultExportStart: null,
    defaultExportEnd: null,
    defaultExportIsSfcMain: false,
  },
  templateOnlyVapor: {
    hasDefaultExport: true,
    hasSfcMainDefined: true,
    hasNamedRenderExport: true,
    hasNamedSsrRenderExport: false,
    defaultExportKeywordEnd: 126,
    defaultExportStart: 112,
    defaultExportEnd: 136,
    defaultExportIsSfcMain: true,
  },
  templateOnlySsr: {
    hasDefaultExport: true,
    hasSfcMainDefined: true,
    hasNamedRenderExport: false,
    hasNamedSsrRenderExport: false,
    defaultExportKeywordEnd: 129,
    defaultExportStart: 115,
    defaultExportEnd: 139,
    defaultExportIsSfcMain: true,
  },
  scriptWithTemplate: {
    hasDefaultExport: true,
    hasSfcMainDefined: true,
    hasNamedRenderExport: false,
    hasNamedSsrRenderExport: false,
    defaultExportKeywordEnd: 129,
    defaultExportStart: 115,
    defaultExportEnd: 139,
    defaultExportIsSfcMain: true,
  },
  scriptOnly: {
    hasDefaultExport: true,
    hasSfcMainDefined: true,
    hasNamedRenderExport: false,
    hasNamedSsrRenderExport: false,
    defaultExportKeywordEnd: 51,
    defaultExportStart: 37,
    defaultExportEnd: 61,
    defaultExportIsSfcMain: true,
  },
  templateErrorFallback: {
    hasDefaultExport: false,
    hasSfcMainDefined: true,
    hasNamedRenderExport: false,
    hasNamedSsrRenderExport: false,
    defaultExportKeywordEnd: null,
    defaultExportStart: null,
    defaultExportEnd: null,
    defaultExportIsSfcMain: false,
  },
  // Fast path: `_sfc_main` does not occur, so the end offset is not computed
  // and the default export provably is not that identifier.
  scriptSetupPlain: {
    hasDefaultExport: true,
    hasSfcMainDefined: false,
    hasNamedRenderExport: false,
    hasNamedSsrRenderExport: false,
    defaultExportKeywordEnd: 14,
    defaultExportStart: 0,
    defaultExportEnd: null,
    defaultExportIsSfcMain: false,
  },
  scriptSetupDefineComponent: {
    hasDefaultExport: true,
    hasSfcMainDefined: false,
    hasNamedRenderExport: false,
    hasNamedSsrRenderExport: false,
    defaultExportKeywordEnd: 73,
    defaultExportStart: 59,
    defaultExportEnd: null,
    defaultExportIsSfcMain: false,
  },
  namedRenderExportObject: {
    hasDefaultExport: true,
    hasSfcMainDefined: false,
    hasNamedRenderExport: true,
    hasNamedSsrRenderExport: false,
    defaultExportKeywordEnd: 74,
    defaultExportStart: 60,
    defaultExportEnd: 90,
    defaultExportIsSfcMain: false,
  },
};

const NAMES = Object.keys(SHAPES);

assert.deepEqual(
  NAMES.map((name) => [name, analyzeModuleOutput(SHAPES[name])]),
  NAMES.map((name) => [name, EXPECTED[name]]),
  "analyzeModuleOutput reports the default export's end offset and whether it is `_sfc_main`",
);

// Reusing the caller's analysis must produce exactly what a fresh parse does,
// in both insertion modes.
const INSERTION = '_sfc_main.__scopeId = "data-v-1a2b3c4d";';
for (const normalizeSemicolon of [false, true]) {
  assert.deepEqual(
    NAMES.map((name) => [
      name,
      insertBeforeSfcMainDefaultExport(SHAPES[name], INSERTION, {
        normalizeSemicolon,
        moduleInfo: EXPECTED[name],
      }),
    ]),
    NAMES.map((name) => [
      name,
      insertBeforeSfcMainDefaultExport(SHAPES[name], INSERTION, { normalizeSemicolon }),
    ]),
    `passing moduleInfo changes nothing (normalizeSemicolon: ${normalizeSemicolon})`,
  );
}

// And the insertion itself is still the documented rewrite, not just
// self-consistent: pinned for one `_sfc_main` shape and one non-`_sfc_main`
// shape, which the function must leave alone.
assert.equal(
  insertBeforeSfcMainDefaultExport(SHAPES.scriptOnly, INSERTION, {
    moduleInfo: EXPECTED.scriptOnly,
  }),
  "const _sfc_main = {\n  name: 'Foo'\n};\n_sfc_main.__scopeId = \"data-v-1a2b3c4d\";\nexport default _sfc_main",
);
assert.equal(
  insertBeforeSfcMainDefaultExport(SHAPES.scriptOnly, INSERTION, {
    normalizeSemicolon: true,
    moduleInfo: EXPECTED.scriptOnly,
  }),
  "const _sfc_main = {\n  name: 'Foo'\n};\n_sfc_main.__scopeId = \"data-v-1a2b3c4d\";\nexport default _sfc_main;",
);
assert.equal(
  insertBeforeSfcMainDefaultExport(SHAPES.scriptSetupPlain, INSERTION, {
    moduleInfo: EXPECTED.scriptSetupPlain,
  }),
  SHAPES.scriptSetupPlain,
);

console.log("✅ vite-plugin-vize module output info tests passed!");
