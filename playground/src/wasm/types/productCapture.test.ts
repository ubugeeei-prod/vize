import { describe, expect, it } from "vite-plus/test";
import { negotiateProductCapture } from "./productCapture";

function capture(kind = "accepted") {
  return {
    schema_version: 2,
    command: "compile-sfc",
    target: "dom",
    source: {
      path: "Component.vue",
      container: "vue-sfc",
      authored_syntax: "vue-template",
      compiled_syntax: "vue-template",
      template_span: { start: 10, end: 18 },
    },
    outcome: { kind, reason: kind === "accepted" ? null : "compatibility selected" },
    observed: { timings: false, remarks: false },
    options: [],
    pages: [],
    timings: [],
    remarks: [],
  };
}

describe("fallible product inspection", () => {
  it("keeps successful feeds unchanged and failures outside valid pages", () => {
    const original = capture();
    expect(negotiateProductCapture(original, "dom")).toEqual({ ok: true, feed: original });
    expect(Object.hasOwn(original, "unavailable")).toBe(false);
    const unavailable = [
      { level: "l2", step: "lower", reason: "native binding unsupported at bytes 4..9: λ" },
    ];
    const raw = { ...original, unavailable };
    expect(negotiateProductCapture(raw, "dom")).toEqual({ ok: true, feed: raw });
    expect(raw.pages).toEqual([]);
  });
  it("refuses failed inspections on nonaccepted product outcomes", () => {
    for (const kind of ["legacy", "unavailable", "rejected"]) {
      expect(
        negotiateProductCapture(
          {
            ...capture(kind),
            unavailable: [{ level: "l2", step: "lower", reason: "native binding unsupported" }],
          },
          "dom",
        ).ok,
      ).toBe(false);
    }
  });
  it("refuses malformed failure records and fake page substitutions", () => {
    for (const unavailable of [
      "none",
      [{ level: "l2", step: "lower", reason: 3 }],
      [{ level: "l2", step: "lower", reason: "refused", text: "fake page" }],
      [{ level: "l7", step: "lower", reason: "refused" }],
    ]) {
      expect(negotiateProductCapture({ ...capture(), unavailable }, "dom").ok).toBe(false);
    }
  });
});
