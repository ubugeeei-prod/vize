import { describe, expect, it } from "vite-plus/test";
import { dumpLines } from "./dumpLines";
import { cursorSpan, traceMatches, tracePages } from "./sourceTrace";
import { irNotation } from "./irNotation";
import type { Rung } from "./ladder";

const tree =
  '[l2-dump-v2]\nops=3\n\n[l2-dump-v2.ops]\nui.element div @0:40\n  ui.interpolation js("x=y" @8:11) @5:14\n  ui.text "other" @20:25\n';
const graph =
  "[l3-dump-v2]\nphase=built\n\n[l3-dump-v2.regions]\nid=0 parent=- owner=- span=0:40\n\n[l3-dump-v2.ops]\nid=0 kind=l3.insert-node region=0 effect=- span=0:40\nid=1 kind=l3.set-text region=0 effect=0 span=5:14\nid=2 kind=l3.set-text region=0 effect=0 span=5:14\n\n[l3-dump-v2.effects]\nid=0 owner=1 region=0 span=5:14\n";

describe("cross-layer source correspondence", () => {
  it("links all exact operations and effects without highlighting their parent or siblings", () => {
    const lines = dumpLines("impeto", graph);
    const matches = traceMatches(lines, { start: 5, end: 14 });
    expect(matches.map((index) => lines[index].text)).toEqual([
      "id=1 kind=l3.set-text region=0 effect=0 span=5:14",
      "id=2 kind=l3.set-text region=0 effect=0 span=5:14",
      "id=0 owner=1 region=0 span=5:14",
    ]);
    expect(traceMatches(lines, { start: 40, end: 41 })).toEqual([]);
  });
  it("uses the narrowest enclosing operation for an expression and contained ops for a range", () => {
    const lines = dumpLines("disegno", tree);
    expect(traceMatches(lines, { start: 8, end: 11 })).toEqual([5]);
    expect(traceMatches(lines, { start: 4, end: 26 })).toEqual([5, 6]);
  });
  it("includes partially covered L1 lines and handles Unicode byte coordinates", () => {
    const lines = dumpLines("surface", "<p>日本語\n{{ msg }}</p>");
    expect(traceMatches(lines, { start: 3, end: 22 }, "surface")).toEqual([0, 1]);
    expect(traceMatches(lines, { start: 13, end: 22 }, "surface")).toEqual([1]);
  });
  it("chooses semantic spans rather than a whole source line at the cursor", () => {
    const rungs: Rung[] = [
      {
        id: "l1",
        name: "Surface",
        ordinal: "L1",
        facts: [],
        pages: [
          {
            key: "l1/parse",
            kind: "surface",
            stage: "l1",
            pass: "parse",
            label: "Surface",
            text: " ".repeat(40),
          },
        ],
      },
      {
        id: "l2",
        name: "Disegno",
        ordinal: "L2",
        facts: [],
        pages: [
          {
            key: "l2/lower",
            kind: "disegno",
            stage: "l2",
            pass: "lower",
            label: "Lowered",
            text: tree,
          },
        ],
      },
    ];
    expect(cursorSpan(tracePages(rungs, null), 9)).toEqual({ start: 5, end: 14 });
    expect(cursorSpan(tracePages(rungs, null), null)).toBeNull();
  });
});

describe("IR DSL presentation", () => {
  it("preserves every mapped row's raw identity and quoted expression contents", () => {
    const original = dumpLines("disegno", tree);
    const shown = irNotation("disegno", original);
    expect(shown.map((line) => line.text).join("\n")).toContain(
      '%1 = ui.interpolation js("x=y" @8:11) loc(5..14)',
    );
    expect(
      shown.filter((line) => line.span).map(({ index, span, node }) => ({ index, span, node })),
    ).toEqual(
      original.filter((line) => line.span).map(({ index, span, node }) => ({ index, span, node })),
    );
    expect(
      shown.every((line) => line.tokens.map((token) => token.text).join("") === line.text),
    ).toBe(true);
    expect(shown.map((line) => line.text).join("\n")).not.toContain("[l2-dump-v2]");
  });
  it("prints real region, operation and effect references without inventing absent effects", () => {
    const shown = irNotation("impeto", dumpLines("impeto", graph));
    const text = shown.map((line) => line.text).join("\n");
    expect(text).toContain("^r0 root loc(0..40)");
    expect(text).toContain("%0 = l3.insert-node in ^r0 loc(0..40)");
    expect(text).toContain("%1 = l3.set-text in ^r0 !e0 loc(5..14)");
    expect(text).toContain("!e0 tracks %1 in ^r0 loc(5..14)");
    expect((text.match(/\{/g) ?? []).length).toBe((text.match(/\}/g) ?? []).length);
    expect(new Set(shown.map((line) => line.index)).size).toBe(shown.length);
  });
  it("keeps unknown fields visible and never rewrites quoted key=value text", () => {
    const shown = irNotation(
      "facts",
      dumpLines("facts", '[future-folio]\nname="key=value" unknown=42\n'),
    );
    expect(shown.map((line) => line.text).join("\n")).toContain('name: "key=value" unknown: 42');
  });
});
