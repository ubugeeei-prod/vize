import { readFileSync } from "node:fs";
import { describe, expect, it } from "vite-plus/test";
import native from "../../npm/vize-native/index.js";

import {
  buildCompileBatchOptions,
  buildCompileFileOptions,
} from "../../npm/vite-plugin-vize/src/compile-options.ts";

const { compileSfc } = native;

describe("vite-plugin vapor options", () => {
  it("builds scoped single-file options with vapor enabled", () => {
    const options = buildCompileFileOptions(
      "/src/App.vue",
      "<template><div /></template><style scoped>.root { color: red; }</style>",
      { sourceMap: true, ssr: false, vapor: true },
    );

    expect(options).toMatchObject({
      filename: "/src/App.vue",
      sourceMap: true,
      ssr: false,
      vapor: true,
    });
    expect(options.scopeId).toMatch(/^data-v-/);
  });

  it("omits scopeId for unscoped styles while keeping vapor", () => {
    const options = buildCompileFileOptions(
      "/src/App.vue",
      "<template><div /></template><style>.root { color: red; }</style>",
      { sourceMap: false, ssr: true, vapor: true },
    );

    expect(options).toEqual({
      filename: "/src/App.vue",
      sourceMap: false,
      ssr: true,
      vapor: true,
      customRenderer: false,
      scopeId: undefined,
    });
  });

  it("builds batch options with vapor enabled", () => {
    expect(buildCompileBatchOptions({ ssr: false, vapor: true })).toEqual({
      ssr: false,
      vapor: true,
      customRenderer: false,
    });
  });

  it("compiles script setup SFCs to a full Vapor render block", () => {
    const result = compileSfc(
      `<script setup lang="ts">
import { ref } from "vue";

const count = ref(1);
</script>

<template>
  <div>{{ count }}</div>
</template>`,
      {
        filename: "/src/App.vue",
        sourceMap: false,
        ssr: false,
        vapor: true,
        isTs: true,
      },
    );

    expect(result.code).toContain("_defineVaporComponent");
    expect(result.code).toContain("const n0 = t0()");
    expect(result.code).toContain("const __ctx = _proxyRefs(__returned__)");
    expect(result.code).toContain("const __vaporRender = render");
    expect(result.code).toContain("return __vaporRender(__ctx, __props, __emit, __attrs, __slots)");
    expect(result.code).toContain("return n0");
  });

  it("warns and falls back to standard SSR when Vapor is requested for SFCs", () => {
    const result = compileSfc(
      `<script setup lang="ts">
const count = 1;
</script>

<template>
  <div>{{ count }}</div>
</template>`,
      {
        filename: "/src/SsrFallback.vue",
        sourceMap: false,
        ssr: true,
        vapor: true,
        isTs: true,
      },
    );

    expect(result.code).toContain("ssrRender");
    expect(result.code).not.toContain("__vapor");
    expect(result.errors).toEqual([]);
    expect(result.warnings).toEqual([
      "SFC Vapor SSR is not supported yet; falling back to standard SSR output.",
    ]);
  });

  it("compiles the playground app itself to Vapor output", () => {
    const source = readFileSync(new URL("../src/App.vue", import.meta.url), "utf8");
    const result = compileSfc(source, {
      filename: "/src/App.vue",
      sourceMap: false,
      ssr: false,
      vapor: true,
      isTs: true,
    });

    expect(result.code).toContain("_defineVaporComponent");
    expect(result.code).toContain("_createComponentWithFallback");
    expect(result.code).toContain("_createIf");
    expect(result.code).toContain("_setClass");
    expect(result.code).toContain('_delegateEvents("click")');
    expect(result.code).toContain("_setInsertionState(n16, null, true)\n  const n17 = _createIf(");
    expect(result.code).toContain("_setInsertionState(n1, null, true)\n  const n22 = _createIf(");
    expect(result.code).toContain(
      '<g transform=\\"translate(15, 10) skewX(-15)\\"><path d=\\"M 65 0 L 40 60 L 70 20 L 65 0 Z\\" fill=\\"currentColor\\"></path><path d=\\"M 20 0 L 40 60 L 53 13 L 20 0 Z\\" fill=\\"currentColor\\"></path></g>',
    );
    expect(result.code).toContain("const __vaporRender = render");
    expect(result.code).toContain("return __vaporRender(__ctx, __props, __emit, __attrs, __slots)");
    expect(result.code).not.toContain("_openBlock");
    expect(result.code).not.toContain("_createElementBlock");
    expect(result.code).not.toContain("_ctx.===");
  });

  it("avoids collisions with local render bindings in script setup", () => {
    const result = compileSfc(
      `<script setup lang="ts">
function render() {
  return "local";
}
</script>

<template>
  <div>Hello</div>
</template>`,
      {
        filename: "/src/Collision.vue",
        sourceMap: false,
        ssr: false,
        vapor: true,
        isTs: true,
      },
    );

    expect(result.code).toContain("const __vaporRender = render");
    expect(result.code).toContain("render: __vaporRender");
    expect(result.code).toContain("return __vaporRender(__ctx, __props, __emit, __attrs, __slots)");
  });

  it("emits Vapor template ref setters for DOM refs used by playground components", () => {
    const monacoSource = readFileSync(
      new URL("../src/shared/MonacoEditor.vue", import.meta.url),
      "utf8",
    );
    const monacoResult = compileSfc(monacoSource, {
      filename: "/src/shared/MonacoEditor.vue",
      sourceMap: false,
      ssr: false,
      vapor: true,
      isTs: true,
    });
    const highlightSource = readFileSync(
      new URL("../src/shared/CodeHighlight.vue", import.meta.url),
      "utf8",
    );
    const highlightResult = compileSfc(highlightSource, {
      filename: "/src/shared/CodeHighlight.vue",
      sourceMap: false,
      ssr: false,
      vapor: true,
      isTs: true,
    });

    expect(monacoResult.code).toContain("createTemplateRefSetter as _createTemplateRefSetter");
    expect(monacoResult.code).toContain(
      "const vaporTemplateRefSetter = _createTemplateRefSetter()",
    );
    expect(monacoResult.code).toContain(
      "const _setRef = _ctx.vaporTemplateRefSetter || _createTemplateRefSetter()",
    );
    expect(monacoResult.code).toContain('_setRef(n0, "containerRef")');
    expect(monacoResult.code).not.toContain('ref="containerRef"');
    expect(highlightResult.code).toContain('"codeContentEl"');
    expect(highlightResult.code).toContain('"lineNumbersEl"');
    expect(highlightResult.code).not.toContain('ref="codeContentEl"');
  });

  it("keeps v-for aliases inside Vapor v-html expressions", () => {
    const source = readFileSync(
      new URL("../src/features/patina/PatinaPlayground.vue", import.meta.url),
      "utf8",
    );
    const result = compileSfc(source, {
      filename: "/src/features/patina/PatinaPlayground.vue",
      sourceMap: false,
      ssr: false,
      vapor: true,
      isTs: true,
    });

    expect(result.code).toContain("_ctx.formatHelp(_for_item0.value.help)");
    expect(result.code).not.toContain("formatHelp(diagnostic.help)");
  });

  it("treats lowercase imported Tres-style components as Vapor components", () => {
    const result = compileSfc(
      `<script setup lang="ts">
import { Primitive } from "@tresjs/core";
</script>

<template>
  <primitive />
</template>`,
      {
        filename: "/src/TresPrimitive.vue",
        sourceMap: false,
        ssr: false,
        vapor: true,
        isTs: true,
      },
    );

    expect(result.code).toContain("const _component_primitive = _ctx.Primitive");
    expect(result.code).toContain("_createComponentWithFallback(_component_primitive");
    expect(result.code).not.toContain('_template("<primitive></primitive>", true)');
  });

  it("keeps Tres-style custom renderer intrinsics as elements around imported lowercase components", () => {
    const result = compileSfc(
      `<script setup lang="ts">
import { Primitive } from "@tresjs/core";
const visible = true;
</script>

<template>
  <mesh>
    <group v-if="visible">
      <primitive />
    </group>
  </mesh>
</template>`,
      {
        filename: "/src/TresCustomRenderer.vue",
        sourceMap: false,
        ssr: false,
        vapor: true,
        customRenderer: true,
        isTs: true,
      },
    );

    expect(result.code).toContain('const t1 = _template("<mesh></mesh>", true)');
    expect(result.code).toContain('const t0 = _template("<group></group>", true)');
    expect(result.code).toContain("const _component_primitive = _ctx.Primitive");
    expect(result.code).not.toContain('_resolveComponent("mesh")');
    expect(result.code).not.toContain('_resolveComponent("group")');
  });

  it("keeps Atelier output tabs reactive even when v-if siblings are present", () => {
    const source = readFileSync(
      new URL("../src/features/atelier/AtelierPlayground.vue", import.meta.url),
      "utf8",
    );
    const result = compileSfc(source, {
      filename: "/src/features/atelier/AtelierPlayground.vue",
      sourceMap: false,
      ssr: false,
      vapor: true,
      isTs: true,
    });

    expect(result.code).toContain("_createInvoker(() => (_ctx.activeTab = 'code'))");
    expect(result.code).toContain("_createInvoker(() => (_ctx.activeTab = 'ast'))");
    expect(result.code).toContain("_createInvoker(() => (_ctx.activeTab = 'helpers'))");
    expect(result.code).toContain("_ctx.activeTab === 'code'");
    expect(result.code).toContain("_ctx.activeTab === 'ast'");
    expect(result.code).toContain("_ctx.activeTab === 'helpers'");
    expect(result.code).toContain("_createIf(() => (_ctx.inputMode === 'sfc')");
  });
});
