import { afterEach, beforeEach, describe, expect, it, vi } from "vite-plus/test";
import { effectScope } from "vue";
import type { CompilerOptions, CssCompileOptions, SfcCompileResult, WasmModule } from "../src/wasm";
import { useAtelierCompiler } from "../src/features/atelier/useAtelierCompiler";
import { createEmptyCodeOutputs } from "../src/features/atelier/codeOutputs";
import { formatCode, formatCss, transpileToJs } from "../src/features/atelier/formatters";

vi.mock("../src/features/atelier/formatters", () => ({
  formatCss: vi.fn(),
  formatCode: vi.fn(),
  transpileToJs: vi.fn(),
}));

function deferred() {
  let resolve!: (value: string) => void;
  const promise = new Promise<string>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

function setup() {
  const scope = effectScope();
  const compileSfc = vi.fn(
    (source: string, _options: CompilerOptions) =>
      ({
        descriptor: { source, styles: [{ content: source, scoped: false }] },
        script: { code: source, bindings: { bindings: { [source]: "setup-const" } } },
        template: { code: source, preamble: "", ast: { value: source }, helpers: [source] },
        errors: [],
        warnings: [],
      }) as unknown as SfcCompileResult,
  );
  const compileCss = vi.fn((source: string, options: CssCompileOptions) => ({
    code: `${source}|${options.minify}`,
    errors: [],
    warnings: [],
  }));
  const compiler = { compileSfc, compileCss } as unknown as WasmModule;
  const state = scope.run(() => useAtelierCompiler(() => compiler))!;
  state.source.value = "old";
  return { scope, state, compileSfc, compileCss };
}

function snapshot(state: ReturnType<typeof useAtelierCompiler>) {
  return {
    output: state.output.value,
    sfc: state.sfcResult.value,
    css: state.cssResult.value,
    formattedCss: state.formattedCss.value,
    code: state.codeOutputs.value,
    ast: state.astJson.value,
    bindings: state.groupedBindings.value,
    duration: state.compileTime.value,
    error: state.error.value,
  };
}

function expectEmpty(state: ReturnType<typeof useAtelierCompiler>) {
  expect(snapshot(state)).toEqual({
    output: null,
    sfc: null,
    css: null,
    formattedCss: "",
    code: createEmptyCodeOutputs(),
    ast: "{}",
    bindings: {},
    duration: null,
    error: null,
  });
}

function expectCompleted(state: ReturnType<typeof useAtelierCompiler>, formattedCss: string) {
  const variant = {
    code: "old",
    formattedCode: "old",
    formattedJsCode: "",
    isTypeScript: false,
    templates: [],
    warnings: [],
    error: null,
  };
  expect(snapshot(state)).toEqual({
    output: { code: "old", preamble: "", ast: { value: "old" }, helpers: ["old"] },
    sfc: {
      descriptor: { source: "old", styles: [{ content: "old", scoped: false }] },
      script: { code: "old", bindings: { bindings: { old: "setup-const" } } },
      template: { code: "old", preamble: "", ast: { value: "old" }, helpers: ["old"] },
      errors: [],
      warnings: [],
    },
    css: { code: "old|false", errors: [], warnings: [] },
    formattedCss,
    code: {
      dom: variant,
      ssr: variant,
      vapor: variant,
      vaporSsr: createEmptyCodeOutputs().vaporSsr,
    },
    ast: JSON.stringify({ value: "old" }, null, 2),
    bindings: { "setup-const": ["old"] },
    duration: expect.any(Number),
    error: null,
  });
  expect(state.isCompiling.value).toBe(false);
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.mocked(formatCode).mockImplementation(async (code) => code);
  vi.mocked(transpileToJs).mockImplementation(async (code) => code);
});

afterEach(() => {
  vi.useRealTimers();
  vi.resetAllMocks();
});

describe("Atelier asynchronous results", () => {
  it("invalidates old compilation as soon as experimental options change", async () => {
    const old = deferred();
    vi.mocked(formatCss).mockReturnValueOnce(old.promise).mockResolvedValue("current css");
    const { state, scope, compileSfc, compileCss } = setup();
    try {
      const pending = state.compile();
      state.experimentals.value = { experimentalInTagComments: true };
      const invalidatedVersion = state.codeOutputVersion.value;
      expectEmpty(state);
      old.resolve("old css");
      await pending;
      expectEmpty(state);
      expect(state.codeOutputVersion.value).toBe(invalidatedVersion);
      expect(state.isCompiling.value).toBe(false);
      expect(compileSfc).toHaveBeenCalledTimes(1);
      expect(compileCss).toHaveBeenCalledTimes(1);
      await vi.advanceTimersByTimeAsync(299);
      expectEmpty(state);
      expect(compileSfc).toHaveBeenCalledTimes(1);
      await vi.advanceTimersByTimeAsync(1);
      expectCompleted(state, "current css");
      expect(compileSfc).toHaveBeenCalledTimes(4);
      expect(compileCss).toHaveBeenCalledTimes(2);
      for (const [source, options] of compileSfc.mock.calls.slice(1)) {
        expect(source).toBe("old");
        expect(options).toMatchObject({ experimentalInTagComments: true });
      }
    } finally {
      scope.stop();
    }
  });

  it("keeps the newest CSS option result when formatters finish out of order", async () => {
    const old = deferred();
    const recent = deferred();
    vi.mocked(formatCss)
      .mockResolvedValueOnce("initial")
      .mockReturnValueOnce(old.promise)
      .mockReturnValueOnce(recent.promise);
    const { state, scope, compileSfc, compileCss } = setup();
    try {
      await state.compile();
      expectCompleted(state, "initial");
      state.cssOptions.value.minify = true;
      expectEmpty(state);
      expect(state.isCompiling.value).toBe(true);
      expect(compileSfc).toHaveBeenCalledTimes(4);
      expect(compileCss).toHaveBeenLastCalledWith("old", {
        minify: true,
        scoped: false,
        scopeId: "data-v-12345678",
      });
      state.cssOptions.value.minify = false;
      expectEmpty(state);
      expect(state.isCompiling.value).toBe(true);
      expect(compileSfc).toHaveBeenCalledTimes(5);
      expect(compileCss).toHaveBeenCalledTimes(3);
      recent.resolve("new css");
      await vi.advanceTimersByTimeAsync(0);
      expectCompleted(state, "new css");
      const completed = snapshot(state);
      const completedVersion = state.codeOutputVersion.value;
      old.resolve("old css");
      await vi.advanceTimersByTimeAsync(0);
      expect(snapshot(state)).toEqual(completed);
      expect(state.codeOutputVersion.value).toBe(completedVersion);
      expect(compileSfc).toHaveBeenCalledTimes(7);
      await vi.advanceTimersByTimeAsync(300);
      expect(snapshot(state)).toEqual(completed);
      expect(compileSfc).toHaveBeenCalledTimes(7);
      expect(compileCss).toHaveBeenCalledTimes(3);
    } finally {
      scope.stop();
    }
  });

  it("manual compilation consumes the queued source edit without compiling it twice", async () => {
    vi.mocked(formatCss).mockResolvedValue("current css");
    const { state, scope, compileSfc, compileCss } = setup();
    try {
      await state.compile();
      expectCompleted(state, "current css");
      const completed = snapshot(state);
      await vi.advanceTimersByTimeAsync(300);
      expect(snapshot(state)).toEqual(completed);
      expect(compileSfc).toHaveBeenCalledTimes(3);
      expect(compileCss).toHaveBeenCalledTimes(1);
    } finally {
      scope.stop();
    }
  });
});
