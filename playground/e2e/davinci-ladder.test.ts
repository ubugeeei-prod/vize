// The Davinci tab's data path against the real wasm compiler: the preset's
// Spolvero feed negotiates, climbs every rung, and every span the stage pages
// print lands on the authored source the editor highlights.
import { beforeAll, describe, expect, it } from "vite-plus/test";
import { loadWasm, type WasmModule } from "../src/wasm";
import { negotiateSpolveroFeed } from "../src/wasm/types/spolvero";
import { ladderStepTimings, negotiateProfileExport } from "../src/wasm/types/profile";
import { DAVINCI_PRESET } from "../src/shared/presets/davinci";
import { buildLadder, type StageLadder } from "../src/features/davinci/ladder";
import { folioLines } from "../src/features/davinci/folioLines";
import { parseProvenance, recordsForNode } from "../src/features/davinci/provenance";
import { remarksAt, summarizeRemarks } from "../src/features/davinci/remarks";
import { templateBytesToSfcRange, templateStartInSfc } from "../src/features/davinci/offsets";

const FILENAME = "Component.vue";
let wasm: WasmModule;
let ladder: StageLadder;
let templateStart: number;

beforeAll(async () => {
  wasm = await loadWasm();
  const analysis = wasm.analyzeSfc(DAVINCI_PRESET, { filename: FILENAME });
  const negotiated = negotiateSpolveroFeed(analysis.spolvero);
  if (!negotiated.ok) throw new Error(negotiated.error);
  const profile = negotiateProfileExport(analysis.spolveroProfile);
  if (!profile.ok) throw new Error(profile.error);
  ladder = buildLadder(negotiated.feed, FILENAME, ladderStepTimings(profile.profile));
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
  it("climbs S1, S2 with its artifact-selected passes, and S3", () => {
    expect(
      ladder.rungs.map((rung) => [rung.id, rung.facts, rung.pages.map((page) => page.key)]),
    ).toEqual([
      ["s1", ["12 lines"], ["s1/parse"]],
      [
        "s2",
        ["19 ops", "3 passes"],
        ["s2/lower", "s2/v-slot", "s2/v-model", "s2/hoist-static", "s2-provenance/transform"],
      ],
      ["s3", ["19 ops", "14 dynamic"], ["s3/lower", "s3-partition/lower", "s3-values/lower"]],
    ]);
    expect(ladder.unplaced).toEqual([]);
  });

  it("shows which passes changed the folio: none, on the Vue 3 path", () => {
    expect(ladder.timeline.map((step) => [step.key, step.producer, step.changed])).toEqual([
      ["s1/parse", true, true],
      ["s2/lower", true, true],
      ["s2/v-slot", false, false],
      ["s2/v-model", false, false],
      ["s2/hoist-static", false, false],
      ["s3/lower", true, true],
    ]);
  });

  it("maps stage lines back to the exact authored source", () => {
    expect(ladder.template).toBe(
      DAVINCI_PRESET.slice(templateStart, templateStart + ladder.template.length),
    );
    expect(authored("s2/lower", 'ui.on name="keyup"')).toBe('@keyup.enter="add"');
    expect(authored("s2/lower", 'js("todo.text"')).toBe("{{ todo.text }}");
    expect(authored("s3/lower", "kind=impeto.for")).toMatch(/^<TodoItem v-for="todo in todos"/);
    expect(authored("s3-partition/lower", "op=17 ")).toBe("<footer>{{ remaining }} left</footer>");
    expect(authored("s3-values/lower", '"model-read",3')).toBe("draft");
  });

  it("times every step through the browser clock, in the profile export", () => {
    // Values depend on the machine; which steps were measured does not.
    expect(ladder.timeline.map((step) => [step.key, typeof step.nanos])).toEqual([
      ["s1/parse", "number"],
      ["s2/lower", "number"],
      ["s2/v-slot", "number"],
      ["s2/v-model", "number"],
      ["s2/hoist-static", "number"],
      ["s3/lower", "number"],
    ]);
    expect(ladder.timeline.every((step) => step.nanos! >= 0)).toBe(true);
  });

  it("diffs consecutive S2 pages through the inspector's line diff", () => {
    const [lowered, vslot] = ladder.rungs[1].pages;
    const diff = wasm.buildInspectorDiff(lowered.text, vslot.text);
    const lineCount = lowered.text.split("\n").length;
    expect(diff.stats).toEqual({ additions: 0, removals: 0, unchanged: lineCount });
  });

  it("answers why an S2 op exists from the provenance page", () => {
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
    expect(ladder.timeline.find(({ key }) => key === "s2/hoist-static")!.remarks).toBe(11);
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
