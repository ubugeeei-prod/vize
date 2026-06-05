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

    const opinionatedRule = rules.find((rule) => rule.name === "vue/no-inline-style");
    expect(opinionatedRule).toBeDefined();

    const scriptRule = rules.find((rule) => rule.name === "script/no-options-api");
    expect(scriptRule).toBeDefined();

    const noGetCurrentInstanceRule = rules.find(
      (rule) => rule.name === "script/no-get-current-instance",
    );
    expect(noGetCurrentInstanceRule).toBeDefined();

    const noNextTickRule = rules.find((rule) => rule.name === "script/no-next-tick");
    expect(noNextTickRule).toBeDefined();
    expect({
      noGetCurrentInstance: noGetCurrentInstanceRule?.presets,
      noInlineStyle: opinionatedRule?.presets,
      noNextTick: noNextTickRule?.presets,
      noOptionsApi: scriptRule?.presets,
      requireVForKey: generalRecommendedRule?.presets,
    }).toMatchSnapshot();
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

  it("should honor cross-file analyzer toggles and imported component aliases", () => {
    const wasm = getWasm();
    expect(wasm).not.toBeNull();
    if (!wasm) {
      return;
    }

    const files = [
      {
        path: "Parent.vue",
        source: `
<script setup lang="ts">
import Panel from './Child.vue'
</script>

<template>
  <Panel />
</template>
`,
      },
      {
        path: "Child.vue",
        source: `
<script setup lang="ts">
defineProps<{
  title: string
}>()
</script>

<template>
  <section>{{ title }}</section>
</template>
`,
      },
    ];

    const relationshipOnly = wasm.analyzeCrossFile(files, {
      componentResolution: true,
      propsValidation: false,
    });
    expect(
      relationshipOnly.diagnostics.some((diagnostic) => diagnostic.type === "props-validation"),
    ).toBe(false);

    const validation = wasm.analyzeCrossFile(files, {
      componentResolution: true,
      propsValidation: true,
    });
    expect(
      validation.diagnostics.some(
        (diagnostic) =>
          diagnostic.type === "props-validation" &&
          diagnostic.code === "vize:croquis/cf/missing-required-prop",
      ),
    ).toBe(true);
    expect(
      validation.diagnostics.some((diagnostic) => diagnostic.type === "component-resolution"),
    ).toBe(false);
  });

  it("should report strict provide/inject and reactivity loss severities", () => {
    const wasm = getWasm();
    expect(wasm).not.toBeNull();
    if (!wasm) {
      return;
    }

    const files = [
      {
        path: "Parent.vue",
        source: `
<script setup lang="ts">
import { provide, reactive } from 'vue'
import Child from './Child.vue'

provide('config', { debug: true })
provide('state', reactive({ count: 0 }))
</script>

<template>
  <Child />
</template>
`,
      },
      {
        path: "Child.vue",
        source: `
<script setup lang="ts">
import { inject } from 'vue'

const useInject = inject
const config = inject('config')
const { count } = inject('state') as { count: number }
const [first] = useInject('items', [1])
const theme = inject('theme', 'light')
</script>

<template>
  <p>{{ config }} {{ count }} {{ first }} {{ theme }}</p>
</template>
`,
      },
    ];

    const result = wasm.analyzeCrossFile(files, {
      provideInject: true,
      reactivityTracking: true,
    });

    const destructuring = result.diagnostics.filter(
      (diagnostic) => diagnostic.code === "vize:croquis/cf/destructuring-breaks-reactivity",
    );
    expect(destructuring.length).toBe(2);
    expect(destructuring.every((diagnostic) => diagnostic.severity === "error")).toBe(true);
    expect(destructuring.every((diagnostic) => diagnostic.type === "reactivity-loss")).toBe(true);
    const stateDestructure = destructuring.find((diagnostic) =>
      diagnostic.message.includes("state"),
    );
    expect(stateDestructure?.relatedLocations?.[0]).toMatchObject({
      file: "Parent.vue",
      message: "provide('state') source",
    });

    const uniqueDiagnosticKeys = new Set(
      result.diagnostics.map(
        (diagnostic) => `${diagnostic.code}:${diagnostic.file}:${diagnostic.offset}`,
      ),
    );
    expect(uniqueDiagnosticKeys.size).toBe(result.diagnostics.length);

    const fileOrder = new Map(files.map((file, index) => [file.path, index]));
    const severityOrder: Record<string, number> = { error: 0, warning: 1, info: 2, hint: 3 };
    const diagnosticOrder = result.diagnostics.map((diagnostic) => ({
      code: diagnostic.code,
      fileIndex: fileOrder.get(diagnostic.file) ?? Number.MAX_SAFE_INTEGER,
      offset: diagnostic.offset,
      severity: severityOrder[diagnostic.severity] ?? Number.MAX_SAFE_INTEGER,
    }));
    const sortedDiagnosticOrder = [...diagnosticOrder].sort(
      (left, right) =>
        left.fileIndex - right.fileIndex ||
        left.offset - right.offset ||
        left.severity - right.severity ||
        left.code.localeCompare(right.code),
    );
    expect(diagnosticOrder).toEqual(sortedDiagnosticOrder);

    const defaultedInject = result.diagnostics.find(
      (diagnostic) =>
        diagnostic.code === "vize:croquis/cf/unmatched-inject" &&
        diagnostic.message.includes("theme"),
    );
    expect(defaultedInject?.severity).toBe("warning");
    expect(defaultedInject?.type).toBe("provide-inject");

    const nonReactiveProvide = result.diagnostics.find(
      (diagnostic) => diagnostic.code === "vize:croquis/cf/non-reactive-provide",
    );
    expect(nonReactiveProvide?.severity).toBe("warning");
    expect(nonReactiveProvide?.type).toBe("provide-inject");

    const stringProvideKeys = result.diagnostics.filter(
      (diagnostic) => diagnostic.code === "vize:croquis/cf/provide-without-symbol",
    );
    const stringInjectKeys = result.diagnostics.filter(
      (diagnostic) => diagnostic.code === "vize:croquis/cf/inject-without-symbol",
    );
    expect(stringProvideKeys.length).toBeGreaterThanOrEqual(2);
    expect(stringInjectKeys.length).toBeGreaterThanOrEqual(4);
    expect(
      [...stringProvideKeys, ...stringInjectKeys].every(
        (diagnostic) => diagnostic.severity === "warning" && diagnostic.type === "provide-inject",
      ),
    ).toBe(true);
  });
});
