import { describe, expect, it, vi } from "vite-plus/test";
import type {
  CompileResult,
  CompilerOptions,
  SfcCompileResult,
  WasmModule,
} from "../../wasm/index";
import { compileCodeOutputs, getCodeOutputTargets } from "./codeOutputs";

vi.mock("./formatters", () => ({
  formatCode: vi.fn(async (code: string, parser: string) => `[${parser}] ${code}`),
  transpileToJs: vi.fn(async (code: string) => `js:${code}`),
}));

function createCompileResult(code: string, templates?: string[]): CompileResult {
  return {
    code,
    preamble: "",
    ast: {},
    helpers: [],
    templates,
  };
}

function createSfcResult(
  scriptCode: string,
  {
    lang,
    templateCode,
    templates,
    warnings = [],
    templateContent = "<div />",
    bindings = {
      bindings: {
        count: "props",
        doubled: "setup-ref",
      },
      propsAliases: {},
      isScriptSetup: true,
    },
  }: {
    lang?: string;
    templateCode?: string;
    templates?: string[];
    warnings?: string[];
    templateContent?: string;
    bindings?: {
      bindings: Record<string, string>;
      propsAliases?: Record<string, string>;
      isScriptSetup?: boolean;
    };
  } = {},
): SfcCompileResult {
  return {
    descriptor: {
      filename: "example.vue",
      source: "",
      styles: [],
      customBlocks: [],
      template: {
        content: templateContent,
        loc: { start: 0, end: 0 },
        attrs: {},
      },
      scriptSetup: lang
        ? {
            content: "",
            loc: { start: 0, end: 0 },
            attrs: {},
            lang,
            setup: true,
          }
        : undefined,
    },
    script: {
      code: scriptCode,
      bindings,
    },
    template: createCompileResult(templateCode || `${scriptCode}:template`, templates),
    warnings,
  };
}

describe("compileCodeOutputs", () => {
  it("compiles template outputs for SSR, Vapor, and Vapor SSR variants", async () => {
    const compile = vi.fn((_: string, options: CompilerOptions) =>
      createCompileResult(options.ssr ? "ssr-code" : "dom-code"),
    );
    const compileVapor = vi.fn((_: string, options: CompilerOptions) =>
      createCompileResult(options.ssr ? "vapor-ssr-code" : "vapor-code", ["tpl-a"]),
    );
    const compiler = {
      compile,
      compileVapor,
    } as unknown as WasmModule;

    const outputs = await compileCodeOutputs({
      compiler,
      inputMode: "template",
      source: "<div />",
      options: { mode: "module" },
      baseOutput: createCompileResult("dom-code"),
      baseSfcResult: null,
    });

    expect(outputs.dom.code).toBe("dom-code");
    expect(outputs.ssr.code).toBe("ssr-code");
    expect(outputs.vapor.code).toBe("vapor-code");
    expect(outputs.vaporSsr.code).toBe("vapor-ssr-code");
    expect(outputs.vapor.templates).toEqual(["tpl-a"]);
    expect(outputs.vaporSsr.templates).toEqual(["tpl-a"]);
    expect(compile).toHaveBeenNthCalledWith(1, "<div />", {
      mode: "module",
      ssr: true,
    });
    expect(compileVapor).toHaveBeenNthCalledWith(1, "<div />", {
      mode: "module",
      ssr: false,
    });
    expect(compileVapor).toHaveBeenNthCalledWith(2, "<div />", {
      mode: "module",
      ssr: true,
    });
  });

  it("exposes Vapor SSR only for template inputs", () => {
    expect(getCodeOutputTargets("template")).toEqual(["dom", "ssr", "vapor", "vaporSsr"]);
    expect(getCodeOutputTargets("sfc")).toEqual(["dom", "ssr", "vapor"]);
  });

  it("normalizes SFC SSR and Vapor template outputs using script setup bindings", async () => {
    const compileSfc = vi.fn((_: string, options: CompilerOptions) => {
      if (options.outputMode === "vapor") {
        return createSfcResult("dom-script", {
          lang: "ts",
          templateCode:
            "export function render(_ctx) { return _toDisplayString(_ctx.count) + _toDisplayString(_ctx.doubled) }",
          templates: ["tpl-vapor"],
        });
      }
      if (options.ssr) {
        return createSfcResult("dom-script", {
          lang: "ts",
          templateCode:
            "function ssrRender(_ctx, _push, _parent, _attrs) { _push(_ssrInterpolate(_ctx.count) + _ssrInterpolate(_ctx.doubled)) }",
        });
      }
      return createSfcResult("dom-script", {
        lang: "ts",
        templateCode: "dom-template",
      });
    });
    const compiler = {
      compileSfc,
    } as unknown as WasmModule;

    const outputs = await compileCodeOutputs({
      compiler,
      inputMode: "sfc",
      source: "<template><div /></template>",
      options: { mode: "module", scriptExt: "preserve" },
      baseOutput: null,
      baseSfcResult: createSfcResult("dom-script", {
        lang: "ts",
        templateCode: "dom-template",
      }),
    });

    expect(outputs.dom.code).toBe("dom-script");
    expect(outputs.dom.isTypeScript).toBe(true);
    expect(outputs.dom.formattedJsCode).toBe("[babel] js:dom-script");
    expect(outputs.ssr.code).toContain("_ssrInterpolate(count)");
    expect(outputs.ssr.code).toContain("_ssrInterpolate(doubled.value)");
    expect(outputs.ssr.code).not.toContain("_ctx.count");
    expect(outputs.ssr.code).not.toContain("_ctx.doubled");
    expect(outputs.vapor.code).toContain("_toDisplayString(count)");
    expect(outputs.vapor.code).toContain("_toDisplayString(doubled.value)");
    expect(outputs.vapor.templates).toEqual(["tpl-vapor"]);
    expect(outputs.vaporSsr.code).toBe("");
    expect(compileSfc).toHaveBeenNthCalledWith(1, "<template><div /></template>", {
      mode: "module",
      scriptExt: "preserve",
      ssr: true,
      outputMode: "vdom",
    });
    expect(compileSfc).toHaveBeenNthCalledWith(2, "<template><div /></template>", {
      mode: "module",
      scriptExt: "preserve",
      ssr: false,
      outputMode: "vapor",
    });
  });

  it("preserves SFC warnings for each rendered output target", async () => {
    const compileSfc = vi.fn((_: string, options: CompilerOptions) => {
      if (options.outputMode === "vapor") {
        return createSfcResult("dom-script", {
          templateCode: "vapor-template",
          warnings: ["vapor warning"],
        });
      }
      return createSfcResult("dom-script", {
        templateCode: "ssr-template",
        warnings: ["ssr warning"],
      });
    });
    const compiler = {
      compileSfc,
    } as unknown as WasmModule;

    const outputs = await compileCodeOutputs({
      compiler,
      inputMode: "sfc",
      source: "<template><div /></template>",
      options: { mode: "module" },
      baseOutput: null,
      baseSfcResult: createSfcResult("dom-script", {
        templateCode: "dom-template",
        warnings: ["dom warning"],
      }),
    });

    expect(outputs.dom.warnings).toEqual(["dom warning"]);
    expect(outputs.ssr.warnings).toEqual(["ssr warning"]);
    expect(outputs.vapor.warnings).toEqual(["vapor warning"]);
    expect(outputs.vaporSsr.warnings).toEqual([]);
  });
});
