import { describe, it, expect } from "vite-plus/test";
import { loadWasm, isWasmLoaded, isUsingMock, getWasm } from "../src/wasm/index";

describe("WASM Module", () => {
  it("should load WASM module", async () => {
    const wasm = await loadWasm();
    expect(wasm).toBeDefined();
    expect(isWasmLoaded()).toBe(true);
  });

  it("should return WASM module after loading", () => {
    const wasm = getWasm();
    expect(wasm).not.toBeNull();
  });

  it("should have compileSfc function", () => {
    const wasm = getWasm();
    expect(wasm).not.toBeNull();
    if (wasm) {
      expect(typeof wasm.compileSfc).toBe("function");
    }
  });

  it("should compile a simple SFC", async () => {
    const wasm = getWasm();
    expect(wasm).not.toBeNull();
    if (wasm) {
      const sfc = `
<template>
  <div>Hello</div>
</template>

<script setup>
const msg = 'Hello'
</script>
`;
      const result = wasm.compileSfc(sfc, {});
      expect(result).toBeDefined();
      expect(result.descriptor).toBeDefined();
    }
  });

  it("should use real WASM, not mock", () => {
    const usingMock = isUsingMock();
    console.log("Using mock:", usingMock);
    expect(usingMock).toBe(false);
  });

  it("should expose lint rules with preset membership", () => {
    const wasm = getWasm();
    expect(wasm).not.toBeNull();
    if (!wasm) {
      return;
    }

    const rules = wasm.getLintRules();
    expect(rules.length).toBeGreaterThan(0);

    const happyPathRule = rules.find((rule) => rule.name === "vue/require-v-for-key");
    expect(happyPathRule).toBeDefined();
    expect(happyPathRule?.presets).toContain("happy-path");
    expect(happyPathRule?.presets).toContain("opinionated");

    const opinionatedRule = rules.find((rule) => rule.name === "vue/no-inline-style");
    expect(opinionatedRule).toBeDefined();
    expect(opinionatedRule?.presets).toContain("opinionated");
    expect(opinionatedRule?.presets).not.toContain("happy-path");

    const scriptRule = rules.find((rule) => rule.name === "script/no-options-api");
    expect(scriptRule).toBeDefined();
    expect(scriptRule?.presets).toContain("opinionated");
    expect(scriptRule?.presets).toContain("nuxt");
    expect(scriptRule?.presets).not.toContain("happy-path");

    const noGetCurrentInstanceRule = rules.find(
      (rule) => rule.name === "script/no-get-current-instance",
    );
    expect(noGetCurrentInstanceRule).toBeDefined();
    expect(noGetCurrentInstanceRule?.presets).toContain("opinionated");
    expect(noGetCurrentInstanceRule?.presets).toContain("nuxt");
    expect(noGetCurrentInstanceRule?.presets).not.toContain("happy-path");

    const noNextTickRule = rules.find((rule) => rule.name === "script/no-next-tick");
    expect(noNextTickRule).toBeDefined();
    expect(noNextTickRule?.presets).toContain("opinionated");
    expect(noNextTickRule?.presets).toContain("nuxt");
    expect(noNextTickRule?.presets).not.toContain("happy-path");
  });

  it("should lint with different built-in presets", () => {
    const wasm = getWasm();
    expect(wasm).not.toBeNull();
    if (!wasm) {
      return;
    }

    const sfc = `
<template>
  <div style="color: red">hello</div>
</template>
`;

    const happyPath = wasm.lintSfc(sfc, {
      filename: "PresetExample.vue",
      preset: "happy-path",
    });
    const opinionated = wasm.lintSfc(sfc, {
      filename: "PresetExample.vue",
      preset: "opinionated",
    });

    expect(happyPath.diagnostics).toHaveLength(0);
    expect(
      opinionated.diagnostics.some((diagnostic) => diagnostic.rule === "vue/no-inline-style"),
    ).toBe(true);
    expect(opinionated.diagnostics.length).toBeGreaterThan(happyPath.diagnostics.length);
  });

  it("should report no-options-api for opinionated preset", () => {
    const wasm = getWasm();
    expect(wasm).not.toBeNull();
    if (!wasm) {
      return;
    }

    const sfc = `
<script>
export default {
  methods: {
    increment() {},
  },
}
</script>
`;

    const happyPath = wasm.lintSfc(sfc, {
      filename: "OptionsApi.vue",
      preset: "happy-path",
    });
    const opinionated = wasm.lintSfc(sfc, {
      filename: "OptionsApi.vue",
      preset: "opinionated",
    });

    expect(
      happyPath.diagnostics.some((diagnostic) => diagnostic.rule === "script/no-options-api"),
    ).toBe(false);
    expect(
      opinionated.diagnostics.some((diagnostic) => diagnostic.rule === "script/no-options-api"),
    ).toBe(true);
  });

  it("should report no-next-tick for opinionated preset", () => {
    const wasm = getWasm();
    expect(wasm).not.toBeNull();
    if (!wasm) {
      return;
    }

    const sfc = `
<script setup lang="ts">
import { nextTick } from "vue"

await nextTick()
</script>
`;

    const happyPath = wasm.lintSfc(sfc, {
      filename: "NextTick.vue",
      preset: "happy-path",
    });
    const opinionated = wasm.lintSfc(sfc, {
      filename: "NextTick.vue",
      preset: "opinionated",
    });

    expect(
      happyPath.diagnostics.some((diagnostic) => diagnostic.rule === "script/no-next-tick"),
    ).toBe(false);
    expect(
      opinionated.diagnostics.some((diagnostic) => diagnostic.rule === "script/no-next-tick"),
    ).toBe(true);
  });

  it("should report no-get-current-instance for opinionated preset", () => {
    const wasm = getWasm();
    expect(wasm).not.toBeNull();
    if (!wasm) {
      return;
    }

    const sfc = `
<script setup lang="ts">
import { getCurrentInstance } from "vue"

const instance = getCurrentInstance()
</script>
`;

    const happyPath = wasm.lintSfc(sfc, {
      filename: "GetCurrentInstance.vue",
      preset: "happy-path",
    });
    const opinionated = wasm.lintSfc(sfc, {
      filename: "GetCurrentInstance.vue",
      preset: "opinionated",
    });

    expect(
      happyPath.diagnostics.some(
        (diagnostic) => diagnostic.rule === "script/no-get-current-instance",
      ),
    ).toBe(false);
    expect(
      opinionated.diagnostics.some(
        (diagnostic) => diagnostic.rule === "script/no-get-current-instance",
      ),
    ).toBe(true);
  });
});
