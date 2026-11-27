import { describe, expect, it } from "vite-plus/test";
import { dumpLines } from "./dumpLines";
import { surfaceLines, linkedSurfaceUnits } from "./surfaceUnits";
import { utf16ToUtf8Offset } from "./offsets";

describe("L1 analysis units", () => {
  it("separates nested attributes and expressions on the same line with Unicode byte spans", () => {
    const text = '<p title="日本語🌸" :class="cls">{{ first }}<b>{{ last }}</b></p>';
    const span = (part: string) => {
      const start = text.indexOf(part);
      return `${utf16ToUtf8Offset(text, start)}:${utf16ToUtf8Offset(text, start + part.length)}`;
    };
    const units = dumpLines(
      "disegno",
      [
        `ui.element p @${span(text)}`,
        `  attr title="日本語🌸" @${span('title="日本語🌸"')}`,
        `  ui.bind name="class" @${span(':class="cls"')}`,
        `  ui.interpolation @${span("{{ first }}")}`,
        `  ui.element b @${span("<b>{{ last }}</b>")}`,
        `    ui.interpolation @${span("{{ last }}")}`,
      ].join("\n"),
    );
    const fragments = surfaceLines(text, units)[0].fragments;
    expect(fragments.map((fragment) => fragment.text).join("")).toBe(text);
    for (const [part, index] of [
      ["title=", 1],
      [":class=", 2],
      ["{{ first }}", 3],
      ["{{ last }}", 5],
    ] as const) {
      expect(fragments.find((fragment) => fragment.text.startsWith(part))?.unit).toBe(units[index]);
    }
    expect(fragments.find((fragment) => fragment.text === " ")?.unit).toBeNull();
    expect([...linkedSurfaceUnits(units, units[2].span)]).toEqual([2]);
  });

  it("keeps a multiline directive one analysis unit and leaves unobserved text inert", () => {
    const text = '<p\n  :title="\n    name\n  "\n>text</p>';
    const start = text.indexOf(":title");
    const end = text.indexOf('"', text.indexOf('"') + 1) + 1;
    const units = dumpLines("disegno", `ui.bind @${start}:${end}`);
    const lines = surfaceLines(text, units);
    expect(
      lines.map((line) => line.fragments.map((fragment) => fragment.text).join("")).join("\n"),
    ).toBe(text);
    expect(
      lines
        .flatMap((line) => line.fragments)
        .filter((fragment) => fragment.unit)
        .map((fragment) => fragment.unit!.span),
    ).toEqual([units[0].span, units[0].span, units[0].span]);
    expect(
      surfaceLines(text, []).every((line) =>
        line.fragments.every((fragment) => fragment.unit === null),
      ),
    ).toBe(true);
  });
});
