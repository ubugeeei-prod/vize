/**
 * `generateOutput` assembles the component from the module shape the native
 * compiler reports, and never parses the emitted module itself. The compiler
 * reports no shape only for a module that parses as neither TypeScript nor
 * TSX; such a module is bound only at an unambiguous scanned `export default`,
 * and otherwise left alone.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import { generateOutput } from "./utils/index.ts";
import { unparsedModuleOutput } from "./utils/module-output.ts";
import { analyzeModuleOutput } from "../test/oxc-module-output.ts";
import type { CompiledModule } from "./types.ts";

const OPTIONS = {
  isProduction: false,
  isDev: true,
  ssr: false,
  hmrUpdateType: null,
  extractCss: false,
  filePath: "/src/App.vue",
} as const;

function compiled(code: string, overrides: Partial<CompiledModule> = {}): CompiledModule {
  return {
    code,
    css: null,
    scopeId: "abc123",
    hasScoped: true,
    styles: [],
    ...overrides,
  } as CompiledModule;
}

void test("a module shape reported by the compiler is used as-is", () => {
  const code = "const _sfc_main = { name: 'App' }\nexport default _sfc_main\n";
  // A shape that disagrees with the code proves the field is consulted rather
  // than the module being re-parsed: `hasSfcMainDefined: false` sends
  // `generateOutput` down the rewrite branch, which the real shape would not.
  const lying = { ...analyzeModuleOutput(code), hasSfcMainDefined: false };
  const output = generateOutput(compiled(code, { moduleShape: lying }), OPTIONS);
  assert.ok(
    output.includes("export default _sfc_main;"),
    `the reported shape should have driven the rewrite branch, got:\n${output}`,
  );
});

void test("a shape-less module binds a lone scanned default export", () => {
  const code = "export default { name: 'App' }\n";
  const output = generateOutput(compiled(code), OPTIONS);
  assert.deepEqual(unparsedModuleOutput(code), analyzeModuleOutput(code));
  assert.ok(
    output.includes('_sfc_main.__scopeId = "data-v-abc123"'),
    `the scan should still find the default export, got:\n${output}`,
  );
});

void test("an unparseable shape-less module the scan cannot read is left alone", () => {
  // Unparseable, as every shape-less module is: no scan can tell what it exports.
  for (const code of [
    "const _sfc_main = { name: 'App' }\nexport default _sfc_main\n) (\n",
    "export function render(_ctx, _cache) { return null }\n) (\n",
    "export { x as default }\n) (\n",
  ]) {
    assert.equal(generateOutput(compiled(code), OPTIONS), code);
    // The oxc analysis this replaced could not parse them either.
    assert.deepEqual(unparsedModuleOutput(code), analyzeModuleOutput(code));
  }
});

console.log("✅ vite-plugin-vize module shape fallback tests passed!");
