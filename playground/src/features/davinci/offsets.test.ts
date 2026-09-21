import { describe, expect, it } from "vite-plus/test";
import {
  sfcOffsetToTemplateBytes,
  templateBytesToSfcRange,
  templateStartInSfc,
  utf16ToUtf8Offset,
  utf8ToUtf16Offset,
} from "./offsets";

// "é" is 2 UTF-8 bytes / 1 UTF-16 unit; "🎨" is 4 bytes / 2 units.
const TEXT = "aé🎨b";

describe("utf8/utf16 offsets", () => {
  it("maps every character boundary exactly in both directions", () => {
    const boundaries = [
      [0, 0],
      [1, 1],
      [3, 2],
      [7, 4],
      [8, 5],
    ];
    for (const [bytes, units] of boundaries) {
      expect(utf8ToUtf16Offset(TEXT, bytes)).toBe(units);
      expect(utf16ToUtf8Offset(TEXT, units)).toBe(bytes);
    }
  });

  it("resolves a byte inside a character to its start and clamps past the end", () => {
    expect(utf8ToUtf16Offset(TEXT, 2)).toBe(1);
    expect(utf8ToUtf16Offset(TEXT, 5)).toBe(2);
    expect(utf8ToUtf16Offset(TEXT, 99)).toBe(5);
  });
});

describe("template <-> SFC ranges", () => {
  const sfc = "<!-- é -->\n<template>\n  <p>🎨{{ x }}</p>\n</template>\n";
  const template = "\n  <p>🎨{{ x }}</p>\n";
  // `<template>` content starts right after the tag, at byte 22.
  const templateByteStart = new TextEncoder().encode("<!-- é -->\n<template>").length;

  it("places the template start in UTF-16 units", () => {
    expect(templateByteStart).toBe(22);
    expect(templateStartInSfc(sfc, templateByteStart)).toBe(21);
  });

  it("maps a template byte span onto the same SFC text", () => {
    const start = templateStartInSfc(sfc, templateByteStart);
    // `{{ x }}` sits after the 4-byte emoji: template bytes 10..17.
    const range = templateBytesToSfcRange(template, start, { start: 10, end: 17 });
    expect(sfc.slice(range.start, range.end)).toBe("{{ x }}");
  });

  it("maps an SFC offset back into template bytes, or null outside it", () => {
    const start = templateStartInSfc(sfc, templateByteStart);
    const mustache = sfc.indexOf("{{");
    expect(sfcOffsetToTemplateBytes(template, start, mustache)).toBe(10);
    expect(sfcOffsetToTemplateBytes(template, start, 3)).toBeNull();
    expect(sfcOffsetToTemplateBytes(template, start, start + template.length + 1)).toBeNull();
  });
});
