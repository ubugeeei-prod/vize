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
    const options = buildCompileFileOptions("/src/App.vue", {
      sourceMap: true,
      ssr: false,
      vapor: true,
    });

    expect(options).toMatchSnapshot();
  });

  it("builds SSR single-file options while keeping vapor", () => {
    const options = buildCompileFileOptions("/src/App.vue", {
      sourceMap: false,
      ssr: true,
      vapor: true,
    });

    expect(options).toEqual({
      filename: "/src/App.vue",
      sourceMap: false,
      ssr: true,
      vapor: true,
      customRenderer: false,
      vueParserQuirks: false,
      scopeId: "data-v-c0cc6f12",
    });
  });

  it("builds batch options with vapor enabled", () => {
    expect(buildCompileBatchOptions({ ssr: false, vapor: true })).toEqual({
      ssr: false,
      vapor: true,
      customRenderer: false,
      vueParserQuirks: false,
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

    expect(result.code).toMatchSnapshot();
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

    expect(result.errors).toEqual([]);
    expect(result.warnings).toEqual([
      "SFC Vapor SSR is not supported yet; falling back to standard SSR output.",
    ]);
    expect(result.code).toMatchSnapshot();
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

    expect(result.code).toMatchSnapshot();
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

    expect(result.code).toMatchSnapshot();
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

    expect({
      highlight: highlightResult.code,
      monaco: monacoResult.code,
    }).toMatchSnapshot();
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

    expect(result.code).toMatchSnapshot();
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

    expect(result.code).toMatchSnapshot();
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

    expect(result.code).toMatchSnapshot();
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

    expect(result.code).toMatchSnapshot();
  });
});
