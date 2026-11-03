// #11583: the default export is bound at the native shape's UTF-8 byte span
// through the shared `bindDefaultExport`, so text before or after it that is
// not ASCII cannot shift the rewrite. Shapes here come from the real native
// compiler; the oxc fallback must report the same bytes.
import assert from "node:assert/strict";
import { compileSfc } from "@vizejs/native";
import { parseSync } from "oxc-parser";
import { buildCompileFileOptions } from "./compile-options.ts";
import { compileFile } from "./compiler.ts";
import type { CompiledModule } from "./types.ts";
import { generateOutput, generateScopeId } from "./utils/index.ts";
import { analyzeModuleOutput, type ModuleOutputInfo } from "./utils/module-output.ts";

const TEXT = {
  cjk: "日本語",
  astral: "🐱𝒜",
  combining: "é café",
};
const OPTIONS = { isProduction: false, isDev: false, filePath: "/src/A.vue" };

/** `compileFile` output plus the shape the same native compile reported. */
function compile(
  source: string,
  ssr: boolean,
): { compiled: CompiledModule; shape: ModuleOutputInfo } {
  const options = { sourceMap: false, ssr, vapor: false };
  const compiled = compileFile("/src/A.vue", new Map(), options, source);
  const scopeId = generateScopeId("/src/A.vue", undefined, false, source);
  const result = compileSfc(source, buildCompileFileOptions("/src/A.vue", options, scopeId));
  assert.equal(result.code, compiled.code, "the shape describes compileFile's code");
  const shape = result.moduleShape;
  assert.ok(shape, "the native compiler reports a shape");
  // #11544: `compileFile` hands that shape through to the output assembly.
  assert.deepEqual(compiled.moduleShape, shape, "compileFile carries the native shape");
  return {
    compiled,
    shape: {
      ...shape,
      defaultExportStart: shape.defaultExportStart ?? null,
      defaultExportKeywordEnd: shape.defaultExportKeywordEnd ?? null,
      defaultExportEnd: shape.defaultExportEnd ?? null,
    },
  };
}

/** The fallback's fast path omits the statement end, which no caller reads. */
function span({ defaultExportEnd: _, ...shape }: ModuleOutputInfo) {
  return shape;
}

let cases = 0;
for (const [name, text] of Object.entries(TEXT)) {
  const sources = {
    // `<script setup>` binds `export default {` with the text before it.
    before: `<script setup>const s = "${text}"</script><template><p>{{ s }} ${text}</p></template><style scoped>p{}</style>`,
    // Options API without a template keeps the authored export, text after it.
    after: `<script>export default { name: "${text}", render() { return "${text}" } }</script><style scoped>p{}</style>`,
    both: `<script>const a = "${text}"; export default { name: "${text}", render() { return a } }</script><style scoped>p{}</style>`,
  };
  for (const [position, source] of Object.entries(sources)) {
    for (const ssr of [false, true]) {
      const label = `${name}/${position}${ssr ? "/ssr" : ""}`;
      const { compiled, shape } = compile(source, ssr);
      assert.equal(shape.hasDefaultExport && !shape.hasSfcMainDefined, true, label);
      assert.deepEqual(span(analyzeModuleOutput(compiled.code)), span(shape), `${label}: bytes`);
      const output = generateOutput({ ...compiled, moduleShape: shape }, { ...OPTIONS, ssr });
      assert.equal(generateOutput(compiled, { ...OPTIONS, ssr }), output, `${label}: same output`);
      assert.deepEqual(parseSync("A.js", output).errors, [], `${label}: output parses`);
      assert.match(output, /const _sfc_main = \{/, label);
      assert.match(output, /\nexport default _sfc_main;/, label);
      assert.match(output, /_sfc_main\.__scopeId = "data-v-/, label);
      assert.ok(output.includes(text), `${label}: authored text survives`);
      cases++;
    }
  }
}
assert.equal(cases, 18);

// Negative control: UTF-16 indices are not the native contract and must throw.
{
  const source = `<script setup>const s = "${TEXT.cjk}"</script><template><p>{{ s }}</p></template>`;
  const { compiled, shape } = compile(source, false);
  const start = compiled.code.indexOf("export default");
  const utf16 = { ...shape, defaultExportStart: start, defaultExportKeywordEnd: start + 14 };
  assert.throws(
    () => generateOutput({ ...compiled, moduleShape: utf16 }, OPTIONS),
    /native component export geometry/i,
  );
}

// `_sfc_main` already exported: `__scopeId` is appended after the module.
{
  const source = `<script>export default { name: "${TEXT.astral}" }</script><template><p /></template><style scoped>p{}</style>`;
  const { compiled, shape } = compile(source, false);
  assert.equal(shape.defaultExportIsSfcMain, true);
  const output = generateOutput({ ...compiled, moduleShape: shape }, OPTIONS);
  const exported = output.indexOf("export default _sfc_main");
  assert.ok(exported >= 0);
  assert.ok(output.indexOf("_sfc_main.__scopeId") > exported, "appended after the export");
}

console.log("✅ vite-plugin-vize native shape binding tests passed!");
