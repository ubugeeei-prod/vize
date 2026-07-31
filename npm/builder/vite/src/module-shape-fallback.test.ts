/**
 * `generateOutput` prefers the module shape the native compiler reports over
 * re-parsing the emitted module with oxc (#3425). Both arms are pinned here,
 * because the fallback is permanent rather than transitional: a `.vpc` cache
 * entry written before the field existed reads back without it, and the rspack
 * and unplugin builders never set it at all.
 *
 * The two arms must produce byte-identical output. If they ever diverge, the
 * plugin's behaviour would depend on whether a cache happened to be warm, which
 * is the kind of defect that only shows up in someone else's build.
 */

import assert from "node:assert/strict";
import { test } from "node:test";

import { generateOutput } from "./utils/index.ts";
import { analyzeModuleOutput } from "./utils/module-output.ts";
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

/** The shapes the SFC compiler actually emits, one per `generateOutput` branch. */
const MODULES: Array<[string, string]> = [
  ["default export, no _sfc_main", "export default { name: 'App' }\n"],
  ["_sfc_main already defined", "const _sfc_main = { name: 'App' }\nexport default _sfc_main\n"],
  ["named render export only", "export function render(_ctx, _cache) { return null }\n"],
  ["named ssrRender export only", "export function ssrRender(_ctx, _push) {}\n"],
  ["neither", "export const useThing = () => 1\n"],
];

void test("the reported shape and the re-parsed shape produce identical output", () => {
  for (const [label, code] of MODULES) {
    const reparsed = generateOutput(compiled(code), OPTIONS);
    const reported = generateOutput(
      compiled(code, { moduleShape: analyzeModuleOutput(code) }),
      OPTIONS,
    );
    assert.equal(reported, reparsed, `${label}: using the reported shape must match re-parsing`);
  }
});

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

void test("an absent module shape falls back to parsing", () => {
  const code = "export default { name: 'App' }\n";
  const output = generateOutput(compiled(code), OPTIONS);
  assert.ok(
    output.includes('_sfc_main.__scopeId = "data-v-abc123"'),
    `the fallback should still find the default export, got:\n${output}`,
  );
});

console.log("✅ vite-plugin-vize module shape fallback tests passed!");
