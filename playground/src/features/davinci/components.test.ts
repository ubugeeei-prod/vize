import { describe, expect, it } from "vite-plus/test";
import { mount } from "@vue/test-utils";
import StageRail from "./StageRail.vue";
import PassTimeline from "./PassTimeline.vue";
import FolioView from "./FolioView.vue";
import { folioLines } from "./folioLines";
import type { Rung, TimelineStep } from "./ladder";

const rungs: Rung[] = [
  { id: "l1", ordinal: "L1", name: "Surface", facts: ["2 lines"], pages: [] },
  { id: "l2", ordinal: "L2", name: "Disegno", facts: ["2 ops", "1 pass"], pages: [] },
  { id: "l3", ordinal: "L3", name: "Impeto", facts: [], pages: [] },
];

describe("StageRail", () => {
  it("shows every stage with its facts and reports the chosen one", async () => {
    const wrapper = mount(StageRail, { props: { rungs, selected: "l2" } });
    const stations = wrapper.findAll(".davinci-station");
    expect(stations.map((station) => station.attributes("data-stage"))).toEqual([
      "l1",
      "l2",
      "l3",
      "l4",
    ]);
    expect(stations.map((station) => station.text())).toEqual([
      "L1Surface2 lines",
      "L2Disegno2 ops, 1 pass",
      "L3Impetono page",
      "L4OutputDOM, Vapor, SSR",
    ]);
    expect(stations[1].attributes("aria-pressed")).toBe("true");
    expect(wrapper.findAll(".davinci-pounce")).toHaveLength(3);
    await stations[3].trigger("click");
    expect(wrapper.emitted("select")).toEqual([["l4"]]);
    wrapper.unmount();
  });
});

describe("PassTimeline", () => {
  const steps: TimelineStep[] = [
    {
      key: "l2/lower",
      rung: "l2",
      pass: "lower",
      changed: true,
      producer: true,
      nanos: null,
      remarks: 0,
      walk: null,
    },
    {
      key: "l2/v-slot",
      rung: "l2",
      pass: "v-slot",
      changed: false,
      producer: false,
      nanos: null,
      remarks: 0,
      walk: null,
    },
    {
      key: "l2/legacy",
      rung: "l2",
      pass: "legacy",
      changed: true,
      producer: false,
      nanos: null,
      remarks: 0,
      walk: null,
    },
  ];

  it("marks producers and changed passes and summarizes them", async () => {
    const wrapper = mount(PassTimeline, { props: { steps, walks: [], current: "l2/v-slot" } });
    const buttons = wrapper.findAll(".davinci-step");
    expect(buttons.map((button) => button.classes().filter((c) => c !== "davinci-step"))).toEqual([
      ["rung-l2", "producer", "changed"],
      ["rung-l2", "current"],
      ["rung-l2", "changed"],
    ]);
    expect(buttons[1].attributes("title")).toBe(
      "v-slot left the L2 folio unchanged (its product is facts)",
    );
    expect(wrapper.find(".davinci-timeline-summary").text()).toBe(
      "1 of 2 passes changed the folio",
    );
    await buttons[2].trigger("click");
    expect(wrapper.emitted("select")).toEqual([[steps[2]]]);
    wrapper.unmount();
  });
});

describe("FolioView", () => {
  const lines = folioLines(
    "disegno",
    '[disegno]\nops=2\n\n[disegno.ops]\nui.element div @3:23\n  ui.interpolation js("msg" @11:14) @8:17\n\n',
  );

  it("renders tokens and links only lines that carry a span", async () => {
    const wrapper = mount(FolioView, {
      props: { lines, kind: "disegno", selected: 5, linked: [4] },
    });
    const rows = wrapper.findAll(".davinci-line");
    expect(rows.map((row) => row.find(".davinci-code").text())).toEqual([
      "[disegno]",
      "ops=2",
      "",
      "[disegno.ops]",
      "ui.element div @3:23",
      'ui.interpolation js("msg" @11:14) @8:17',
      "",
    ]);
    expect(rows[4].classes()).toContain("linked");
    expect(rows[5].classes()).toContain("selected");
    expect(rows[5].attributes("title")).toBe("Authored bytes 8–17");
    expect(rows[4].find(".tk-tag").text()).toBe("div");

    // Only span-carrying lines are interactive; keyboard mirrors the mouse.
    expect(rows.map((row) => row.attributes("tabindex") ?? null)).toEqual([
      null,
      null,
      null,
      null,
      "0",
      "0",
      null,
    ]);
    await rows[0].trigger("click");
    await rows[4].trigger("mouseenter");
    await rows[4].trigger("mouseleave");
    await rows[4].trigger("focus");
    await rows[4].trigger("click");
    await rows[5].trigger("keydown", { key: "Enter" });
    expect(wrapper.emitted("hover")).toEqual([[4], [null], [4]]);
    expect(wrapper.emitted("select")).toEqual([[4], [null]]);
    wrapper.unmount();
  });

  it("marks the lines it is given in the gutter", () => {
    const wrapper = mount(FolioView, {
      props: {
        lines,
        kind: "disegno",
        selected: null,
        linked: [],
        marks: new Map([
          [4, "static"],
          [5, "dynamic"],
        ]),
      },
    });
    const marks = wrapper.findAll(".davinci-mark");
    expect(marks.map((mark) => mark.classes().filter((c) => c !== "davinci-mark"))).toEqual([
      [],
      [],
      [],
      [],
      ["static"],
      ["dynamic"],
      [],
    ]);
    wrapper.unmount();
  });
});
