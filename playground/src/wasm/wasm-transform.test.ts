import { describe, expect, it } from "vite-plus/test";
import { createTransformAnalyzeSfc } from "./wasm-transform";

describe("createTransformAnalyzeSfc", () => {
  it("preserves scope origin flags, ranges, and parent-child relationships", () => {
    const scopes = [
      { id: 0, kind: "setup", start: 10, end: 30, bindings: ["subject"], isTemplateScope: false },
      {
        id: 1,
        kind: "v-match",
        start: 40,
        end: 90,
        bindings: [],
        parentIds: [0],
        isTemplateScope: true,
      },
      {
        id: 2,
        kind: "v-when",
        start: 55,
        end: 80,
        bindings: ["row"],
        parentIds: [1],
        isTemplateScope: true,
      },
      { id: 3, kind: "mod", start: 0, end: 0, bindings: [] },
    ];
    const analyze = createTransformAnalyzeSfc(() => ({ croquis: { scopes } }));
    const actual = analyze("", {}).croquis.scopes;
    expect(actual.map(({ isTemplateScope }) => isTemplateScope)).toEqual([
      false,
      true,
      true,
      undefined,
    ]);
    expect(actual[1]).toMatchObject({
      kind: "v-match",
      start: 40,
      end: 90,
      bindings: [],
      parentIds: [0],
      children: [2],
    });
    expect(actual[2]).toMatchObject({
      kind: "v-when",
      start: 55,
      end: 80,
      bindings: ["row"],
      parentIds: [1],
      children: [],
    });
  });

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

  it("passes the Spolvero feed through un-interpreted for negotiation downstream", () => {
    const spolvero = { schema_version: 99, command: "analyze-sfc", pages: "opaque" };
    const analyze = createTransformAnalyzeSfc(() => ({ croquis: {}, spolvero }));
    expect(analyze("", {}).spolvero).toBe(spolvero);
    expect(createTransformAnalyzeSfc(() => ({ croquis: {} }))("", {}).spolvero).toBeUndefined();
  });
});
