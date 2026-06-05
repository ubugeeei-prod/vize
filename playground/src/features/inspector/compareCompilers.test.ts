import { describe, expect, it, vi } from "vite-plus/test";
import { compileInspectorReport } from "./compareCompilers";
import type { CompilerOptions, WasmModule } from "../../wasm/index";

vi.mock("../atelier/formatters", () => ({
  formatCode: vi.fn(async (code: string, parser: string) => `[${parser}]\n${code}`),
}));

describe("compileInspectorReport", () => {
  it("includes virtual ts, vir, and graph inspector outputs", async () => {
    const compileSfc = vi.fn((_: string, options: CompilerOptions) => ({
      descriptor: {
        filename: "src/App.vue",
        source: "",
        template: {
          content: "{{ msg }}",
          loc: { start: 0, end: 0 },
          attrs: {},
        },
        script: undefined,
        scriptSetup: { content: "", loc: { start: 0, end: 0 }, attrs: {}, setup: true },
        styles: [],
        customBlocks: [],
      },
      script: { code: options.ssr ? "export const ssr = true;" : "export const dom = true;" },
      warnings: [],
    }));
    const typeCheck = vi.fn(() => ({
      diagnostics: [
        {
          severity: "warning" as const,
          message: "virtual note",
          start: 0,
          end: 0,
          code: "vts-note",
          related: [],
        },
      ],
      virtualTs: "const __vts = 1;",
      errorCount: 0,
      warningCount: 1,
    }));
    const analyzeSfc = vi.fn(() => ({
      croquis: {
        is_setup: true,
        bindings: [],
        scopes: [],
        macros: [],
        props: [],
        emits: [],
        provides: [],
        injects: [],
        typeExports: [],
        invalidExports: [],
        diagnostics: [],
        stats: {
          binding_count: 0,
          unused_binding_count: 0,
          scope_count: 0,
          macro_count: 0,
          type_export_count: 0,
          invalid_export_count: 0,
          error_count: 0,
          warning_count: 0,
        },
      },
      diagnostics: [],
      vir: "[vir]\nbindings=0\n",
    }));
    const analyzeCrossFile = vi.fn(() => ({
      diagnostics: [],
      circularDependencies: [],
      stats: {
        filesAnalyzed: 2,
        vueComponents: 2,
        dependencyEdges: 1,
        errorCount: 0,
        warningCount: 0,
        infoCount: 0,
        analysisTimeMs: 0,
      },
      filePaths: ["src/App.vue", "src/Child.vue"],
    }));
    const buildInspectorGraph = vi.fn(() => ({
      nodes: [
        {
          path: "src/App.vue",
          kind: "vue",
          isEntry: true,
          sourceLines: 1,
          sourceBytes: 110,
        },
        {
          path: "src/Child.vue",
          kind: "vue",
          isEntry: false,
          sourceLines: 1,
          sourceBytes: 28,
        },
      ],
      edges: [
        {
          from: "src/App.vue",
          to: "src/Child.vue",
          kind: "component",
          specifier: "./Child.vue",
        },
        {
          from: "src/App.vue",
          to: "src/Child.vue",
          kind: "import",
          specifier: "./Child.vue",
        },
      ],
    }));
    const buildInspectorDiff = vi.fn(() => ({
      lines: [
        { kind: "remove", leftLine: 1, rightLine: null, text: "[babel]" },
        { kind: "add", leftLine: null, rightLine: 1, text: "[babel]" },
      ],
      stats: {
        additions: 1,
        removals: 1,
        unchanged: 0,
      },
    }));
    const compiler = {
      compileSfc,
      typeCheck,
      analyzeSfc,
      analyzeCrossFile,
      buildInspectorGraph,
      buildInspectorDiff,
    } as unknown as WasmModule;

    const report = await compileInspectorReport({
      compiler,
      file: {
        path: "src/App.vue",
        source:
          "<script setup>import Child from './Child.vue'; const msg = 'hi'</script><template><Child />{{ msg }}</template>",
      },
      files: [
        {
          path: "src/App.vue",
          source:
            "<script setup>import Child from './Child.vue'; const msg = 'hi'</script><template><Child />{{ msg }}</template>",
        },
        {
          path: "src/Child.vue",
          source: "<template><span /></template>",
        },
      ],
      target: "dom",
    });

    expect(report.virtualTs.code).toBe("const __vts = 1;");
    expect(report.virtualTs.formattedCode).toMatchInlineSnapshot(`
      "[typescript]
      const __vts = 1;"
    `);
    expect(report.virtualTs.warnings).toEqual(["warning vts-note: virtual note"]);
    expect(report.vir.code).toBe("[vir]\nbindings=0\n");
    expect(report.graph.nodes).toHaveLength(2);
    expect(report.graph.edges).toEqual([
      {
        from: "src/App.vue",
        to: "src/Child.vue",
        kind: "component",
        specifier: "./Child.vue",
      },
      {
        from: "src/App.vue",
        to: "src/Child.vue",
        kind: "import",
        specifier: "./Child.vue",
      },
    ]);
    expect(report.stats).toEqual({ additions: 1, removals: 1, unchanged: 0 });
    expect(typeCheck).toHaveBeenCalledWith(expect.any(String), {
      filename: "src/App.vue",
      includeVirtualTs: true,
    });
    expect(buildInspectorGraph).toHaveBeenCalledWith([
      {
        path: "src/App.vue",
        source:
          "<script setup>import Child from './Child.vue'; const msg = 'hi'</script><template><Child />{{ msg }}</template>",
      },
      {
        path: "src/Child.vue",
        source: "<template><span /></template>",
      },
    ]);
    expect(buildInspectorDiff).toHaveBeenCalled();
  });

  it("compares script setup DOM output with Vue's inline production render shape", async () => {
    const buildInspectorDiff = vi.fn(() => ({
      lines: [],
      stats: {
        additions: 0,
        removals: 0,
        unchanged: 0,
      },
    }));
    const compiler = {
      compileSfc: vi.fn(() => ({
        descriptor: {
          filename: "src/App.vue",
          source: "",
          template: {
            content: "{{ msg }}",
            loc: { start: 0, end: 0 },
            attrs: {},
          },
          script: undefined,
          scriptSetup: {
            content: "const msg: string = 'hi'",
            loc: { start: 0, end: 0 },
            attrs: { lang: "ts" },
            lang: "ts",
            setup: true,
          },
          styles: [],
          customBlocks: [],
        },
        script: { code: "export default {}" },
        warnings: [],
      })),
      typeCheck: vi.fn(() => ({
        diagnostics: [],
        virtualTs: "",
        errorCount: 0,
        warningCount: 0,
      })),
      analyzeSfc: vi.fn(() => ({
        croquis: {
          is_setup: true,
          bindings: [],
          scopes: [],
          macros: [],
          props: [],
          emits: [],
          provides: [],
          injects: [],
          typeExports: [],
          invalidExports: [],
          diagnostics: [],
          stats: {
            binding_count: 0,
            unused_binding_count: 0,
            scope_count: 0,
            macro_count: 0,
            type_export_count: 0,
            invalid_export_count: 0,
            error_count: 0,
            warning_count: 0,
          },
        },
        diagnostics: [],
        vir: "",
      })),
      analyzeCrossFile: vi.fn(() => ({
        diagnostics: [],
        circularDependencies: [],
        stats: null,
        filePaths: ["src/App.vue"],
      })),
      buildInspectorGraph: vi.fn(() => ({
        nodes: [],
        edges: [],
      })),
      buildInspectorDiff,
    } as unknown as WasmModule;

    await compileInspectorReport({
      compiler,
      file: {
        path: "src/App.vue",
        source:
          "<script setup lang=\"ts\">const msg: string = 'hi'</script><template><div>{{ msg }}</div></template>",
      },
      target: "dom",
    });

    const [officialOutput] = buildInspectorDiff.mock.calls[0]!;

    expect(officialOutput).toMatchInlineSnapshot(`
      "[typescript]
      import { defineComponent as _defineComponent } from 'vue'
      import { toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock } from "vue"

      const msg: string = 'hi'
      export default /*@__PURE__*/_defineComponent({
        __name: 'App',
        setup(__props) {

      return (_ctx: any,_cache: any) => {
        return (_openBlock(), _createElementBlock("div", null, _toDisplayString(msg)))
      }
      }

      })"
    `);

    buildInspectorDiff.mockClear();

    await compileInspectorReport({
      compiler,
      file: {
        path: "src/App.vue",
        source:
          "<script setup lang=\"ts\">const msg: string = 'hi'</script><template><div>{{ msg }}</div></template>",
      },
      target: "ssr",
    });

    const [ssrOfficialOutput] = buildInspectorDiff.mock.calls[0]!;

    expect(ssrOfficialOutput).toMatchInlineSnapshot(`
      "[typescript]
      import { defineComponent as _defineComponent } from 'vue'
      const msg: string = 'hi'
      export default /*@__PURE__*/_defineComponent({
        __name: 'App',
        setup(__props, { expose: __expose }) {
        __expose();

      const __returned__ = { msg }
      Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })
      return __returned__
      }

      })

      import { ssrRenderAttrs as _ssrRenderAttrs, ssrInterpolate as _ssrInterpolate } from "vue/server-renderer"

      export function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {
        _push(\`<div\${
          _ssrRenderAttrs(_attrs)
        }>\${
          _ssrInterpolate($setup.msg)
        }</div>\`)
      }"
    `);
  });
});
