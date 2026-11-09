// The Levels tab's data path against the real wasm compiler: each target's
// `compileSfc` `captureStages` feed negotiates, climbs its rungs, and every
// span the stage pages print lands on the authored source the editor highlights.
import { beforeAll, describe, expect, it } from "vite-plus/test";
import { ref, shallowRef } from "vue";
import { loadWasm, type CompilerOptions, type WasmModule } from "../src/wasm";
import { negotiateProductCapture } from "../src/wasm/types/productCapture";
import { LEVEL_PRESET } from "../src/shared/presets/levels";
import type { CodeOutputs } from "../src/features/atelier/codeOutputs";
import type { StageLadder } from "../src/features/stages/ladder";
import {
  useProductCaptureState,
  type ProductCapturedResult,
} from "../src/features/stages/productCaptureState";
import { dumpLines } from "../src/features/stages/dumpLines";
import { parseProvenance, recordsForNode } from "../src/features/stages/provenance";
import { remarksAt, summarizeRemarks } from "../src/features/stages/remarks";
import { templateBytesToSfcRange } from "../src/features/stages/offsets";

const FILENAME = "Component.vue";
// The stage tab's own `compileSfc` options per target.
const BASE: CompilerOptions = {
  mode: "module",
  scriptExt: "preserve",
  filename: FILENAME,
  captureStages: true,
};
const TARGETS = {
  dom: BASE,
  ssr: { ...BASE, ssr: true, outputMode: "vdom" },
} as const;

interface Run {
  ladder: StageLadder;
  templateStart: number;
  observed: { timings: boolean; remarks: boolean };
}

let wasm: WasmModule;
let dom: Run;
let ssr: Run;

function run(target: keyof typeof TARGETS): Run {
  const result = wasm.compileSfc(LEVEL_PRESET, TARGETS[target]);
  const negotiation = negotiateProductCapture(result.stageCapture, target);
  if (!negotiation.ok) throw new Error(negotiation.error);
  const captured: ProductCapturedResult = { result, negotiation };
  const state = useProductCaptureState(
    shallowRef({ [target]: captured }),
    ref(LEVEL_PRESET),
    ref(LEVEL_PRESET),
    shallowRef({} as CodeOutputs),
    ref(target),
  );
  const ladder = state.ladder.value;
  const templateStart = state.templateStart.value;
  if (!ladder || templateStart === null) throw new Error(`${target}: no verified ladder`);
  return { ladder, templateStart, observed: negotiation.feed.observed };
}

beforeAll(async () => {
  wasm = await loadWasm();
  dom = run("dom");
  ssr = run("ssr");
});

function page({ ladder }: Run, key: string) {
  return ladder.rungs.flatMap((rung) => rung.pages).find((p) => p.key === key);
}

function authored(at: Run, pageKey: string, needle: string): string {
  const shown = page(at, pageKey)!;
  const line = dumpLines(shown.kind, shown.text).find((l) => l.text.includes(needle))!;
  const range = templateBytesToSfcRange(at.ladder.template, at.templateStart, line.span!);
  return LEVEL_PRESET.slice(range.start, range.end);
}

function why(at: Run, needle: string) {
  const provenance = at.ladder.rungs[1].pages.find((p) => p.kind === "provenance");
  expect(provenance).toBeDefined();
  const records = parseProvenance(provenance!.text);
  const line = dumpLines("l2", page(at, "l2/lower")!.text).find((l) => l.text.includes(needle))!;
  return recordsForNode(records, line.node!).map(({ rule, after }) => [rule, after]);
}

describe("Stage ladder from the real compiler", () => {
  it("climbs L1 and L2 from the DOM product run, keeping L4 apart", () => {
    expect(
      dom.ladder.rungs.map((rung) => [rung.id, rung.facts, rung.pages.map((p) => p.key)]),
    ).toEqual([
      ["l1", ["12 lines"], ["l1/parse"]],
      [
        "l2",
        ["19 ops", "4 passes"],
        [
          "l2/lower",
          "l2-plan/plan",
          "l2/v-slot",
          "l2/v-model",
          "l2/hoist-static",
          "l2/template-complexity",
          "l2-provenance/provenance",
          "l2-facts/facts",
        ],
      ],
    ]);
    expect(dom.ladder.l4Pages.map(({ step }) => step)).toEqual(["emit"]);
    expect(dom.ladder.unplaced).toEqual([]);
  });

  it("climbs L1, L2 and L3 from the SSR product run", () => {
    expect(
      ssr.ladder.rungs.map((rung) => [rung.id, rung.facts, rung.pages.map((p) => p.key)]),
    ).toEqual([
      ["l1", ["12 lines"], ["l1/parse"]],
      ["l2", ["19 ops", "0 passes"], ["l2/lower", "l2-provenance/provenance"]],
      ["l3", ["19 ops", "14 dynamic"], ["l3/lower", "l3-partition/partition", "l3-values/values"]],
    ]);
    expect(ssr.ladder.unplaced).toEqual([]);
  });

  it("shows each product step as a new artifact", () => {
    expect(ssr.ladder.timeline.map((step) => [step.key, step.producer, step.changed])).toEqual([
      ["l1/parse", true, true],
      ["l2/lower", true, true],
      ["l3/lower", true, true],
    ]);
  });

  it("maps stage lines back to the exact authored source", () => {
    expect(ssr.ladder.template).toBe(
      LEVEL_PRESET.slice(ssr.templateStart, ssr.templateStart + ssr.ladder.template.length),
    );
    expect(authored(ssr, "l2/lower", 'ui.on name="keyup"')).toBe('@keyup.enter="add"');
    expect(authored(ssr, "l2/lower", 'js("todo.text"')).toBe("{{ todo.text }}");
    expect(authored(ssr, "l3/lower", "kind=l3.for")).toMatch(/^<TodoItem v-for="todo in todos"/);
    expect(authored(ssr, "l3-partition/partition", "op=17 ")).toBe(
      "<footer>{{ remaining }} left</footer>",
    );
    expect(authored(ssr, "l3-values/values", '"model-read",3')).toBe("draft");
  });

  it("answers why an L2 op exists from the provenance page", () => {
    expect(why(ssr, 'ui.on name="keyup"')).toEqual([["lower.on", 'ui.on "keyup"']]);
    expect(why(ssr, "ui.element h1")).toEqual([["lower.element", "ui.element h1"]]);
    const provenance = ssr.ladder.rungs[1].pages.find((p) => p.kind === "provenance")!;
    expect(
      parseProvenance(provenance.text)
        .filter(({ node }) => node === null)
        .map(({ rule }) => rule),
    ).toContain("condense.drop-whitespace");
  });

  it("runs the artifact-selected L2 passes, one page each", () => {
    expect(dom.ladder.timeline.map((step) => [step.key, step.producer, step.changed])).toEqual([
      ["l1/parse", true, true],
      ["l2/lower", true, true],
      ["l2/v-slot", false, false],
      ["l2/v-model", false, false],
      ["l2/hoist-static", false, false],
      ["l2/template-complexity", false, false],
    ]);
  });

  it("diffs consecutive L2 pages through the inspector's line diff", () => {
    const [lowered, vslot] = [page(dom, "l2/lower")!, page(dom, "l2/v-slot")];
    expect(vslot).toBeDefined();
    const diff = wasm.buildInspectorDiff(lowered.text, vslot!.text);
    const lineCount = lowered.text.split("\n").length;
    expect(diff.stats).toEqual({ additions: 0, removals: 0, unchanged: lineCount });
  });

  it("runs each pass in its own walk, per the compiler's plan page", () => {
    // Two mandatory barriers, then the two optional analyses fused into the
    // third walk.
    expect(dom.ladder.walks.map(({ index, passes, fusable }) => [index, passes, fusable])).toEqual([
      [0, ["v-slot"], false],
      [1, ["v-model"], false],
      [2, ["hoist-static", "template-complexity"], true],
    ]);
    expect(dom.ladder.timeline.map(({ key, walk }) => [key, walk])).toEqual([
      ["l1/parse", null],
      ["l2/lower", null],
      ["l2/v-slot", 0],
      ["l2/v-model", 1],
      ["l2/hoist-static", 2],
      ["l2/template-complexity", 2],
    ]);
  });

  it("times every step and walk through the browser clock", () => {
    // Values depend on the machine; which steps were measured does not.
    expect(dom.observed.timings).toBe(true);
    expect(dom.ladder.timeline.every(({ nanos }) => typeof nanos === "number" && nanos >= 0)).toBe(
      true,
    );
    expect(dom.ladder.walks.length).toBeGreaterThan(0);
    expect(dom.ladder.walks.every(({ nanos }) => typeof nanos === "number" && nanos >= 0)).toBe(
      true,
    );
  });

  it("answers which pass facts the DOM run recorded for an L2 op", () => {
    expect(why(dom, "ui.element h1")).toEqual([
      ["lower.element", "ui.element h1"],
      ["pass.hoist-static.fact", "level=fully-static props=false nested=true native=true"],
    ]);
  });

  it("carries hoist-static's optimization remarks and ties them to ops", () => {
    expect(dom.observed.remarks).toBe(true);
    expect(summarizeRemarks(dom.ladder.remarks)).toEqual({ applied: 3, missed: 8, analysis: 0 });
    expect(dom.ladder.timeline.find(({ key }) => key === "l2/hoist-static")!.remarks).toBe(11);
    const lines = dumpLines("l2", page(dom, "l2/lower")!.text);
    const about = (needle: string) => {
      const line = lines.find((l) => l.text.includes(needle))!;
      return remarksAt(dom.ladder.remarks, line.span!).map(({ kind, name, args }) => [
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
