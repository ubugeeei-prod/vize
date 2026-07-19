import { describe, expect, it, vi } from "vite-plus/test";
import { compileInspectorReport } from "./compareCompilers";
import type { WasmModule } from "../../wasm/index";

vi.mock("../atelier/formatters", () => ({
  formatCode: vi.fn(async (code: string, parser: string) => `[${parser}]\n${code}`),
}));

describe("compileInspectorReport (vapor)", () => {
  // Regression test for #2969: the Vapor pane must show the full compiled SFC
  // module (`script.code`), not the template-only render function, and the
  // official reference must compile in vapor mode even without a `vapor` attr.
  it("prefers the full Vapor SFC module and compiles the official reference in vapor mode", async () => {
    const buildInspectorDiff = vi.fn(() => ({
      lines: [],
      stats: {
        additions: 0,
        removals: 0,
        unchanged: 0,
      },
    }));
    const fullVaporModule = [
      "const t0 = _template('<div> </div>', true)",
      "export default _defineVaporComponent({ setup(__props) { const msg = 'hi' } })",
    ].join("\n");
    const compiler = {
      compileSfc: vi.fn(() => ({
        descriptor: {
          filename: "src/App.vue",
          source: "",
          template: {
            content: "<div>{{ msg }}</div>",
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
        script: { code: fullVaporModule },
        template: { code: "export function render(_ctx) {}" },
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

    const report = await compileInspectorReport({
      compiler,
      file: {
        path: "src/App.vue",
        source:
          "<script setup lang=\"ts\">const msg: string = 'hi'</script><template><div>{{ msg }}</div></template>",
      },
      target: "vapor",
    });

    expect(report.vize.code).toBe(fullVaporModule);

    const [officialOutput] = buildInspectorDiff.mock.calls[0]!;

    expect(officialOutput).toMatchInlineSnapshot(`
      "[typescript]
      import { defineVaporComponent as _defineVaporComponent } from 'vue'
      import { txt as _txt, toDisplayString as _toDisplayString, setText as _setText, template as _template } from 'vue';
      const t0 = _template("<div> ", true)
      const msg: string = 'hi'
      export default /*@__PURE__*/_defineVaporComponent({
        __name: 'App',
        __multiRoot: false,
        setup(__props) {


        const n0 = t0()
        const x0 = _txt(n0)
        _setText(x0, _toDisplayString(msg))
        return n0

      }

      })"
    `);
  });
});
