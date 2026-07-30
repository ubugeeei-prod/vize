/**
 * `embedsInlineCss` is the single source of truth for "does the module output
 * contain `compiled.css`". `loadCompiledSfcModule` consults it before paying for
 * `@import` resolution, so if it ever disagreed with `generateOutput` the plugin
 * would either do work it discards again (the defect it was introduced to fix)
 * or -- much worse -- emit CSS whose `@import`s were never resolved.
 *
 * The matrix test below therefore does not assert the predicate against a
 * hand-written expectation alone: it also compares it against what
 * `generateOutput` actually emits for the same inputs, so the two cannot drift.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import { embedsInlineCss, generateOutput } from "./utils/index.ts";
import type { CompiledModule, StyleBlockInfo } from "./types.ts";

const CSS_MARKER = ".marker-embedded-inline{color:red}";

function styleBlock(overrides: Partial<StyleBlockInfo> = {}): StyleBlockInfo {
  return {
    content: CSS_MARKER,
    src: null,
    lang: "css",
    scoped: false,
    module: false,
    index: 0,
    ...overrides,
  };
}

function compiled(overrides: Partial<CompiledModule> = {}): CompiledModule {
  return {
    code: 'const _sfc_main = { name: "Marker" }\nexport default _sfc_main\n',
    css: CSS_MARKER,
    scopeId: "marker01",
    hasScoped: false,
    styles: [styleBlock()],
    ...overrides,
  } as CompiledModule;
}

const FILE = "/src/Marker.vue";

/**
 * Every case that decides the branch, named so a failure says which one broke.
 * `expected` is the predicate's answer; the test also derives the same answer
 * from `generateOutput` and requires the two to agree.
 */
const cases = [
  {
    name: "dev, plain scoped style",
    compiled: compiled({ hasScoped: true, styles: [styleBlock({ scoped: true })] }),
    options: { isProduction: false, isDev: true, extractCss: false, filePath: FILE },
    expected: true,
  },
  {
    name: "dev, no style block but css present",
    compiled: compiled({ styles: [] }),
    options: { isProduction: false, isDev: true, extractCss: false, filePath: FILE },
    expected: true,
  },
  {
    name: "dev, no filePath",
    compiled: compiled(),
    options: { isProduction: false, isDev: true, extractCss: false },
    expected: true,
  },
  {
    name: "dev, SSR",
    compiled: compiled(),
    options: { isProduction: false, isDev: false, ssr: true, extractCss: false, filePath: FILE },
    expected: false,
  },
  {
    name: "dev, preprocessor block delegated to Vite",
    compiled: compiled({ styles: [styleBlock({ lang: "scss" })] }),
    options: { isProduction: false, isDev: true, extractCss: false, filePath: FILE },
    expected: false,
  },
  {
    name: "dev, CSS Modules block delegated to Vite",
    compiled: compiled({ styles: [styleBlock({ module: "$style" })] }),
    options: { isProduction: false, isDev: true, extractCss: false, filePath: FILE },
    expected: false,
  },
  {
    name: "production client build with CSS extraction",
    compiled: compiled(),
    options: { isProduction: true, isDev: false, extractCss: true, filePath: FILE },
    expected: false,
  },
  {
    name: "production client build, extraction off",
    compiled: compiled(),
    options: { isProduction: true, isDev: false, extractCss: false, filePath: FILE },
    expected: true,
  },
  {
    name: "production SSR build",
    compiled: compiled(),
    options: { isProduction: true, isDev: false, ssr: true, extractCss: false, filePath: FILE },
    expected: false,
  },
  {
    name: "production, extraction on, no style block",
    compiled: compiled({ styles: [] }),
    options: { isProduction: true, isDev: false, extractCss: true, filePath: FILE },
    expected: false,
  },
  {
    name: "no css at all",
    compiled: compiled({ css: undefined, styles: [] }),
    options: { isProduction: false, isDev: true, extractCss: false, filePath: FILE },
    expected: false,
  },
] as const;

void test("embedsInlineCss matches the declared expectation for every branch", () => {
  assert.deepEqual(
    cases.map((testCase) => [testCase.name, embedsInlineCss(testCase.compiled, testCase.options)]),
    cases.map((testCase) => [testCase.name, testCase.expected]),
  );
});

void test("embedsInlineCss agrees with what generateOutput emits", () => {
  assert.deepEqual(
    cases.map((testCase) => [
      testCase.name,
      generateOutput(testCase.compiled, testCase.options).includes(CSS_MARKER),
    ]),
    cases.map((testCase) => [testCase.name, testCase.expected]),
  );
});

// Regression guard for over-skipping: a dev build must still embed the CSS, so
// `loadCompiledSfcModule` must still resolve its `@import`s there. If the
// predicate ever returned `false` for this case the plugin would ship an
// unresolved `@import` to the browser.
void test("a dev build still embeds plain CSS, so its @imports still need resolving", () => {
  const withImport = compiled({ css: `@import "./partial.css";\n${CSS_MARKER}` });
  const options = { isProduction: false, isDev: true, extractCss: false, filePath: FILE };

  assert.equal(embedsInlineCss(withImport, options), true);
  const output = generateOutput(withImport, options);
  assert.equal(
    output.includes(JSON.stringify(`@import "./partial.css";\n${CSS_MARKER}`)),
    true,
    "the dev module must carry the CSS verbatim so an unresolved @import would be visible",
  );
});
