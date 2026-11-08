import assert from "node:assert/strict";
import { test } from "vitest";

import { firstChild, mountComponent } from "../testing/index.js";
import { AsciiFont, createGlyphAsciiFont } from "./index.js";

const provider = createGlyphAsciiFont({
  name: "test-font",
  glyphs: {
    A: [" A ", "A A", "AAA"],
    B: ["BB ", "BBB", "BB "],
  },
  fallback: [" ? ", " ? ", " ? "],
});

test("creates a normalized glyph provider with Unicode-safe fallback", () => {
  assert.equal(provider.height, 3);
  assert.deepEqual(provider.render("a🦀b"), [" A   ?  BB ", "A A  ?  BBB", "AAA  ?  BB "]);
});

test("renders provider rows without entering the core component entry", () => {
  const mounted = mountComponent(AsciiFont, {
    text: "AB",
    provider,
    fg: "cyan",
    bold: true,
  });
  const root = firstChild(mounted);

  assert.equal(root.props["data-ascii-font-provider"], "test-font");
  assert.equal(root.props["aria-label"], "AB");
  assert.deepEqual(
    root.children.map((row) => ({ text: row.props.text, fg: row.props.fg, bold: row.props.bold })),
    [
      { text: " A  BB ", fg: "cyan", bold: true },
      { text: "A A BBB", fg: "cyan", bold: true },
      { text: "AAA BB ", fg: "cyan", bold: true },
    ],
  );
  mounted.unmount();
});

test("rejects inconsistent provider glyph heights", () => {
  assert.throws(
    () =>
      createGlyphAsciiFont({
        name: "broken",
        glyphs: { A: ["A", "A"], B: ["B"] },
      }),
    /FRESCO_ASCII_FONT_HEIGHT/u,
  );
});
