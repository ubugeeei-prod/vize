import { describe, expect, it } from "vite-plus/test";
import { createTransformAnalyzeSfc } from "./wasm-transform";

describe("createTransformAnalyzeSfc", () => {
  it("passes through reactivity overlay data", () => {
    const analyze = createTransformAnalyzeSfc(() => ({
      croquis: {
        is_setup: true,
        scopes: [],
        bindings: [],
        macros: [],
        props: [],
        emits: [],
        provides: [],
        injects: [],
        reactivityOverlay: {
          summary: {
            sourceCount: 1,
            refSourceCount: 1,
            reactiveSourceCount: 0,
            computedSourceCount: 0,
            readonlySourceCount: 0,
            needsValueAccessCount: 1,
            lossCount: 1,
            effectEdgeCount: 0,
            effectCycleCount: 0,
          },
          sources: [
            {
              id: 0,
              name: "count",
              kind: "ref",
              category: "ref",
              needsValueAccess: true,
              declarationOffset: 10,
              declarationEndOffset: 15,
            },
          ],
          losses: [
            {
              kind: "refValueExtract",
              category: "loss",
              sourceName: "count",
              targetName: "plain",
              extractedProps: [],
              start: 20,
              end: 30,
            },
          ],
          effectGraph: {
            edges: [],
            cycle: null,
          },
        },
      },
      diagnostics: [],
      vir: "",
    }));

    const result = analyze("", {});

    expect(result.croquis.reactivityOverlay.summary.sourceCount).toBe(1);
    expect(result.croquis.reactivityOverlay.losses[0]?.targetName).toBe("plain");
  });
});
