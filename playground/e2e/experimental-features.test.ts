import { beforeAll, describe, expect, it } from "vite-plus/test";
import { mount } from "@vue/test-utils";
import { loadWasm, type WasmModule } from "../src/wasm";
import ExperimentalFeatures from "../src/shared/ExperimentalFeatures.vue";
import { mapGeneratedRange } from "../src/features/canon/sourceMappings";
import {
  EXPERIMENTAL_FEATURES,
  featuresFor,
  parseExperimentalOptions,
} from "../src/shared/experimentalFeatures";

let wasm: WasmModule;
beforeAll(async () => {
  wasm = await loadWasm();
});
const example = (key: string) =>
  EXPERIMENTAL_FEATURES.find((feature) => feature.key === key)!.source;

describe("experimental feature controls", () => {
  it("only admits supported, explicit boolean flags", () => {
    expect(
      parseExperimentalOptions(
        { experimentalInTagComments: "true", experimentalSelfComponent: true },
        "croquis",
      ),
    ).toEqual({ experimentalInTagComments: false });
    expect(parseExperimentalOptions(null, "compiler")).toEqual({});
    expect(parseExperimentalOptions([], "compiler")).toEqual({});
    expect(featuresFor("typechecker").map(({ key }) => key)).toEqual([
      "experimentalInTagComments",
      "experimentalStrictSlotChildren",
    ]);
  });

  it("emits checkbox changes and example selections without replacing other flags", async () => {
    const wrapper = mount(ExperimentalFeatures, {
      props: { scope: "compiler", modelValue: { experimentalSelfComponent: true } },
    });
    await wrapper.find("input").setValue(true);
    expect(wrapper.emitted("update:modelValue")).toEqual([
      [
        {
          experimentalSelfComponent: true,
          experimentalInTagComments: true,
        },
      ],
    ]);
    await wrapper.find("select").setValue("experimentalPatternedTemplate");
    expect(wrapper.emitted("example")).toEqual([["experimentalPatternedTemplate"]]);
    expect((wrapper.find("select").element as HTMLSelectElement).value).toBe("");
    expect(wrapper.find("summary").text()).toBe("Experimental (1)");
    wrapper.unmount();
  });
});

describe("experimental flags in real WASM", () => {
  it("compiles patterned templates in VDOM, SSR, and Vapor only when opted in", () => {
    const source = example("experimentalPatternedTemplate");
    for (const target of [{}, { ssr: true }, { outputMode: "vapor" as const }]) {
      let rejection = "";
      try {
        rejection = (wasm.compileSfc(source, target).errors ?? []).join("\n");
      } catch (error) {
        rejection = String(error);
      }
      expect(rejection).toMatch(/require.*experimentals.patternedTemplate/);
      const on = wasm.compileSfc(source, { ...target, experimentalPatternedTemplate: true });
      expect(on.errors ?? []).toEqual([]);
      expect(on.template?.code || on.script?.code).toMatch(/Ready/);
      const nested = source
        .replace("<template v-match=", "<template><template v-match=")
        .replace("</template>", "</template></template>");
      const inner = wasm.compileSfc(nested, { ...target, experimentalPatternedTemplate: true });
      expect(on.template?.code).toBe(inner.template?.code);
      expect(on.script?.code).toBe(inner.script?.code);
      expect(on.descriptor.template?.content).not.toContain("v-match");
    }
  });

  it("preserves in-tag comments through compiler, typechecker, and Croquis", () => {
    const source = example("experimentalInTagComments");
    const options = { experimentalInTagComments: true };
    expect(wasm.compileSfc(source, options).errors ?? []).toEqual([]);
    expect(wasm.typeCheck(source, options).diagnostics).toEqual([]);
    const analysis = wasm.analyzeSfc(source, options);
    expect(analysis.croquis.bindings.find(({ name }) => name === "label")?.usedInTemplate).toBe(
      true,
    );
    expect(wasm.typeCheck(source, {}).errorCount).toBeGreaterThan(0);
    expect(() => wasm.analyzeSfc(source, {})).toThrow();
    expect(wasm.typeCheck(source, { experimentalInTagComments: false }).errorCount).toBeGreaterThan(
      0,
    );
  });

  it("changes self-component resolution in all compiler targets", () => {
    const source = example("experimentalSelfComponent");
    for (const target of [{}, { ssr: true }, { outputMode: "vapor" as const }]) {
      const off = wasm.compileSfc(source, { filename: "Tree.vue", ...target });
      const on = wasm.compileSfc(source, {
        filename: "Tree.vue",
        ...target,
        experimentalSelfComponent: true,
      });
      expect(on.errors ?? []).toEqual([]);
      expect(on.template?.code).not.toEqual(off.template?.code);
    }
  });

  it("adds and removes strict slot children checks in virtual TypeScript", () => {
    const source = example("experimentalStrictSlotChildren");
    const off = wasm.typeCheck(source, { includeVirtualTs: true });
    const on = wasm.typeCheck(source, {
      includeVirtualTs: true,
      experimentalStrictSlotChildren: true,
    });
    expect(off.virtualTs).not.toContain("const __vize_slot_children_");
    expect(on.virtualTs).toContain("const __vize_slot_children_");
    expect(on.virtualTs).toContain("__VizeProvidedSlotChildren<[HTMLInputElement]>");
    expect(
      wasm.typeCheck(source, { includeVirtualTs: true, experimentalStrictSlotChildren: false })
        .virtualTs,
    ).toBe(off.virtualTs);
  });

  it("exposes UTF-16 mappings for synthetic strict-slot checks after non-ASCII source", () => {
    const source = example("experimentalStrictSlotChildren").replace(
      '<script setup lang="ts">',
      '<script setup lang="ts">\n// \u65e5\u672c\u8a9e \ud83c\udfa8',
    );
    const result = wasm.typeCheck(source, {
      includeVirtualTs: true,
      experimentalStrictSlotChildren: true,
    });
    const start = result.virtualTs!.indexOf("const __vize_slot_children_") + "const ".length;
    const range = mapGeneratedRange(
      start,
      start + "__vize_slot_children_0_default".length,
      result.sourceMappings!,
    );
    expect(range).not.toBeNull();
    expect(source.slice(range!.start, range!.end)).toBe("<input />");
  });
});
