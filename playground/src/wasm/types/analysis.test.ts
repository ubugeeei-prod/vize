import { describe, expect, expectTypeOf, it } from "vite-plus/test";
import type {
  CrossFileComplexityHotspot,
  CrossFileComplexityInput,
  CrossFileResult,
} from "./analysis";

const complexityInputFixture = {
  componentCount: 1,
  templateIfCount: 2,
  templateForCount: 3,
  templateLogicalOperatorCount: 4,
  componentTreeVIfMaxDepth: 5,
  componentTreeVForMaxDepth: 6,
  componentTreeScopedSlotMaxDepth: 7,
  componentTreeTemplateNestingScore: 8,
  slotCount: 9,
  propDrillingEdgeCount: 10,
  globalStateReferenceCount: 11,
  provideInjectMaxDepth: 12,
  provideInjectReferenceCount: 13,
  provideInjectFanoutCount: 14,
  fallthroughRiskCount: 15,
  reactiveNodeCount: 16,
  reactiveEdgeCount: 17,
  reactiveCycleCount: 18,
} satisfies CrossFileComplexityInput;

const complexityHotspotFixture = {
  fileId: 7,
  fileName: "App.vue",
  componentName: "App",
  input: complexityInputFixture,
  dimensions: {
    templateControlFlow: 1,
    slotUsage: 2,
    propDrilling: 3,
    globalState: 4,
    provideInject: 5,
    fallthroughAttrs: 6,
    reactiveGraph: 7,
  },
  totalScore: 28,
  dominantDimension: {
    dimension: "reactive-graph",
    score: 7,
  },
} satisfies CrossFileComplexityHotspot;

describe("CrossFileComplexityInput", () => {
  it("keeps the serialized WASM input shape in sync", () => {
    expect(Object.keys(complexityInputFixture)).toEqual([
      "componentCount",
      "templateIfCount",
      "templateForCount",
      "templateLogicalOperatorCount",
      "componentTreeVIfMaxDepth",
      "componentTreeVForMaxDepth",
      "componentTreeScopedSlotMaxDepth",
      "componentTreeTemplateNestingScore",
      "slotCount",
      "propDrillingEdgeCount",
      "globalStateReferenceCount",
      "provideInjectMaxDepth",
      "provideInjectReferenceCount",
      "provideInjectFanoutCount",
      "fallthroughRiskCount",
      "reactiveNodeCount",
      "reactiveEdgeCount",
      "reactiveCycleCount",
    ]);
    expect(complexityInputFixture.provideInjectFanoutCount).toBe(14);
    expectTypeOf<CrossFileComplexityInput["provideInjectFanoutCount"]>().toEqualTypeOf<number>();
  });

  it("keeps ranked hotspots on the cross-file result contract", () => {
    const result = {
      complexityHotspots: [complexityHotspotFixture],
    } satisfies Pick<CrossFileResult, "complexityHotspots">;

    expect(result.complexityHotspots[0]?.dominantDimension).toEqual({
      dimension: "reactive-graph",
      score: 7,
    });
    expectTypeOf<CrossFileResult["complexityHotspots"]>().toEqualTypeOf<
      CrossFileComplexityHotspot[]
    >();
  });
});
