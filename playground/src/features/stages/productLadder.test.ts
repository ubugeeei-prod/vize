import { describe, expect, it } from "vite-plus/test";
import type { SfcCompileResult } from "../../wasm/index";
import { negotiateProductCapture } from "../../wasm/types/productCapture";
import { buildProductLadder } from "./productLadder";

const source = "<template><div>hello</div></template>";
const result = {
  descriptor: { template: { content: "<div>hello</div>" } },
  script: { code: "export default {}" },
} as SfcCompileResult;

function capture(target: string, outcome = "accepted") {
  return {
    schema_version: 2,
    command: "compile-sfc",
    source: {
      path: "Component.vue",
      container: "vue-sfc",
      authored_syntax: "vue-template",
      compiled_syntax: "vue-template",
      template_span: { start: 10, end: 26 },
    },
    target,
    outcome: { kind: outcome, reason: outcome === "accepted" ? null : "legacy selected" },
    observed: { timings: false, remarks: false },
    options: [],
    pages:
      outcome === "accepted"
        ? [
            { level: "l1", step: "parse", text: "<div>hello</div>" },
            { level: "l2", step: "lower", text: "ops=1\n" },
            { level: "l4", step: "emit", text: "export default {}" },
          ]
        : [],
    timings: [],
    remarks: [],
  };
}

describe("same-run product stage feed", () => {
  it("renders only the levels observed by the DOM product compile", () => {
    const negotiated = negotiateProductCapture(capture("dom"), "dom");
    expect(negotiated.ok).toBe(true);
    if (!negotiated.ok) return;
    const ladder = buildProductLadder(negotiated.feed, result);
    expect(ladder.rungs.map((rung) => rung.id)).toEqual(["l1", "l2"]);
    expect(ladder.unplaced).toEqual([]);
    expect(ladder.template).toBe("<div>hello</div>");
    expect(ladder.timeline.map((step) => step.nanos)).toEqual([null, null]);
    expect(source.slice(10, 26)).toBe(ladder.template);
  });

  it("shows target and fallback reason without invented stages", () => {
    const fallback = negotiateProductCapture(capture("ssr", "legacy"), "ssr");
    expect(fallback.ok && fallback.feed.outcome).toEqual({
      kind: "legacy",
      reason: "legacy selected",
    });
    expect(fallback.ok && fallback.feed.pages).toEqual([]);
  });

  it("fails closed on target, version, or impossible fallback pages", () => {
    expect(negotiateProductCapture(capture("ssr"), "dom").ok).toBe(false);
    expect(negotiateProductCapture({ ...capture("dom"), schema_version: 1 }, "dom").ok).toBe(false);
    expect(
      negotiateProductCapture({ ...capture("dom", "legacy"), pages: capture("dom").pages }, "dom")
        .ok,
    ).toBe(false);
  });
});
