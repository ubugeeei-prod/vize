import { describe, expect, it } from "vite-plus/test";
import { SPOLVERO_FEED_SCHEMA_VERSION, layerId, negotiateSpolveroFeed } from "./spolvero";

const page = { path: "Component.vue", stage: "s1", pass: "parse", text: "<p />" };

describe("negotiateSpolveroFeed", () => {
  it("accepts the committed schema version with well-formed pages", () => {
    const raw = { schema_version: 1, command: "analyze-sfc", pages: [page] };
    expect(negotiateSpolveroFeed(raw)).toEqual({ ok: true, feed: raw });
    expect(SPOLVERO_FEED_SCHEMA_VERSION).toBe(1);
  });

  it("refuses another schema version before reading pages", () => {
    // Pages of an unknown shape must not be inspected once the version fails.
    const raw = { schema_version: 2, command: "analyze-sfc", pages: "not read" };
    expect(negotiateSpolveroFeed(raw)).toEqual({
      ok: false,
      error: "Spolvero feed schema_version 2 is not supported; this view renders version 1.",
    });
  });

  it("refuses a missing feed, a missing version and malformed pages", () => {
    expect(negotiateSpolveroFeed(undefined)).toEqual({
      ok: false,
      error: "The compiler result carries no Spolvero feed.",
    });
    expect(negotiateSpolveroFeed({ command: "x", pages: [] })).toEqual({
      ok: false,
      error: "The Spolvero feed has no numeric schema_version.",
    });
    expect(
      negotiateSpolveroFeed({
        schema_version: 1,
        command: "analyze-sfc",
        pages: [page, { ...page, text: 3 }],
      }),
    ).toEqual({ ok: false, error: "Spolvero feed page 1 does not match the schema." });
    expect(negotiateSpolveroFeed({ schema_version: 1, pages: [] })).toEqual({
      ok: false,
      error: "The Spolvero feed is missing its command or pages.",
    });
  });

  it("accepts the additive remarks member and refuses a malformed remark", () => {
    const remark = {
      path: "Component.vue",
      stage: "s2",
      pass: "hoist-static",
      kind: "applied",
      name: "static-subtree",
      span: { start: 3, end: 9 },
      args: [{ key: "tag", value: "h1" }],
    };
    const raw = { schema_version: 1, command: "analyze-sfc", pages: [page], remarks: [remark] };
    expect(negotiateSpolveroFeed(raw)).toEqual({ ok: true, feed: raw });
    expect(
      negotiateSpolveroFeed({ ...raw, remarks: [remark, { ...remark, kind: "maybe" }] }),
    ).toEqual({ ok: false, error: "Spolvero feed remark 1 does not match the schema." });
    expect(negotiateSpolveroFeed({ ...raw, remarks: "none" })).toEqual({
      ok: false,
      error: "Spolvero feed remark 0 does not match the schema.",
    });
  });
});

describe("layerId", () => {
  it("maps display layers without modifying negotiated wire records", () => {
    expect(
      ["s0", "s1", "s2-plan", "s3-values", "s4", "s2-to-s3", "custom", "custom-s2", "l2"].map(
        layerId,
      ),
    ).toEqual(["l0", "l1", "l2-plan", "l3-values", "l4", "l2-to-l3", "custom", "custom-s2", "l2"]);
    const raw = { schema_version: 1, command: "analyze-sfc", pages: [page] };
    negotiateSpolveroFeed(raw);
    expect(raw.pages[0].stage).toBe("s1");
  });
});
