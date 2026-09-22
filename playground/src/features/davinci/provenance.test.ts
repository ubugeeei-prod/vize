import { describe, expect, it } from "vite-plus/test";
import { isPassRecord, parseProvenance, recordsForNode } from "./provenance";
import { folioLines, folioTokens } from "./folioLines";

// Byte-for-byte what the compiler prints for `<div :class="cls">{{ msg }}</div>`
// with a trailing comment (see the Rust `spolvero_ladder` pins).
const PAGE = `[s2-provenance-folio]

[s2-provenance-folio.records]
rule=lower.element node=0 before="<div :class=\\"cls\\">" after="ui.element div" @3:36
rule=lower.bind node=1 before=":class=\\"cls\\"" after="ui.bind \\"class\\"" @8:20
rule=drop.comment node=- before="<!-- a\\nb -->" after="" @36:49
rule=pass.hoist-static.fact node=0 before="ui.element" after="level=not-static props=false" @3:36

`;

describe("parseProvenance", () => {
  it("reads every record, unescaping the page's strings", () => {
    expect(parseProvenance(PAGE)).toEqual([
      {
        rule: "lower.element",
        node: 0,
        before: '<div :class="cls">',
        after: "ui.element div",
        span: { start: 3, end: 36 },
      },
      {
        rule: "lower.bind",
        node: 1,
        before: ':class="cls"',
        after: 'ui.bind "class"',
        span: { start: 8, end: 20 },
      },
      {
        rule: "drop.comment",
        node: null,
        before: "<!-- a\nb -->",
        after: "",
        span: { start: 36, end: 49 },
      },
      {
        rule: "pass.hoist-static.fact",
        node: 0,
        before: "ui.element",
        after: "level=not-static props=false",
        span: { start: 3, end: 36 },
      },
    ]);
  });

  it("answers why an op exists: its lowering, then the facts passes attached", () => {
    const records = parseProvenance(PAGE);
    expect(recordsForNode(records, 0).map(({ rule }) => rule)).toEqual([
      "lower.element",
      "pass.hoist-static.fact",
    ]);
    expect(recordsForNode(records, 0).map(isPassRecord)).toEqual([false, true]);
    expect(recordsForNode(records, 9)).toEqual([]);
  });

  it("links provenance lines to their spans and colors the rule", () => {
    const lines = folioLines("provenance", PAGE);
    expect(lines.map((line) => line.span)).toEqual([
      null,
      null,
      null,
      { start: 3, end: 36 },
      { start: 8, end: 20 },
      { start: 36, end: 49 },
      { start: 3, end: 36 },
      null,
    ]);
    expect(folioTokens('rule=drop.comment node=- before="" after="" @0:1').slice(0, 4)).toEqual([
      { type: "key", text: "rule=" },
      { type: "mnemonic", text: "drop.comment" },
      { type: "text", text: " " },
      { type: "key", text: "node=" },
    ]);
  });

  it("numbers S2 op lines by page order, the ids records name", () => {
    const s2 = `[disegno]
ops=3

[disegno.ops]
ui.element div @3:36
  ui.bind name="class" value=js("cls" @16:19) @8:20
    attr element-kind="input" @8:20
  ui.interpolation js("msg" @24:27) @21:30

`;
    expect(folioLines("disegno", s2).map((line) => line.node)).toEqual([
      null,
      null,
      null,
      null,
      0,
      1,
      null,
      2,
      null,
    ]);
  });
});
