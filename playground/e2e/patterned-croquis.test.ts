import { beforeAll, describe, expect, it } from "vite-plus/test";
import { loadWasm, type WasmModule } from "../src/wasm";
import { featuresFor } from "../src/shared/experimentalFeatures";

let wasm: WasmModule;
beforeAll(async () => {
  wasm = await loadWasm();
});
const options = { experimentalPatternedTemplate: true };

describe("patterned template analysis in real WASM", () => {
  const source = `<script setup lang="ts">
// \u65e5\u672c\u8a9e \ud83c\udfa8
const result = { kind: 'ok', rows: [1] };
</script>
<template v-match="result">
  <template v-when="{ kind: &quot;ok&quot;, const rows, ...const rest } as whole if (rows.length &gt; 0)">
    <p v-for="rows in rows">{{ rows }}{{ rest }}{{ whole }}</p>
  </template>
  <p v-when="_">Empty</p>
</template>`;

  it("offers Croquis without falsely enabling Canon", () => {
    expect(featuresFor("croquis").some(({ key }) => key === "experimentalPatternedTemplate")).toBe(
      true,
    );
    expect(
      featuresFor("typechecker").some(({ key }) => key === "experimentalPatternedTemplate"),
    ).toBe(false);
  });

  it("retains root/arm scope nesting and UTF-16 authored ranges", () => {
    const result = wasm.analyzeSfc(source, options);
    expect(result.diagnostics).toEqual([]);
    const scopes = result.croquis.scopes;
    const match = scopes.find(({ kind }) => kind === "v-match")!;
    const arms = scopes.filter(({ kind }) => kind === "v-when");
    expect(match).toBeDefined();
    expect(match.isTemplateScope).toBe(true);
    expect(arms).toHaveLength(2);
    expect(arms[0].bindings.toSorted()).toEqual(["rest", "rows", "whole"]);
    expect(arms[1].bindings).toEqual([]);
    expect(arms.every(({ parentIds }) => parentIds?.[0] === match.id)).toBe(true);
    expect(source.slice(match.start, match.end)).toContain('v-match="result"');
    expect(source.slice(arms[0].start, arms[0].end)).toContain('v-for="rows in rows"');
    expect(source.slice(arms[0].start, arms[0].end)).not.toContain("Empty");
    expect(source.slice(arms[1].start, arms[1].end)).toContain("Empty");
    expect(result.vir).toContain("v-match");
    expect(result.vir).toContain("v-when");
    expect(() => wasm.analyzeSfc(source, {})).toThrow(/patternedTemplate/);
  });

  it("reports malformed patterns then recovers after an edit", () => {
    const broken = source.replace("...const rest", "const rows");
    const result = wasm.analyzeSfc(broken, options);
    expect(result.croquis.stats.error_count).toBe(1);
    expect(result.croquis.diagnostics).toEqual(result.diagnostics);
    const diagnostic = result.diagnostics[0];
    expect(diagnostic.severity).toBe("error");
    expect(diagnostic.message).toContain("Duplicate");
    expect(diagnostic.start).toBeGreaterThan(broken.indexOf("&quot;"));
    expect(diagnostic.end).toBeLessThanOrEqual(broken.indexOf('">', broken.indexOf("v-when")));
    expect(wasm.analyzeSfc(source, options).diagnostics).toEqual([]);
  });

  it("keeps missing direct arms as warnings with source ranges", () => {
    const invalid = '<template v-match="subject"><p>Not an arm</p></template>';
    const result = wasm.analyzeSfc(invalid, options);
    expect(result.croquis.stats.error_count).toBe(0);
    expect(result.croquis.stats.warning_count).toBe(2);
    expect(result.diagnostics.every(({ severity }) => severity === "warning")).toBe(true);
    expect(invalid.slice(result.diagnostics[0].start, result.diagnostics[0].end)).toBe("<p>");
  });
});
