import { describe, expect, it } from "vite-plus/test";
import {
  LADDER_STEP_KEY,
  LADDER_WALK_KEY,
  ladderStepTimings,
  ladderWalkTimings,
  negotiateProfileExport,
} from "./profile";

const wall = (total: number) => ({
  total,
  self: total,
  min: total,
  max: total,
  p50: 0,
  p95: 0,
  p99: 0,
});

const profile = {
  schema_version: 1,
  tool: "vize",
  tool_version: "0.0.0",
  command: "analyze-sfc",
  spans: [
    {
      key: LADDER_STEP_KEY,
      count: 1,
      wall_ns: wall(21000),
      attribution: { stage: "s3", pass: "lower", block: "template" },
    },
    {
      key: LADDER_STEP_KEY,
      count: 1,
      wall_ns: wall(9000),
      attribution: { stage: "s2", pass: "hoist-static", block: "template" },
    },
    {
      key: LADDER_WALK_KEY,
      count: 1,
      wall_ns: wall(9500),
      attribution: { stage: "s2", pass: "hoist-static", block: "template" },
    },
    // Other keys and unattributed buckets are not ladder steps.
    { key: "atelier.dom.template.parse", count: 1, wall_ns: wall(50) },
    { key: LADDER_STEP_KEY, count: 1, wall_ns: wall(7) },
  ],
};

describe("negotiateProfileExport", () => {
  it("accepts schema version 1 and reads the ladder step timings", () => {
    const negotiated = negotiateProfileExport(profile);
    expect(negotiated.ok).toBe(true);
    if (!negotiated.ok) return;
    expect([...ladderStepTimings(negotiated.profile)]).toEqual([
      ["s3/lower", 21000],
      ["s2/hoist-static", 9000],
    ]);
  });

  it("reads walks under the timing observer's key, by lead pass", () => {
    const negotiated = negotiateProfileExport(profile);
    if (!negotiated.ok) throw new Error(negotiated.error);
    expect(LADDER_WALK_KEY).toBe("davinci.pass.walk");
    expect([...ladderWalkTimings(negotiated.profile)]).toEqual([["s2/hoist-static", 9500]]);
  });

  it("refuses other versions, missing profiles and malformed spans", () => {
    expect(negotiateProfileExport({ ...profile, schema_version: 2 })).toEqual({
      ok: false,
      error: "Profile export schema_version 2 is not supported; this view reads version 1.",
    });
    expect(negotiateProfileExport(null)).toEqual({
      ok: false,
      error: "The compiler result carries no profile.",
    });
    expect(negotiateProfileExport({ ...profile, spans: [{ key: "x" }] })).toEqual({
      ok: false,
      error: "The profile export's spans do not match the schema.",
    });
  });
});
