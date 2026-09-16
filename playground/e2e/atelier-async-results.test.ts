import { afterEach, describe, expect, it, vi } from "vite-plus/test";
import { effectScope, nextTick } from "vue";
import type { SfcCompileResult, WasmModule } from "../src/wasm";
import { useAtelierCompiler } from "../src/features/atelier/useAtelierCompiler";
import { formatCss } from "../src/features/atelier/formatters";

vi.mock("../src/features/atelier/formatters", () => ({
  formatCss: vi.fn(),
  formatCode: vi.fn(async (code: string) => code),
  transpileToJs: vi.fn(async (code: string) => code),
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
    () =>
      ({
        descriptor: { styles: [{ content: ".example {}", scoped: false }] },
        script: { code: "const value = 1" },
        errors: [],
      }) as unknown as SfcCompileResult,
  );
  const compiler = {
    compileSfc,
    compileCss: vi.fn((_source, options) => ({ code: String(options.minify) })),
  } as unknown as WasmModule;
  const state = scope.run(() => useAtelierCompiler(() => compiler))!;
  return { scope, state, compileSfc };
}

afterEach(() => {
  vi.useRealTimers();
  vi.resetAllMocks();
});

describe("Atelier asynchronous results", () => {
  it("invalidates old compilation as soon as experimental options change", async () => {
    vi.useFakeTimers();
    const old = deferred();
    vi.mocked(formatCss).mockReturnValueOnce(old.promise);
    const { state, scope, compileSfc } = setup();
    try {
      const pending = state.compile();
      state.experimentals.value = { experimentalInTagComments: true };
      await nextTick();
      old.resolve("old css");
      await pending;
      expect(state.formattedCss.value).toBe("");
      expect(state.codeOutputVersion.value).toBe(0);
      expect(state.isCompiling.value).toBe(false);
      expect(compileSfc).toHaveBeenCalledTimes(1);
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
    const { state, scope } = setup();
    try {
      await state.compile();
      state.cssOptions.value.minify = true;
      await nextTick();
      state.cssOptions.value.minify = false;
      await nextTick();
      recent.resolve("new css");
      await nextTick();
      old.resolve("old css");
      await nextTick();
      expect(state.formattedCss.value).toBe("new css");
    } finally {
      scope.stop();
    }
  });
});
