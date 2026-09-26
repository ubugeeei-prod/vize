import { describe, expect, it } from "vite-plus/test";
import type { SpolveroFeed } from "../../wasm/types/spolvero";
import { buildLadder } from "./ladder";

// The feed `analyzeSfc` returns for `<div>{{ msg }}</div>`, byte-for-byte as
// the Rust `wasm::tests_spolvero` pin states it.
const L2 = `[disegno]
ops=2

[disegno.ops]
ui.element div @3:23
  ui.interpolation js("msg" @11:14) @8:17

`;
const L3 = `[s3-folio]
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
      { path, stage: "s2", pass: "lower", text: L2 },
      { path, stage: "s2", pass: "hoist-static", text: L2 },
      { path, stage: "s3", pass: "lower", text: L3 },
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
        id: "l1",
        ordinal: "L1",
        name: "Surface",
        facts: ["2 lines"],
        pages: [{ key: "l1/parse", kind: "surface", label: "Surface" }],
      },
      {
        id: "l2",
        ordinal: "L2",
        name: "Disegno",
        facts: ["2 ops", "1 pass"],
        pages: [
          { key: "l2/lower", kind: "disegno", label: "Lowered" },
          { key: "l2/hoist-static", kind: "disegno", label: "hoist-static" },
        ],
      },
      {
        id: "l3",
        ordinal: "L3",
        name: "Impeto",
        facts: ["2 ops", "1 dynamic"],
        pages: [
          { key: "l3/lower", kind: "impeto", label: "Graph" },
          { key: "l3-partition/lower", kind: "partition", label: "Partition" },
          { key: "l3-values/lower", kind: "values", label: "Values" },
        ],
      },
    ]);
    expect(ladder.template).toBe("\n  <div>{{ msg }}</div>\n");
    expect(ladder.unplaced).toEqual([]);
  });

  it("lists steps in run order and marks which passes changed the folio", () => {
    const changed = { path: "Component.vue", stage: "s2", pass: "legacy", text: `${L2}x` };
    const timings = new Map([
      ["l2/lower", 5000],
      ["l2/legacy", 9000],
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
      remarks: 0,
      walk: null,
    });
    expect(ladder.timeline).toEqual([
      step("parse", "l1", true, true),
      { ...step("lower", "l2", true, true), nanos: 5000 },
      step("hoist-static", "l2", false, false),
      { ...step("legacy", "l2", true, false), nanos: 9000 },
      step("lower", "l3", true, true),
    ]);
  });

  it("keeps unknown stages visible and filters by file", () => {
    const other = { path: "Other.vue", stage: "s1", pass: "parse", text: "<p />" };
    const future = { path: "Component.vue", stage: "s4-plan", pass: "lower", text: "[plan]\n" };
    const ladder = buildLadder(feed([other, future]), "Component.vue");
    expect(ladder.unplaced).toEqual(["l4-plan"]);
    expect(ladder.rungs[0].pages).toHaveLength(1);
  });

  it("files the provenance page under L2 without counting it as a pass or step", () => {
    const provenance = {
      path: "Component.vue",
      stage: "s2-provenance",
      pass: "transform",
      text: "[s2-provenance-folio]\n\n[s2-provenance-folio.records]\n\n",
    };
    const ladder = buildLadder(feed([provenance]));
    const s2 = ladder.rungs[1];
    expect(s2.pages.map(({ key, kind, label }) => [key, kind, label])).toEqual([
      ["l2/lower", "disegno", "Lowered"],
      ["l2/hoist-static", "disegno", "hoist-static"],
      ["l2-provenance/transform", "provenance", "Provenance"],
    ]);
    expect(s2.facts).toEqual(["2 ops", "1 pass"]);
    expect(ladder.timeline.map(({ key }) => key)).not.toContain("l2-provenance/transform");
  });

  it("reads the transform plan's walks off the plan page and times them", () => {
    const plan = {
      path: "Component.vue",
      stage: "s2-plan",
      pass: "transform",
      text: "[fusion-plan-folio]\nstage=s2\nwalks=1\n\n[fusion-plan-folio.passes]\nwalk=0 pass=hoist-static kind=optional fusability=fusable\n\n",
    };
    const pages = feed().pages;
    const ladder = buildLadder(
      { ...feed(), pages: [...pages.slice(0, 2), plan, ...pages.slice(2)] },
      "Component.vue",
      new Map([["l2/hoist-static", 4_000]]),
      new Map([["l2/hoist-static", 4_000]]),
    );
    const s2 = ladder.rungs[1];
    expect(s2.pages.map(({ key, kind, label }) => [key, kind, label])).toEqual([
      ["l2/lower", "disegno", "Lowered"],
      ["l2-plan/transform", "plan", "Plan"],
      ["l2/hoist-static", "disegno", "hoist-static"],
    ]);
    // The rail keeps two facts; the walk count is the timeline summary's.
    expect(s2.facts).toEqual(["2 ops", "1 pass"]);
    // The plan describes steps; it is not one. Only passes carry a walk.
    expect(ladder.timeline.map(({ key, walk }) => [key, walk])).toEqual([
      ["l1/parse", null],
      ["l2/lower", null],
      ["l2/hoist-static", 0],
      ["l3/lower", null],
    ]);
    expect(ladder.walks).toEqual([
      { index: 0, passes: ["hoist-static"], fusable: true, nanos: 4_000 },
    ]);
    expect(buildLadder(feed()).walks).toEqual([]);
  });

  it("takes this file's remarks from the feed and counts them per step", () => {
    const remark = (path: string | null, pass: string) => ({
      path,
      stage: "s2",
      pass,
      kind: "missed" as const,
      name: "static-subtree",
      span: { start: 3, end: 23 },
      args: [{ key: "tag", value: "div" }],
    });
    const withRemarks: SpolveroFeed = {
      ...feed(),
      remarks: [remark("Component.vue", "hoist-static"), remark("Other.vue", "hoist-static")],
    };
    const ladder = buildLadder(withRemarks, "Component.vue");
    expect(ladder.remarks).toEqual([
      {
        stage: "l2",
        pass: "hoist-static",
        kind: "missed",
        name: "static-subtree",
        span: { start: 3, end: 23 },
        args: [{ key: "tag", value: "div" }],
      },
    ]);
    expect(ladder.timeline.map(({ key, remarks }) => [key, remarks])).toEqual([
      ["l1/parse", 0],
      ["l2/lower", 0],
      ["l2/hoist-static", 1],
      ["l3/lower", 0],
    ]);
    expect(buildLadder(feed()).remarks).toEqual([]);
  });
});
