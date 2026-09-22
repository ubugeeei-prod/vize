import { describe, expect, it } from "vite-plus/test";
import { mount } from "@vue/test-utils";
import PassTimeline from "./PassTimeline.vue";
import RemarksPanel from "./RemarksPanel.vue";
import FolioDiffView from "./FolioDiffView.vue";
import type { TimelineStep } from "./ladder";
import type { SpolveroRemark } from "./remarks";

describe("PassTimeline timings", () => {
  const steps: TimelineStep[] = [
    {
      key: "s1/parse",
      rung: "s1",
      pass: "parse",
      changed: true,
      producer: true,
      nanos: 5_000,
      remarks: 0,
    },
    {
      key: "s2/lower",
      rung: "s2",
      pass: "lower",
      changed: true,
      producer: true,
      nanos: 15_000,
      remarks: 0,
    },
    {
      key: "s2/hoist-static",
      rung: "s2",
      pass: "hoist-static",
      changed: false,
      producer: false,
      nanos: 0,
      remarks: 5,
    },
    {
      key: "s3/lower",
      rung: "s3",
      pass: "lower",
      changed: true,
      producer: true,
      nanos: 20_000,
      remarks: 0,
    },
  ];

  it("labels each step and splits the strip by measured share", () => {
    const wrapper = mount(PassTimeline, { props: { steps, current: "s2/lower" } });
    expect(wrapper.findAll(".davinci-step-time").map((label) => label.text())).toEqual([
      "5 µs",
      "15 µs",
      "<1 µs",
      "20 µs",
    ]);
    const segments = wrapper.findAll(".davinci-time-segment");
    expect(segments.map((segment) => segment.attributes("style"))).toEqual([
      "width: 12.5%;",
      "width: 37.5%;",
      "width: 0%;",
      "width: 50%;",
    ]);
    expect(segments[1].classes()).toContain("current");
    const badge = wrapper.find(".davinci-step-remarks");
    expect([badge.text(), badge.attributes("title")]).toEqual(["5", "5 optimization remarks"]);
    expect(wrapper.find(".davinci-time-strip").attributes("aria-label")).toBe(
      "Measured compiler work: 40 µs in total",
    );
    expect(wrapper.find(".davinci-timeline-summary").text()).toBe(
      "0 of 1 pass changed the folio, 40 µs total",
    );
    wrapper.unmount();
  });

  it("draws no strip when nothing was measured", () => {
    const unmeasured = steps.map((step) => ({ ...step, nanos: null }));
    const wrapper = mount(PassTimeline, { props: { steps: unmeasured, current: null } });
    expect(wrapper.find(".davinci-time-strip").exists()).toBe(false);
    expect(wrapper.findAll(".davinci-step-time")).toHaveLength(0);
    wrapper.unmount();
  });
});

describe("RemarksPanel", () => {
  it("says so when the passes had nothing to explain", () => {
    const wrapper = mount(RemarksPanel, { props: { remarks: [] } });
    expect(wrapper.find(".davinci-remarks-title").text()).toBe(
      "No optimization remarks for this source",
    );
    expect(wrapper.find(".davinci-remark-list").exists()).toBe(false);
    wrapper.unmount();
  });

  it("lists remarks by kind with their arguments and locates each site", async () => {
    const remarks: SpolveroRemark[] = [
      {
        stage: "s2",
        pass: "hoist-static",
        kind: "applied",
        name: "static-subtree",
        span: { start: 3, end: 9 },
        args: [{ key: "tag", value: "h1" }],
      },
      {
        stage: "s2",
        pass: "hoist-static",
        kind: "missed",
        name: "static-props",
        span: { start: 10, end: 30 },
        args: [
          { key: "tag", value: "p" },
          { key: "blocker", value: "binding" },
        ],
      },
    ];
    const wrapper = mount(RemarksPanel, { props: { remarks } });
    expect(wrapper.find(".davinci-remarks-title").text()).toBe("1 applied, 1 missed");
    const rows = wrapper.findAll(".davinci-remark");
    expect(rows.map((row) => row.classes())).toEqual([
      ["davinci-remark", "applied"],
      ["davinci-remark", "missed"],
    ]);
    expect(rows[1].findAll(".davinci-remark-arg").map((arg) => arg.text())).toEqual([
      "tag=p",
      "blocker=binding",
    ]);
    await rows[1].find("button").trigger("click");
    expect(wrapper.emitted("locate")).toEqual([[remarks[1]]]);
    wrapper.unmount();
  });
});

describe("FolioDiffView", () => {
  it("says so when a pass left the page identical", () => {
    const wrapper = mount(FolioDiffView, {
      props: {
        before: "Lowered",
        after: "v-slot",
        diff: {
          lines: [{ kind: "same", leftLine: 1, rightLine: 1, text: "[disegno]" }],
          stats: { additions: 0, removals: 0, unchanged: 1 },
        },
      },
    });
    expect(wrapper.find(".davinci-diff-summary").text()).toBe(
      "v-slot left the page byte-for-byte identical to Lowered: its product is facts, not tree edits.",
    );
    wrapper.unmount();
  });

  it("marks added and removed lines", () => {
    const wrapper = mount(FolioDiffView, {
      props: {
        before: "Lowered",
        after: "legacy",
        diff: {
          lines: [
            { kind: "remove", leftLine: 5, rightLine: null, text: 'ui.on name="x" @1:2' },
            { kind: "add", leftLine: null, rightLine: 5, text: 'ui.bind name="x" @1:2' },
          ],
          stats: { additions: 1, removals: 1, unchanged: 4 },
        },
      },
    });
    expect(wrapper.find(".davinci-diff-summary").text()).toBe(
      "legacy against Lowered: 1 added, 1 removed, 4 unchanged",
    );
    const rows = wrapper.findAll(".davinci-line");
    expect(rows.map((row) => row.classes())).toEqual([
      ["davinci-line", "diff-remove"],
      ["davinci-line", "diff-add"],
    ]);
    expect(rows.map((row) => row.find(".davinci-diff-mark").text())).toEqual(["−", "+"]);
    wrapper.unmount();
  });
});
