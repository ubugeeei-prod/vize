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

    const generalRecommendedRule = rules.find((rule) => rule.name === "vue/require-v-for-key");
    expect(generalRecommendedRule).toBeDefined();
    expect(generalRecommendedRule?.presets).toContain("general-recommended");
    expect(generalRecommendedRule?.presets).toContain("opinionated");

    const opinionatedRule = rules.find((rule) => rule.name === "vue/no-inline-style");
    expect(opinionatedRule).toBeDefined();
    expect(opinionatedRule?.presets).toContain("opinionated");
    expect(opinionatedRule?.presets).not.toContain("general-recommended");

    const scriptRule = rules.find((rule) => rule.name === "script/no-options-api");
    expect(scriptRule).toBeDefined();
    expect(scriptRule?.presets).toContain("opinionated");
    expect(scriptRule?.presets).toContain("nuxt");
    expect(scriptRule?.presets).not.toContain("general-recommended");

    const noGetCurrentInstanceRule = rules.find(
      (rule) => rule.name === "script/no-get-current-instance",
    );
    expect(noGetCurrentInstanceRule).toBeDefined();
    expect(noGetCurrentInstanceRule?.presets).toContain("opinionated");
    expect(noGetCurrentInstanceRule?.presets).toContain("nuxt");
    expect(noGetCurrentInstanceRule?.presets).not.toContain("general-recommended");

    const noNextTickRule = rules.find((rule) => rule.name === "script/no-next-tick");
    expect(noNextTickRule).toBeDefined();
    expect(noNextTickRule?.presets).toContain("opinionated");
    expect(noNextTickRule?.presets).toContain("nuxt");
    expect(noNextTickRule?.presets).not.toContain("general-recommended");
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

    const generalRecommended = wasm.lintSfc(sfc, {
      filename: "PresetExample.vue",
      preset: "general-recommended",
    });
    const opinionated = wasm.lintSfc(sfc, {
      filename: "PresetExample.vue",
      preset: "opinionated",
    });

    expect(generalRecommended.diagnostics).toHaveLength(0);
    expect(
      opinionated.diagnostics.some((diagnostic) => diagnostic.rule === "vue/no-inline-style"),
    ).toBe(true);
    expect(opinionated.diagnostics.length).toBeGreaterThan(generalRecommended.diagnostics.length);
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

    const generalRecommended = wasm.lintSfc(sfc, {
      filename: "OptionsApi.vue",
      preset: "general-recommended",
    });
    const opinionated = wasm.lintSfc(sfc, {
      filename: "OptionsApi.vue",
      preset: "opinionated",
    });

    expect(
      generalRecommended.diagnostics.some(
        (diagnostic) => diagnostic.rule === "script/no-options-api",
      ),
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

    const generalRecommended = wasm.lintSfc(sfc, {
      filename: "NextTick.vue",
      preset: "general-recommended",
    });
    const opinionated = wasm.lintSfc(sfc, {
      filename: "NextTick.vue",
      preset: "opinionated",
    });

    expect(
      generalRecommended.diagnostics.some(
        (diagnostic) => diagnostic.rule === "script/no-next-tick",
      ),
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

    const generalRecommended = wasm.lintSfc(sfc, {
      filename: "GetCurrentInstance.vue",
      preset: "general-recommended",
    });
    const opinionated = wasm.lintSfc(sfc, {
      filename: "GetCurrentInstance.vue",
      preset: "opinionated",
    });

    expect(
      generalRecommended.diagnostics.some(
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
