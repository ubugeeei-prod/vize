import { effectScope, nextTick } from "vue";
import { afterEach, describe, expect, it, vi } from "vite-plus/test";
import type { SfcCompileResult, WasmModule } from "../../wasm/index";
import { formatCode, formatCss } from "./formatters";
import { useAtelierCompiler } from "./useAtelierCompiler";

vi.mock("./formatters", () => ({
  formatCode: vi.fn(async (code: string) => code),
  transpileToJs: vi.fn(async (code: string) => code),
  formatCss: vi.fn(async (code: string) => code),
}));

function sfcResult(source: string): SfcCompileResult {
  return {
    descriptor: {
      filename: "Component.vue",
      source,
      styles: [{ content: source, attrs: {}, loc: { start: 0, end: source.length } }],
      customBlocks: [],
    },
    script: { code: source, bindings: { bindings: { old: "setup-const" } } },
    template: { code: source, preamble: "", ast: { source }, helpers: ["oldHelper"] },
    errors: [],
    warnings: [],
  } as SfcCompileResult;
}

const scopes: ReturnType<typeof effectScope>[] = [];
function harness() {
  const compileSfc = vi.fn((source: string) => {
    if (source === "invalid") throw new Error("Invalid SFC");
    return sfcResult(source);
  });
  const compiler = {
    compileSfc,
    compile: vi.fn((source: string) => ({ code: source, ast: { source }, helpers: [] })),
    compileVapor: vi.fn((source: string) => ({ code: source, ast: { source }, helpers: [] })),
    compileCss: vi.fn((source: string) => ({ code: source, errors: [], warnings: [] })),
  } as unknown as WasmModule;
  const scope = effectScope();
  scopes.push(scope);
  const state = scope.run(() => useAtelierCompiler(() => compiler))!;
  return { state, compileSfc };
}

function expectEmpty(state: ReturnType<typeof useAtelierCompiler>) {
  expect(state.output.value).toBeNull();
  expect(state.sfcResult.value).toBeNull();
  expect(state.cssResult.value).toBeNull();
  expect(state.formattedCss.value).toBe("");
  expect(state.compileTime.value).toBeNull();
  expect(state.astJson.value).toBe("{}");
  expect(state.groupedBindings.value).toEqual({});
  expect(Object.values(state.codeOutputs.value).every((output) => output.code === "")).toBe(true);
}

afterEach(() => {
  scopes.splice(0).forEach((scope) => scope.stop());
  vi.mocked(formatCode)
    .mockReset()
    .mockImplementation(async (code) => code);
  vi.mocked(formatCss)
    .mockReset()
    .mockImplementation(async (code) => code);
});

describe("Atelier result ownership", () => {
  it("clears all previous results when the current compilation fails", async () => {
    const { state } = harness();
    await state.compile();
    expect(state.output.value).not.toBeNull();
    expect(state.cssResult.value).not.toBeNull();
    state.source.value = "invalid";
    await nextTick();
    await state.compile();
    expect(state.error.value).toBe("Invalid SFC");
    expectEmpty(state);
  });

  it.each([
    { label: "empty", source: "" },
    { label: "UTF-8", source: "日本語😀" },
    { label: "large", source: "x".repeat(100_000) },
  ])("invalidates every pane while a $label input waits for compilation", async ({ source }) => {
    const { state } = harness();
    await state.compile();
    state.source.value = source;
    await nextTick();
    expectEmpty(state);
    await state.compile();
    expect(state.codeOutputs.value.dom.code).toBe(source);
    expect(state.error.value).toBeNull();
  });

  it("invalidates output when compiler, mode, or CSS options change", async () => {
    const { state } = harness();
    await state.compile();
    state.options.value.ssr = true;
    expectEmpty(state);
    await state.compile();
    state.cssOptions.value.minify = true;
    expectEmpty(state);
    await state.compile();
    state.inputMode.value = "template";
    expectEmpty(state);
    await state.compile();
    expect(state.output.value).not.toBeNull();
    expect(state.sfcResult.value).toBeNull();
    expect(state.cssResult.value).toBeNull();
  });

  it("does not expose a partial SFC while its CSS formatting is pending", async () => {
    const { state } = harness();
    await state.compile();
    let finish!: (code: string) => void;
    vi.mocked(formatCss).mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
    state.source.value = "new";
    await nextTick();
    const pending = state.compile();
    expectEmpty(state);
    state.source.value = "latest";
    await nextTick();
    finish("obsolete CSS");
    await pending;
    expectEmpty(state);
    await state.compile();
    expect(state.output.value?.code).toBe("latest");
    expect(state.formattedCss.value).toBe("latest");
  });

  it.each(["sfc", "template"] as const)(
    "keeps the latest %s result when an older code formatter finishes last",
    async (mode) => {
      const { state } = harness();
      state.inputMode.value = mode;
      state.source.value = "old";
      await nextTick();
      let finish!: (code: string) => void;
      vi.mocked(formatCode).mockImplementationOnce(
        () =>
          new Promise((resolve) => {
            finish = resolve;
          }),
      );
      const pending = state.compile();
      await vi.waitFor(() => expect(finish).toBeTypeOf("function"));
      expectEmpty(state);
      state.source.value = "latest";
      await nextTick();
      await state.compile();
      expect(state.codeOutputs.value.dom.code).toBe("latest");
      finish("obsolete formatted code");
      await pending;
      expect(state.output.value?.code).toBe("latest");
      expect(state.codeOutputs.value.dom.formattedCode).toBe("latest");
      expect(state.isCompiling.value).toBe(false);
    },
  );
});
