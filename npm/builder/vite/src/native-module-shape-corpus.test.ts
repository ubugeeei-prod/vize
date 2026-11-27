// The plugin assembles every component from the native `moduleShape` and no
// longer parses emitted modules with oxc. Over the compiler differential corpus
// and the JSX scripts the emitter leaves for the bundler, the native compiler
// must report a shape for every module it compiles, and that shape must be the
// one the oxc analysis it replaced derived from the same code.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { analyzeModuleOutput } from "../test/oxc-module-output.ts";
import { compileFile } from "./compiler.ts";
import type { CompiledModule } from "./types.ts";

const CORPUS = new URL("../../../../tests/_fixtures/differential/compiler/", import.meta.url);

const JSX_SCRIPTS: Record<string, string> = {
  "tsx-setup.vue": `<script setup lang="tsx">
import { ref } from "vue";
const n = ref<number>(2);
const Render = () => <span class="tsx">{n.value}</span>;
</script>
<template><div><Render /></div></template>
`,
  "jsx-setup.vue": `<script setup lang="jsx">
const label = "jsx";
const Render = () => <b>{label}</b>;
</script>
<template><Render /></template>
`,
  "tsx-options.vue": `<script lang="tsx">
export default { render() { return <p>{1 as number}</p> } }
</script>
<style scoped>p{}</style>
`,
};

function corpus(): [string, string][] {
  const inputs: [string, string][] = [];
  const pending = [fileURLToPath(CORPUS)];
  while (pending.length > 0) {
    const dir = pending.pop()!;
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const file = path.join(dir, entry.name);
      if (entry.isDirectory()) pending.push(file);
      else if (/\.vue(?:\.txt)?$/.test(entry.name)) {
        inputs.push([entry.name.replace(/\.txt$/, ""), fs.readFileSync(file, "utf8")]);
      }
    }
  }
  return inputs;
}

/** The fields the output assembly reads; the scan reports no statement end. */
function geometry(shape: NonNullable<CompiledModule["moduleShape"]>, withEnd: boolean) {
  return {
    hasDefaultExport: shape.hasDefaultExport,
    hasSfcMainDefined: shape.hasSfcMainDefined,
    hasNamedRenderExport: shape.hasNamedRenderExport,
    hasNamedSsrRenderExport: shape.hasNamedSsrRenderExport,
    defaultExportStart: shape.defaultExportStart ?? null,
    defaultExportKeywordEnd: shape.defaultExportKeywordEnd ?? null,
    defaultExportIsSfcMain: shape.defaultExportIsSfcMain,
    ...(withEnd ? { defaultExportEnd: shape.defaultExportEnd ?? null } : {}),
  };
}

void test("the native module shape is reported and equals the oxc analysis", () => {
  const inputs = [...corpus(), ...Object.entries(JSX_SCRIPTS)];
  let compared = 0;
  let jsx = 0;
  for (const [name, source] of inputs) {
    for (const ssr of [false, true]) {
      const label = `${name}${ssr ? " (ssr)" : ""}`;
      let compiled: CompiledModule;
      try {
        compiled = compileFile(
          `/src/${name}`,
          new Map(),
          { sourceMap: false, ssr, vapor: false },
          source,
          { logWarnings: false },
        );
      } catch {
        // Corpus inputs that pin compile errors produce no module.
        continue;
      }
      assert.ok(compiled.moduleShape, `${label}: the native compiler reports the shape`);
      const expected = analyzeModuleOutput(compiled.code);
      const withEnd = expected.defaultExportEnd !== null;
      assert.deepEqual(
        geometry(compiled.moduleShape, withEnd),
        geometry(expected, withEnd),
        `${label}: native shape`,
      );
      compared++;
      if (/lang="[jt]sx"/.test(source)) jsx++;
    }
  }
  assert.ok(compared >= 500, `compared ${compared} modules`);
  assert.ok(jsx >= 8, `compared ${jsx} JSX modules`);
});
