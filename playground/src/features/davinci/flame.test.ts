import { describe, expect, it } from "vite-plus/test";
import { mount } from "@vue/test-utils";
import FlameView from "./FlameView.vue";
import { flameGraph, flameTrend } from "./flame";
import { LADDER_STEP_KEY, LADDER_WALK_KEY, type ProfileExport } from "../../wasm/types/profile";

const span = (key: string, stage: string | null, pass: string, total: number, count = 1) => ({
  key,
  count,
  wall_ns: { total, self: total, min: total, max: total, p50: 0, p95: 0, p99: 0 },
  attribution: stage === null ? undefined : { stage, pass, block: "template" },
});

const run = (spans: ProfileExport["spans"]): ProfileExport => ({
  schema_version: 1,
  tool: "vize",
  tool_version: "0.0.0",
  command: "analyze-sfc",
  spans,
});

// Ranked by cost, as the exporter writes them.
const PROFILE = run([
  span(LADDER_STEP_KEY, "s3", "lower", 21_000),
  span(LADDER_STEP_KEY, "s2", "hoist-static", 17_000),
  // A walk overlaps its passes' steps; the graph never adds it in.
  span(LADDER_WALK_KEY, "s2", "hoist-static", 17_000),
  span(LADDER_STEP_KEY, "s2", "v-slot", 9_000),
  span(LADDER_STEP_KEY, "s2", "lower", 5_000),
  span(LADDER_STEP_KEY, "s1", "parse", 1_000),
]);

const shape = (profile: ProfileExport, baseline: ProfileExport | null = null) =>
  flameGraph(profile, LADDER_STEP_KEY, baseline).frames.map((frame) => [
    frame.path.join("/"),
    frame.depth,
    frame.start,
    frame.nanos,
    frame.baseline,
  ]);

describe("flameGraph", () => {
  it("stacks stage > pass > block by time, siblings sorted by name", () => {
    expect(flameGraph(PROFILE, LADDER_STEP_KEY).total).toBe(53_000);
    expect(shape(PROFILE)).toEqual([
      ["s1", 0, 0, 1_000, null],
      ["s1/parse", 1, 0, 1_000, null],
      ["s1/parse/template", 2, 0, 1_000, null],
      ["s2", 0, 1_000, 31_000, null],
      ["s2/hoist-static", 1, 1_000, 17_000, null],
      ["s2/hoist-static/template", 2, 1_000, 17_000, null],
      ["s2/lower", 1, 18_000, 5_000, null],
      ["s2/lower/template", 2, 18_000, 5_000, null],
      ["s2/v-slot", 1, 23_000, 9_000, null],
      ["s2/v-slot/template", 2, 23_000, 9_000, null],
      ["s3", 0, 32_000, 21_000, null],
      ["s3/lower", 1, 32_000, 21_000, null],
      ["s3/lower/template", 2, 32_000, 21_000, null],
    ]);
  });

  it("aggregates repeated calls and keeps unattributed spans visible", () => {
    const flame = flameGraph(
      run([span("k", "s2", "p", 4_000, 2), span("k", "s2", "p", 6_000, 3), span("k", null, "", 5)]),
      "k",
    );
    expect(flame.frames.map(({ path, nanos, count }) => [path.join("/"), nanos, count])).toEqual([
      ["(unattributed)", 5, 1],
      ["(unattributed)/(unattributed)", 5, 1],
      ["(unattributed)/(unattributed)/(unattributed)", 5, 1],
      ["s2", 10_000, 5],
      ["s2/p", 10_000, 5],
      ["s2/p/template", 10_000, 5],
    ]);
  });

  it("compares every frame with a pinned baseline run", () => {
    const baseline = run([
      span(LADDER_STEP_KEY, "s2", "hoist-static", 8_000),
      span(LADDER_STEP_KEY, "s2", "v-slot", 8_500),
      span(LADDER_STEP_KEY, "s2", "lower", 5_000),
      span(LADDER_STEP_KEY, "s1", "parse", 2_000),
    ]);
    const frames = flameGraph(PROFILE, LADDER_STEP_KEY, baseline).frames.filter(
      (frame) => frame.depth < 2,
    );
    expect(
      frames.map((frame) => [frame.path.join("/"), frame.baseline, flameTrend(frame)]),
    ).toEqual([
      ["s1", 2_000, "faster"],
      ["s1/parse", 2_000, "faster"],
      ["s2", 21_500, "slower"],
      ["s2/hoist-static", 8_000, "slower"],
      ["s2/lower", 5_000, "same"],
      ["s2/v-slot", 8_500, "same"],
      ["s3", null, "new"],
      ["s3/lower", null, "new"],
    ]);
  });
});

describe("FlameView", () => {
  it("lays frames out by share and opens what a frame names", async () => {
    const wrapper = mount(FlameView, { props: { profile: PROFILE, baseline: null } });
    const rows = wrapper.findAll(".davinci-flame-row");
    expect(rows.map((row) => row.find(".davinci-flame-level").text())).toEqual([
      "Stage",
      "Pass",
      "Block",
    ]);
    const stages = rows[0].findAll(".davinci-flame-frame");
    expect(stages.map((frame) => frame.find(".davinci-flame-name").text())).toEqual([
      "s1",
      "s2",
      "s3",
    ]);
    // Compare through the engine's own CSS serializer (Chromium rounds
    // percentages; happy-dom keeps every digit), so the oracle stays exact.
    const probe = document.createElement("div");
    probe.style.left = `${(1_000 / 53_000) * 100}%`;
    probe.style.width = `${(31_000 / 53_000) * 100}%`;
    const placed = (stages[1].element as HTMLElement).style;
    expect([placed.left, placed.width]).toEqual([probe.style.left, probe.style.width]);
    expect(stages[1].attributes("title")).toBe("s2: 31 µs, 58% of the run, 3 calls");
    expect(stages[1].classes()).toEqual(["davinci-flame-frame", "rung-s2"]);
    await rows[1].findAll(".davinci-flame-frame")[1].trigger("click");
    expect(wrapper.emitted("select")).toEqual([[["s2", "hoist-static"]]]);
    await wrapper.find(".davinci-flame-bar button").trigger("click");
    expect(wrapper.emitted("pin")).toEqual([[]]);
    expect(wrapper.find(".davinci-flame-legend").exists()).toBe(false);
    wrapper.unmount();
  });

  it("colours frames by trend against the baseline and can drop it", async () => {
    const baseline = run([span(LADDER_STEP_KEY, "s1", "parse", 2_000)]);
    const wrapper = mount(FlameView, { props: { profile: PROFILE, baseline } });
    const stages = wrapper.findAll(".davinci-flame-row")[0].findAll(".davinci-flame-frame");
    expect(stages.map((frame) => frame.classes().slice(-1)[0])).toEqual([
      "trend-faster",
      "trend-new",
      "trend-new",
    ]);
    expect(stages[0].attributes("title")).toBe("s1: 1 µs, 2% of the run, 1 call; baseline 2 µs");
    const buttons = wrapper.findAll(".davinci-flame-bar button");
    expect(buttons.map((button) => button.text())).toEqual([
      "Pin this run instead",
      "Clear baseline",
    ]);
    await buttons[1].trigger("click");
    expect(wrapper.emitted("unpin")).toEqual([[]]);
    expect(wrapper.find(".davinci-flame-legend").exists()).toBe(true);
    wrapper.unmount();
  });

  it("opens a --profile-json file on its heaviest key, and goes back", async () => {
    const build = run([
      span("compile.template", "s2", "lower", 900),
      span("davinci.pass.walk", "s2", "hoist-static", 4_000),
      span("davinci.pass.walk", "s2", "v-slot", 3_000),
    ]);
    const wrapper = mount(FlameView, { props: { profile: PROFILE, baseline: PROFILE } });
    const input = wrapper.find<HTMLInputElement>("#davinci-flame-file");
    // Reading a File is asynchronous in every engine: wait for the view to settle.
    const choose = async (text: string, settled: () => boolean) => {
      const file = new File([text], "build.json", { type: "application/json" });
      Object.defineProperty(input.element, "files", { value: [file], configurable: true });
      await input.trigger("change");
      for (let tries = 0; tries < 100 && !settled(); tries += 1) {
        await new Promise((resolve) => setTimeout(resolve, 10));
      }
    };

    await choose("{ not json", () => wrapper.find("[role=alert]").exists());
    expect(wrapper.find("[role=alert]").text()).toBe("build.json: The file is not JSON.");

    await choose(JSON.stringify(build), () => wrapper.find("#davinci-flame-key").exists());
    expect(wrapper.find("[role=alert]").exists()).toBe(false);
    expect(wrapper.find(".davinci-flame-bar p").text()).toBe(
      "Showing build.json (tool vize, command analyze-sfc)",
    );
    const select = wrapper.find<HTMLSelectElement>("#davinci-flame-key");
    expect(select.findAll("option").map((option) => option.text())).toEqual([
      "davinci.pass.walk",
      "compile.template",
    ]);
    const names = () => wrapper.findAll(".davinci-flame-name").map((name) => name.text());
    expect(names()).toEqual(["s2", "hoist-static", "v-slot", "template", "template"]);
    // An opened file is not compared with this run's baseline or linked to pages.
    expect(wrapper.find(".davinci-flame-legend").exists()).toBe(false);
    await wrapper.findAll(".davinci-flame-frame")[1].trigger("click");
    expect(wrapper.emitted("select")).toBeUndefined();

    await select.setValue("compile.template");
    expect(names()).toEqual(["s2", "lower", "template"]);
    await wrapper.find(".davinci-flame-bar button").trigger("click");
    expect(names()[0]).toBe("s1");
    expect(wrapper.find(".davinci-flame-legend").exists()).toBe(true);
    wrapper.unmount();
  });

  it("says so when the run has no step timings", () => {
    const wrapper = mount(FlameView, { props: { profile: null, baseline: null } });
    expect(wrapper.find(".davinci-message").text()).toBe("This run has no step timings to draw.");
    wrapper.unmount();
  });
});
