import { describe, expect, it } from "vite-plus/test";
import type { SpolveroFeed } from "../../wasm/types/spolvero";
import { buildLadder } from "./ladder";

// The feed `analyzeSfc` returns for `<div>{{ msg }}</div>`, byte-for-byte as
// the Rust `wasm::tests_spolvero` pin states it.
const S2 = `[disegno]
ops=2

[disegno.ops]
ui.element div @3:23
  ui.interpolation js("msg" @11:14) @8:17

`;
const S3 = `[s3-folio]
phase=built

[s3-folio.regions]
id=0 parent=- owner=- span=3:23
id=1 parent=0 owner=0 span=8:17

[s3-folio.ops]
id=0 kind=impeto.insert-node region=0 effect=- span=3:23
id=1 kind=impeto.set-text region=1 effect=0 span=8:17

[s3-folio.effects]
id=0 owner=1 region=1 span=8:17

`;
const PARTITION = `[s3-partition-folio]

[s3-partition-folio.ops]
op=0 kind=static span=3:23
op=1 kind=dynamic span=8:17

`;
const VALUES = `[s3-values-folio]

[s3-values-folio.operands]
operand=[0,"tag",null,null,null,"literal","div","",3,23]
operand=[0,"namespace",null,null,null,"literal","html","",3,23]
operand=[1,"text",null,null,null,"js","msg","",11,14]

`;

function feed(extra: SpolveroFeed["pages"] = []): SpolveroFeed {
  const path = "Component.vue";
  return {
    schema_version: 1,
    command: "analyze-sfc",
    pages: [
      { path, stage: "s1", pass: "parse", text: "\n  <div>{{ msg }}</div>\n" },
      { path, stage: "s2", pass: "lower", text: S2 },
      { path, stage: "s2", pass: "hoist-static", text: S2 },
      { path, stage: "s3", pass: "lower", text: S3 },
      { path, stage: "s3-partition", pass: "lower", text: PARTITION },
      { path, stage: "s3-values", pass: "lower", text: VALUES },
      ...extra,
    ],
  };
}

describe("buildLadder", () => {
  it("places every page on its rung with facts read off the pages", () => {
    const ladder = buildLadder(feed(), "Component.vue");
    expect(
      ladder.rungs.map(({ id, ordinal, name, facts, pages }) => ({
        id,
        ordinal,
        name,
        facts,
        pages: pages.map(({ key, kind, label }) => ({ key, kind, label })),
      })),
    ).toEqual([
      {
        id: "s1",
        ordinal: "S1",
        name: "Surface",
        facts: ["2 lines"],
        pages: [{ key: "s1/parse", kind: "surface", label: "Surface" }],
      },
      {
        id: "s2",
        ordinal: "S2",
        name: "Disegno",
        facts: ["2 ops", "1 pass"],
        pages: [
          { key: "s2/lower", kind: "disegno", label: "Lowered" },
          { key: "s2/hoist-static", kind: "disegno", label: "hoist-static" },
        ],
      },
      {
        id: "s3",
        ordinal: "S3",
        name: "Impeto",
        facts: ["2 ops", "1 dynamic"],
        pages: [
          { key: "s3/lower", kind: "impeto", label: "Graph" },
          { key: "s3-partition/lower", kind: "partition", label: "Partition" },
          { key: "s3-values/lower", kind: "values", label: "Values" },
        ],
      },
    ]);
    expect(ladder.template).toBe("\n  <div>{{ msg }}</div>\n");
    expect(ladder.unplaced).toEqual([]);
  });

  it("lists steps in run order and marks which passes changed the folio", () => {
    const changed = { path: "Component.vue", stage: "s2", pass: "legacy", text: `${S2}x` };
    const timings = new Map([
      ["s2/lower", 5000],
      ["s2/legacy", 9000],
    ]);
    const ladder = buildLadder(feed([changed]), undefined, timings);
    const step = (
      pass: string,
      rung: string,
      changed: boolean,
      producer: boolean,
      nanos = null,
    ) => ({
      key: `${rung}/${pass}`,
      rung,
      pass,
      changed,
      producer,
      nanos,
    });
    expect(ladder.timeline).toEqual([
      step("parse", "s1", true, true),
      { ...step("lower", "s2", true, true), nanos: 5000 },
      step("hoist-static", "s2", false, false),
      { ...step("legacy", "s2", true, false), nanos: 9000 },
      step("lower", "s3", true, true),
    ]);
  });

  it("keeps unknown stages visible and filters by file", () => {
    const other = { path: "Other.vue", stage: "s1", pass: "parse", text: "<p />" };
    const future = { path: "Component.vue", stage: "s4-plan", pass: "lower", text: "[plan]\n" };
    const ladder = buildLadder(feed([other, future]), "Component.vue");
    expect(ladder.unplaced).toEqual(["s4-plan"]);
    expect(ladder.rungs[0].pages).toHaveLength(1);
  });

  it("files the provenance page under S2 without counting it as a pass or step", () => {
    const provenance = {
      path: "Component.vue",
      stage: "s2-provenance",
      pass: "transform",
      text: "[s2-provenance-folio]\n\n[s2-provenance-folio.records]\n\n",
    };
    const ladder = buildLadder(feed([provenance]));
    const s2 = ladder.rungs[1];
    expect(s2.pages.map(({ key, kind, label }) => [key, kind, label])).toEqual([
      ["s2/lower", "disegno", "Lowered"],
      ["s2/hoist-static", "disegno", "hoist-static"],
      ["s2-provenance/transform", "provenance", "Provenance"],
    ]);
    expect(s2.facts).toEqual(["2 ops", "1 pass"]);
    expect(ladder.timeline.map(({ key }) => key)).not.toContain("s2-provenance/transform");
  });
});
