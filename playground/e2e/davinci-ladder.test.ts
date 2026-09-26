// The Davinci tab's data path against the real wasm compiler: the preset's
// Spolvero feed negotiates, climbs every rung, and every span the stage pages
// print lands on the authored source the editor highlights.
import { beforeAll, describe, expect, it } from "vite-plus/test";
import { loadWasm, type WasmModule } from "../src/wasm";
import { negotiateSpolveroFeed } from "../src/wasm/types/spolvero";
import {
  LADDER_STEP_KEY,
  ladderStepTimings,
  ladderWalkTimings,
  negotiateProfileExport,
  type ProfileExport,
} from "../src/wasm/types/profile";
import { DAVINCI_PRESET } from "../src/shared/presets/davinci";
import { buildLadder, type StageLadder } from "../src/features/davinci/ladder";
import { folioLines } from "../src/features/davinci/folioLines";
import { parseProvenance, recordsForNode } from "../src/features/davinci/provenance";
import { remarksAt, summarizeRemarks } from "../src/features/davinci/remarks";
import { templateBytesToSfcRange, templateStartInSfc } from "../src/features/davinci/offsets";
import { flameGraph } from "../src/features/davinci/flame";

const FILENAME = "Component.vue";
let wasm: WasmModule;
let ladder: StageLadder;
let templateStart: number;
let exported: ProfileExport;

beforeAll(async () => {
  wasm = await loadWasm();
  const analysis = wasm.analyzeSfc(DAVINCI_PRESET, { filename: FILENAME });
  const negotiated = negotiateSpolveroFeed(analysis.spolvero);
  if (!negotiated.ok) throw new Error(negotiated.error);
  const profile = negotiateProfileExport(analysis.spolveroProfile);
  if (!profile.ok) throw new Error(profile.error);
  exported = profile.profile;
  ladder = buildLadder(
    negotiated.feed,
    FILENAME,
    ladderStepTimings(profile.profile),
    ladderWalkTimings(profile.profile),
  );
  const sfc = wasm.compileSfc(DAVINCI_PRESET, { filename: FILENAME });
  templateStart = templateStartInSfc(DAVINCI_PRESET, sfc.descriptor.template!.loc.start);
});

function authored(pageKey: string, needle: string): string {
  const page = ladder.rungs.flatMap((rung) => rung.pages).find((p) => p.key === pageKey)!;
  const line = folioLines(page.kind, page.text).find((l) => l.text.includes(needle))!;
  const range = templateBytesToSfcRange(ladder.template, templateStart, line.span!);
  return DAVINCI_PRESET.slice(range.start, range.end);
}

describe("Davinci stage ladder from the real compiler", () => {
  it("climbs L1, L2 with its artifact-selected passes, and L3", () => {
    expect(
      ladder.rungs.map((rung) => [rung.id, rung.facts, rung.pages.map((page) => page.key)]),
    ).toEqual([
      ["l1", ["12 lines"], ["l1/parse"]],
      [
        "l2",
        ["19 ops", "4 passes"],
        [
          "l2/lower",
          "l2-plan/transform",
          "l2/v-slot",
          "l2/v-model",
          "l2/hoist-static",
          "l2/template-complexity",
          "l2-provenance/transform",
        ],
      ],
      ["l3", ["19 ops", "14 dynamic"], ["l3/lower", "l3-partition/lower", "l3-values/lower"]],
    ]);
    expect(ladder.unplaced).toEqual([]);
  });

  it("shows which passes changed the folio: none, on the Vue 3 path", () => {
    expect(ladder.timeline.map((step) => [step.key, step.producer, step.changed])).toEqual([
      ["l1/parse", true, true],
      ["l2/lower", true, true],
      ["l2/v-slot", false, false],
      ["l2/v-model", false, false],
      ["l2/hoist-static", false, false],
      ["l2/template-complexity", false, false],
      ["l3/lower", true, true],
    ]);
  });

  it("maps stage lines back to the exact authored source", () => {
    expect(ladder.template).toBe(
      DAVINCI_PRESET.slice(templateStart, templateStart + ladder.template.length),
    );
    expect(authored("l2/lower", 'ui.on name="keyup"')).toBe('@keyup.enter="add"');
    expect(authored("l2/lower", 'js("todo.text"')).toBe("{{ todo.text }}");
    expect(authored("l3/lower", "kind=impeto.for")).toMatch(/^<TodoItem v-for="todo in todos"/);
    expect(authored("l3-partition/lower", "op=17 ")).toBe("<footer>{{ remaining }} left</footer>");
    expect(authored("l3-values/lower", '"model-read",3')).toBe("draft");
  });

  it("times every step through the browser clock, in the profile export", () => {
    // Values depend on the machine; which steps were measured does not.
    expect(ladder.timeline.map((step) => [step.key, typeof step.nanos])).toEqual([
      ["l1/parse", "number"],
      ["l2/lower", "number"],
      ["l2/v-slot", "number"],
      ["l2/v-model", "number"],
      ["l2/hoist-static", "number"],
      ["l2/template-complexity", "number"],
      ["l3/lower", "number"],
    ]);
    expect(ladder.timeline.every((step) => step.nanos! >= 0)).toBe(true);
  });

  it("runs each pass in its own walk, per the compiler's plan page", () => {
    // Two mandatory barriers, then the two optional analyses fused into
    // the third walk, timed once and attributed to its lead pass.
    expect(ladder.walks.map(({ index, passes, fusable }) => [index, passes, fusable])).toEqual([
      [0, ["v-slot"], false],
      [1, ["v-model"], false],
      [2, ["hoist-static", "template-complexity"], true],
    ]);
    expect(ladder.walks.every(({ nanos }) => typeof nanos === "number" && nanos >= 0)).toBe(true);
    expect(ladder.timeline.map(({ key, walk }) => [key, walk])).toEqual([
      ["l1/parse", null],
      ["l2/lower", null],
      ["l2/v-slot", 0],
      ["l2/v-model", 1],
      ["l2/hoist-static", 2],
      ["l2/template-complexity", 2],
      ["l3/lower", null],
    ]);
  });

  it("draws the flame view from the export's attributed step spans", () => {
    const flame = flameGraph(exported, LADDER_STEP_KEY);
    const at = (depth: number) =>
      flame.frames.filter((frame) => frame.depth === depth).map((f) => f.path.join("/"));
    expect(at(0)).toEqual(["l1", "l2", "l3"]);
    expect(at(1)).toEqual([
      "l1/parse",
      "l2/hoist-static",
      "l2/lower",
      "l2/template-complexity",
      "l2/v-model",
      "l2/v-slot",
      "l3/lower",
    ]);
    expect(at(2)).toEqual(at(1).map((path) => `${path}/template`));
    // Every step is one frame, so the graph's width is the steps' sum.
    const steps = ladder.timeline.reduce((sum, step) => sum + step.nanos!, 0);
    expect(flame.total).toBe(steps);
  });

  it("diffs consecutive L2 pages through the inspector's line diff", () => {
    const page = (key: string) => ladder.rungs[1].pages.find((p) => p.key === key)!;
    const [lowered, vslot] = [page("l2/lower"), page("l2/v-slot")];
    const diff = wasm.buildInspectorDiff(lowered.text, vslot.text);
    const lineCount = lowered.text.split("\n").length;
    expect(diff.stats).toEqual({ additions: 0, removals: 0, unchanged: lineCount });
  });

  it("answers why an L2 op exists from the provenance page", () => {
    const [lowered] = ladder.rungs[1].pages;
    const provenance = ladder.rungs[1].pages.find((page) => page.kind === "provenance")!;
    const records = parseProvenance(provenance.text);
    const lines = folioLines("disegno", lowered.text);
    const why = (needle: string) => {
      const line = lines.find((l) => l.text.includes(needle))!;
      return recordsForNode(records, line.node!).map(({ rule, after }) => [rule, after]);
    };
    expect(why('ui.on name="keyup"')).toEqual([["lower.on", 'ui.on "keyup"']]);
    expect(why("ui.element h1")).toEqual([
      ["lower.element", "ui.element h1"],
      ["pass.hoist-static.fact", "level=fully-static props=false nested=true native=true"],
    ]);
    expect(records.filter(({ node }) => node === null).map(({ rule }) => rule)).toContain(
      "condense.drop-whitespace",
    );
  });

  it("carries hoist-static's optimization remarks and ties them to ops", () => {
    expect(summarizeRemarks(ladder.remarks)).toEqual({ applied: 3, missed: 8, analysis: 0 });
    expect(ladder.timeline.find(({ key }) => key === "l2/hoist-static")!.remarks).toBe(11);
    const lowered = ladder.rungs[1].pages[0];
    const lines = folioLines("disegno", lowered.text);
    const about = (needle: string) => {
      const line = lines.find((l) => l.text.includes(needle))!;
      return remarksAt(ladder.remarks, line.span!).map(({ kind, name, args }) => [
        kind,
        name,
        args.map(({ key, value }) => `${key}=${String(value)}`).join(" "),
      ]);
    };
    expect(about("ui.element h1")).toEqual([["applied", "static-subtree", "tag=h1"]]);
    expect(about("ui.element input")).toEqual([
      ["missed", "static-props", "tag=input blocker=binding op=ui.model"],
      ["missed", "static-subtree", "tag=input blocker=binding op=ui.model"],
    ]);
  });
});
