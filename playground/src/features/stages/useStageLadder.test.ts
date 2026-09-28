import { defineComponent, h, nextTick } from "vue";
import { mount } from "@vue/test-utils";
import { afterEach, describe, expect, it, vi } from "vite-plus/test";
import type { CompilerOptions, SfcCompileResult, WasmModule } from "../../wasm/index";
import { formatCode } from "../atelier/formatters";
import { useStageLadder } from "./useStageLadder";

vi.mock("../atelier/formatters", () => ({
  formatCode: vi.fn(async (code: string) => code),
  transpileToJs: vi.fn(async (code: string) => code),
}));

function result(input: string, target: "dom" | "ssr" | "vapor"): SfcCompileResult {
  const match = /<template[^>]*>([\s\S]*?)<\/template>/.exec(input);
  const content = match?.[1] ?? "";
  const start = match ? match.index + match[0].indexOf(content) : 0;
  const authoredSyntax = input.includes('lang="pug"') ? "pug" : "vue-template";
  const code = `${target}-module:${input.includes("NEW") ? "new" : "old"}`;
  const fallback = target === "vapor";
  return {
    descriptor: {
      filename: "Component.vue",
      source: input,
      template: { content, loc: { start, end: start + content.length }, attrs: {} },
      styles: [],
      customBlocks: [],
    },
    script: { code },
    template: { code: `${target}-template`, preamble: "", ast: {}, helpers: [] },
    errors: [],
    warnings: [],
    stageCapture: {
      schema_version: 2,
      command: "compile-sfc",
      source: {
        path: "Component.vue",
        container: "vue-sfc",
        authored_syntax: authoredSyntax,
        compiled_syntax: "vue-template",
        template_span: { start, end: start + content.length },
      },
      target,
      outcome: {
        kind: fallback ? "legacy" : "accepted",
        reason: fallback ? "legacy selected" : null,
      },
      observed: { timings: false, remarks: false },
      options: [],
      pages: fallback
        ? []
        : [
            { level: "l1", step: "parse", text: content },
            { level: "l2", step: "lower", text: "ops=1\n" },
            ...(target === "ssr"
              ? [{ level: "l3", step: "lower", text: "[l3-dump-v2.ops]\n" }]
              : []),
            { level: "l4", step: "emit", text: code },
          ],
      timings: [],
      remarks: [],
    },
  };
}

function harness(domFailure = false) {
  const compileSfc = vi.fn((input: string, options: CompilerOptions) => {
    if (domFailure && !options.ssr && options.outputMode !== "vapor") {
      throw new Error("DOM route rejected");
    }
    return result(input, options.outputMode === "vapor" ? "vapor" : options.ssr ? "ssr" : "dom");
  });
  const analyzeSfc = vi.fn();
  const compiler = { compileSfc, analyzeSfc } as unknown as WasmModule;
  let state!: ReturnType<typeof useStageLadder>;
  const wrapper = mount(
    defineComponent({
      setup() {
        state = useStageLadder(() => compiler);
        return () => h("div");
      },
    }),
  );
  return { state, wrapper, compileSfc, analyzeSfc };
}

afterEach(() => {
  vi.mocked(formatCode).mockReset();
  vi.mocked(formatCode).mockImplementation(async (code) => code);
});

describe("product stage tab", () => {
  it("uses each target's own compiled module and stages without analyzeSfc", async () => {
    const { state, wrapper, compileSfc, analyzeSfc } = harness();
    await vi.waitFor(() => expect(state.outputs.value.vapor.code).toContain("vapor-module"));

    expect(compileSfc).toHaveBeenCalledTimes(3);
    expect(compileSfc.mock.calls.every(([, options]) => options.captureStages === true)).toBe(true);
    expect(analyzeSfc).not.toHaveBeenCalled();
    expect(state.ladder.value?.rungs.map((rung) => rung.id)).toEqual(["l1", "l2"]);
    expect(state.outputs.value.dom.code).toContain("dom-module");

    state.outputTarget.value = "ssr";
    await nextTick();
    expect(state.ladder.value?.rungs.map((rung) => rung.id)).toEqual(["l1", "l2", "l3"]);
    expect(state.outputs.value.ssr.code).toContain("ssr-module");

    state.outputTarget.value = "vapor";
    await nextTick();
    expect(state.ladder.value).toBeNull();
    expect(state.captureStatus.value).toBe("VAPOR legacy: legacy selected");
    expect(state.outputs.value.vapor.code).toContain("vapor-module");
    wrapper.unmount();
  });

  it("keeps SSR and Vapor results when the DOM route rejects", async () => {
    const { state, wrapper, compileSfc } = harness(true);
    await vi.waitFor(() => expect(state.outputs.value.ssr.code).toContain("ssr-module"));
    expect(compileSfc).toHaveBeenCalledTimes(3);
    expect(state.error.value).toBeNull();
    expect(state.outputs.value.dom.error).toBe("DOM route rejected");
    state.outputTarget.value = "ssr";
    await nextTick();
    expect(state.ladder.value?.rungs.map(({ id }) => id)).toEqual(["l1", "l2", "l3"]);
    expect(state.outputs.value.ssr.code).toContain("ssr-module");
    wrapper.unmount();
  });

  it("does not let formatting from an older source replace a newer compile", async () => {
    let release: (text: string) => void = () => {};
    let waiting = false;
    vi.mocked(formatCode).mockImplementation((code) =>
      code === "dom-module:old"
        ? new Promise<string>((resolve) => {
            waiting = true;
            release = resolve;
          })
        : Promise.resolve(code),
    );
    const { state, wrapper } = harness();
    await vi.waitFor(() => expect(waiting).toBe(true));
    state.source.value = "<template><div>NEW</div></template>";
    await new Promise((resolve) => setTimeout(resolve, 280));
    await vi.waitFor(() => expect(state.outputs.value.dom.code).toBe("dom-module:new"));
    release("dom-module:old");
    await nextTick();
    expect(state.outputs.value.dom.code).toBe("dom-module:new");
    wrapper.unmount();
  });

  it("only maps native spans to exactly matching authored template bytes", async () => {
    const { state, wrapper } = harness();
    await vi.waitFor(() => expect(state.outputs.value.dom.code).toContain("dom-module"));
    state.locateRemark({
      stage: "l2",
      pass: "lower",
      kind: "analysis",
      name: "source",
      span: { start: 0, end: 3 },
      args: [],
    });
    expect(state.highlights.value).toHaveLength(1);

    state.source.value = '<template lang="pug">div NEW</template>';
    await new Promise((resolve) => setTimeout(resolve, 280));
    await vi.waitFor(() => expect(state.outputs.value.dom.code).toBe("dom-module:new"));
    state.locateRemark({
      stage: "l2",
      pass: "lower",
      kind: "analysis",
      name: "source",
      span: { start: 0, end: 3 },
      args: [],
    });
    expect(state.highlights.value).toEqual([]);
    expect(state.syntaxNote.value).toContain("pug source compiled as vue-template");
    wrapper.unmount();
  });
});
