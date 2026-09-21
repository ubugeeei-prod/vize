// The Davinci tab's data path against the real wasm compiler: the preset's
// Spolvero feed negotiates, climbs every rung, and every span the stage pages
// print lands on the authored source the editor highlights.
import { beforeAll, describe, expect, it } from "vite-plus/test";
import { loadWasm, type WasmModule } from "../src/wasm";
import { negotiateSpolveroFeed } from "../src/wasm/types/spolvero";
import { DAVINCI_PRESET } from "../src/shared/presets/davinci";
import { buildLadder, type StageLadder } from "../src/features/davinci/ladder";
import { folioLines } from "../src/features/davinci/folioLines";
import { templateBytesToSfcRange, templateStartInSfc } from "../src/features/davinci/offsets";

const FILENAME = "Component.vue";
let wasm: WasmModule;
let ladder: StageLadder;
let templateStart: number;

beforeAll(async () => {
  wasm = await loadWasm();
  const negotiated = negotiateSpolveroFeed(
    wasm.analyzeSfc(DAVINCI_PRESET, { filename: FILENAME }).spolvero,
  );
  if (!negotiated.ok) throw new Error(negotiated.error);
  ladder = buildLadder(negotiated.feed, FILENAME);
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
      ["s2", ["19 ops", "3 passes"], ["s2/lower", "s2/v-slot", "s2/v-model", "s2/hoist-static"]],
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
});
