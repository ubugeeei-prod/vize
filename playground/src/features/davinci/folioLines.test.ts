import { describe, expect, it } from "vite-plus/test";
import { folioLines, folioTokens, lineSpan, linesCovering, surfaceTokens } from "./folioLines";

// Page texts as the compiler prints them (pinned byte-for-byte by the Rust
// TS-52 `spolvero_ladder` suite for this template).
const TEMPLATE = '\n  <div :class="cls">{{ msg }}</div>\n';
const S2 = `[disegno]
ops=3

[disegno.ops]
ui.element div @3:36
  ui.bind name="class" value=js("cls" @16:19) @8:20
  ui.interpolation js("msg" @24:27) @21:30

`;

describe("folioTokens", () => {
  it("colors an S2 op line by role, keeping every byte", () => {
    const line = '  ui.bind name="class" value=js("cls" @16:19) @8:20';
    const tokens = folioTokens(line);
    expect(tokens.map((token) => token.text).join("")).toBe(line);
    expect(tokens).toEqual([
      { type: "text", text: "  " },
      { type: "mnemonic", text: "ui.bind" },
      { type: "text", text: " " },
      { type: "key", text: "name=" },
      { type: "string", text: '"class"' },
      { type: "text", text: " " },
      { type: "key", text: "value=" },
      { type: "call", text: "js(" },
      { type: "string", text: '"cls"' },
      { type: "text", text: " " },
      { type: "span", text: "@16:19" },
      { type: "punct", text: ")" },
      { type: "text", text: " " },
      { type: "span", text: "@8:20" },
    ]);
  });

  it("marks element names, sections, absent ids and partition kinds", () => {
    expect(folioTokens("ui.element div @3:36")).toEqual([
      { type: "mnemonic", text: "ui.element" },
      { type: "text", text: " " },
      { type: "tag", text: "div" },
      { type: "text", text: " " },
      { type: "span", text: "@3:36" },
    ]);
    expect(folioTokens("[s3-folio.ops]")).toEqual([{ type: "section", text: "[s3-folio.ops]" }]);
    expect(folioTokens("id=0 parent=- owner=- span=3:36").map((token) => token.type)).toEqual([
      "key",
      "number",
      "text",
      "key",
      "absent",
      "text",
      "key",
      "absent",
      "text",
      "key",
      "span",
    ]);
    expect(folioTokens("op=1 kind=dynamic span=8:20")[4]).toEqual({
      type: "dynamic",
      text: "dynamic",
    });
  });

  it("colors authored markup on the S1 page", () => {
    expect(surfaceTokens('<div :class="cls">{{ msg }}</div>')).toEqual([
      { type: "tag", text: "<div" },
      { type: "text", text: " " },
      { type: "directive", text: ":class" },
      { type: "text", text: "=" },
      { type: "string", text: '"cls"' },
      { type: "tag", text: ">" },
      { type: "mustache", text: "{{ msg }}" },
      { type: "tag", text: "</div" },
      { type: "tag", text: ">" },
    ]);
  });
});

describe("spans", () => {
  it("reads the op's own span, not an inner expression span", () => {
    expect(lineSpan("disegno", '  ui.bind name="class" value=js("cls" @16:19) @8:20')).toEqual({
      start: 8,
      end: 20,
    });
    expect(lineSpan("disegno", "[disegno.ops]")).toBeNull();
    expect(lineSpan("impeto", "id=1 parent=0 owner=0 span=21:30")).toEqual({ start: 21, end: 30 });
    expect(lineSpan("partition", "op=0 kind=static span=3:36")).toEqual({ start: 3, end: 36 });
    expect(lineSpan("values", 'operand=[1,"value",0,null,null,"js","cls","",16,19]')).toEqual({
      start: 16,
      end: 19,
    });
  });

  it("gives each S1 line its own byte range in the template", () => {
    const lines = folioLines("surface", TEMPLATE);
    expect(lines.map((line) => line.span)).toEqual([null, { start: 1, end: 36 }]);
    expect(TEMPLATE.slice(3, 36)).toBe('<div :class="cls">{{ msg }}</div>');
  });

  it("answers the reverse query narrowest-first", () => {
    const lines = folioLines("disegno", S2);
    expect(lines.map((line) => line.depth)).toEqual([0, 0, 0, 0, 0, 1, 1, 0]);
    // Byte 25 sits inside `msg`: the interpolation, then its element.
    expect(linesCovering(lines, 25)).toEqual([6, 4]);
    expect(linesCovering(lines, 0)).toEqual([]);
  });
});
