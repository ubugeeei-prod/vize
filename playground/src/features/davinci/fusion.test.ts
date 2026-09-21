import { describe, expect, it } from "vite-plus/test";
import { mount } from "@vue/test-utils";
import PassTimeline from "./PassTimeline.vue";
import { describeWalk, parseFusionPlan, planWalks, type TimelineWalk } from "./fusion";
import type { TimelineStep } from "./ladder";

// Two fusable runs around a barrier, as the compiler prints the plan
// (the Rust `fusion_plan_folio` pin states the same page).
const PLAN = `[fusion-plan-folio]
stage=s2
walks=3

[fusion-plan-folio.passes]
walk=0 pass=normalize kind=optional fusability=fusable
walk=0 pass=fold kind=optional fusability=fusable
walk=1 pass=check kind=mandatory-diagnostic fusability=barrier
walk=2 pass=tidy kind=optional fusability=fusable

`;

describe("parseFusionPlan", () => {
  it("reads every pass with its walk, kind and fusability", () => {
    expect(parseFusionPlan(PLAN)).toEqual({
      stage: "s2",
      walks: 3,
      passes: [
        { walk: 0, pass: "normalize", kind: "optional", fusability: "fusable" },
        { walk: 0, pass: "fold", kind: "optional", fusability: "fusable" },
        { walk: 1, pass: "check", kind: "mandatory-diagnostic", fusability: "barrier" },
        { walk: 2, pass: "tidy", kind: "optional", fusability: "fusable" },
      ],
    });
    expect(parseFusionPlan("[fusion-plan-folio]\nstage=s2\nwalks=0\n\n")).toEqual({
      stage: "s2",
      walks: 0,
      passes: [],
    });
  });

  it("refuses other pages and malformed records", () => {
    expect(parseFusionPlan("[disegno]\nops=0\n\n")).toBeNull();
    expect(parseFusionPlan("[fusion-plan-folio]\nwalks=1\n\n")).toBeNull();
    expect(parseFusionPlan(PLAN.replace("fusability=barrier", "fusability=fused"))).toBeNull();
  });
});

describe("planWalks", () => {
  it("groups fused passes into one walk timed by its lead pass", () => {
    const walks = planWalks(
      parseFusionPlan(PLAN)!,
      new Map([
        ["s2/normalize", 7_000],
        ["s2/check", 2_000],
        // A pass that does not lead a walk has no walk span of its own.
        ["s2/fold", 1],
      ]),
    );
    expect(walks).toEqual([
      { index: 0, passes: ["normalize", "fold"], fusable: true, nanos: 7_000 },
      { index: 1, passes: ["check"], fusable: false, nanos: 2_000 },
      { index: 2, passes: ["tidy"], fusable: true, nanos: null },
    ]);
    expect(walks.map(describeWalk)).toEqual(["2 passes fused", "barrier", "fusable"]);
  });
});

describe("PassTimeline walks", () => {
  const step = (pass: string, walk: number | null, producer = false): TimelineStep => ({
    key: `s2/${pass}`,
    rung: "s2",
    pass,
    changed: producer,
    producer,
    nanos: null,
    remarks: 0,
    walk,
  });
  const steps = [
    step("lower", null, true),
    step("normalize", 0),
    step("fold", 0),
    step("check", 1),
  ];
  const walks: TimelineWalk[] = [
    { index: 0, passes: ["normalize", "fold"], fusable: true, nanos: 7_000 },
    { index: 1, passes: ["check"], fusable: false, nanos: null },
  ];

  it("brackets the passes of each walk and names how it ran", async () => {
    const wrapper = mount(PassTimeline, { props: { steps, walks, current: null } });
    const groups = wrapper.findAll(".davinci-walk");
    expect(groups.map((group) => group.classes())).toEqual([
      ["davinci-walk", "fusable"],
      ["davinci-walk"],
    ]);
    expect(
      groups.map((group) => group.findAll(".davinci-step-pass").map((pass) => pass.text())),
    ).toEqual([["normalize", "fold"], ["check"]]);
    const label = (group: (typeof groups)[number]) =>
      group.findAll(".davinci-walk-label > span").map((part) => part.text());
    expect(groups.map(label)).toEqual([
      ["walk 1", "2 passes fused", "7 µs"],
      ["walk 2", "barrier"],
    ]);
    expect(groups[0].find(".davinci-walk-steps").attributes("aria-label")).toBe(
      "Walk 1 over the S2 tree (2 passes fused): normalize, fold, 7 µs",
    );
    expect(wrapper.find(".davinci-timeline-summary").text()).toBe(
      "0 of 3 passes changed the folio in 2 walks",
    );
    await groups[0].findAll(".davinci-step")[1].trigger("click");
    expect(wrapper.emitted("select")).toEqual([[steps[2]]]);
    wrapper.unmount();
  });
});
