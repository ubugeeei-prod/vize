import { afterEach, beforeAll, describe, expect, it } from "vite-plus/test";
import { ref } from "vue";
import "../src/monacoBootstrap";
import { loadWasm, type WasmModule } from "../src/wasm";
import { useMonacoTypeCheck } from "../src/features/canon/useMonacoTypeCheck";
import type { ExperimentalOptions } from "../src/shared/experimentalFeatures";

let wasm: WasmModule;
let check: ReturnType<typeof useMonacoTypeCheck> | null = null;
beforeAll(async () => {
  wasm = await loadWasm();
});
afterEach(() => {
  check?.dispose();
  check = null;
});

const valid = `<script setup lang="ts">
// \u65e5\u672c\u8a9e \ud83c\udfa8
type Result = { kind: 'ok'; rows: number[] } | { kind: 'err'; message: string };
const result = {} as Result;
</script>
<template v-match="result">
  <template v-when="{ kind: &quot;ok&quot;, const rows } as whole if (rows.length &gt; 0)">
    <p v-for="rows in rows">{{ rows.toFixed() }} {{ whole.rows.length }} {{ result.rows.length }}</p>
  </template>
  <p v-when="_">Fallback</p>
</template>`;

async function create(sourceText = valid) {
  const source = ref(sourceText);
  const experimentals = ref<ExperimentalOptions>({ experimentalPatternedTemplate: true });
  check = useMonacoTypeCheck({
    source,
    experimentals,
    compiler: () => wasm,
    strictMode: ref(true),
    checkProps: ref(true),
    checkEmits: ref(true),
    checkTemplateBindings: ref(true),
    useMonacoTs: ref(true),
  });
  expect(await check.configureTypeScript()).toBe(true);
  await check.typeCheck();
  expect(check.error.value).toBeNull();
  return { source, experimentals, check };
}

describe("patterned Canon with real WASM and Monaco TypeScript", () => {
  it("checks root arms, as/loop bindings, guards and the original subject", async () => {
    const { source, check } = await create();
    expect(check.diagnostics.value).toEqual([]);
    expect(check.typeCheckResult.value?.virtualTsHelpers).toContain("namespace __VizePatterns");
    source.value = valid.replace("rows.toFixed()", "rows.toUpperCase()");
    await check.typeCheck();
    expect(check.diagnostics.value).toHaveLength(1);
    expect(check.diagnostics.value[0].code).toBe(2339);
    expect(check.diagnostics.value[0].startLine).toBe(8);
    source.value = valid;
    await check.typeCheck();
    expect(check.diagnostics.value).toEqual([]);
  });

  it("reports missing coverage on the authored root header", async () => {
    const { check } = await create(valid.replace('  <p v-when="_">Fallback</p>', ""));
    expect(check.diagnostics.value).toHaveLength(1);
    expect(check.diagnostics.value[0]).toMatchObject({
      severity: "error",
      code: 2322,
      startLine: 6,
      startColumn: 20,
      endColumn: 26,
    });
    expect(check.diagnostics.value[0].message).toContain("Non-exhaustive v-match");
    expect(check.diagnostics.value[0].help).toContain("Guards do not prove coverage");
    expect(check.diagnostics.value[0].help).not.toContain("Type assertion");
  });

  it("keeps unreachable arms warning-only", async () => {
    const { check } = await create(`<script setup lang="ts">const value = 'a' as 'a' | 'b';</script>
<template v-match="value"><p v-when="'a'"/><p v-when="'a'"/><p v-when="'b'"/></template>`);
    expect(check.diagnostics.value).toHaveLength(1);
    expect(check.diagnostics.value[0].severity).toBe("warning");
    expect(check.diagnostics.value[0].message).toContain("Unreachable v-when");
    expect(check.diagnostics.value[0].help).toContain("check its pattern and order");
    expect(check.errorCount.value).toBe(0);
    expect(check.warningCount.value).toBe(1);
  });

  it("reports malformed source in UTF-16 and clears helpers when disabled", async () => {
    const { source, experimentals, check } = await create();
    source.value = valid.replace("const rows }", "const rows, const rows }");
    await check.typeCheck();
    const diagnostic = check.typeCheckResult.value!.diagnostics.find((d) =>
      d.message.includes("Duplicate"),
    )!;
    const duplicate = "const rows, const rows";
    const expectedStart = source.value.indexOf(duplicate) + duplicate.length;
    const expectedEnd = source.value.indexOf('">', expectedStart);
    expect(diagnostic.start).toBe(expectedStart);
    expect(diagnostic.end).toBe(expectedEnd);
    source.value = valid;
    experimentals.value = { experimentalPatternedTemplate: false };
    await check.typeCheck();
    expect(check.diagnostics.value.some((d) => d.message.includes("patternedTemplate"))).toBe(true);
    expect(check.typeCheckResult.value?.virtualTsHelpers).toBeUndefined();
    experimentals.value = { experimentalPatternedTemplate: true };
    await check.typeCheck();
    expect(check.diagnostics.value).toEqual([]);
  });
});
