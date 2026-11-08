/** Optional, provider-driven ASCII font entry point. */

import { computed, defineComponent, h, type PropType } from "vue";

export interface AsciiFontProvider {
  /** Stable provider name used by diagnostics and devtools. */
  readonly name: string;
  /** Number of terminal rows produced for every input. */
  readonly height: number;
  /** Render text into equal-height terminal rows. */
  render(text: string): readonly string[];
}

export interface GlyphAsciiFontOptions {
  name: string;
  glyphs: Readonly<Record<string, readonly string[]>>;
  fallback?: readonly string[];
  letterSpacing?: number;
  normalize?: (character: string) => string;
}

export interface AsciiFontProps {
  text: string;
  provider: AsciiFontProvider;
  fg?: string;
  bg?: string;
  bold?: boolean;
}

function normalizedSpacing(value: number | undefined): number {
  if (value === undefined || !Number.isFinite(value)) return 1;
  return Math.max(0, Math.trunc(value));
}

function validateRows(label: string, rows: readonly string[], height: number): string[] {
  if (rows.length !== height) {
    throw new Error(
      `FRESCO_ASCII_FONT_HEIGHT: ${label} has ${rows.length} rows; expected ${height}`,
    );
  }
  return [...rows];
}

/** Create an immutable provider from a consumer-owned glyph table. */
export function createGlyphAsciiFont(options: GlyphAsciiFontOptions): AsciiFontProvider {
  const glyphEntries = Object.entries(options.glyphs);
  const firstGlyph = glyphEntries[0];
  if (!firstGlyph || firstGlyph[1].length === 0) {
    throw new Error("FRESCO_ASCII_FONT_EMPTY: glyphs must contain at least one non-empty glyph");
  }

  const height = firstGlyph[1].length;
  const glyphs = new Map(
    glyphEntries.map(([character, rows]) => [character, validateRows(character, rows, height)]),
  );
  const fallback = validateRows("fallback", options.fallback ?? Array(height).fill("?"), height);
  const separator = " ".repeat(normalizedSpacing(options.letterSpacing));
  const normalize = options.normalize ?? ((character: string) => character.toUpperCase());

  return Object.freeze({
    name: options.name,
    height,
    render(text: string): readonly string[] {
      const characters = Array.from(
        text,
        (character) => glyphs.get(normalize(character)) ?? fallback,
      );
      return Array.from({ length: height }, (_, row) =>
        characters.map((glyph) => glyph[row] ?? "").join(separator),
      );
    },
  });
}

/** Render text with a provider kept outside Fresco's core entry point. */
export const AsciiFont = defineComponent({
  name: "AsciiFont",
  props: {
    text: { type: String, required: true },
    provider: { type: Object as PropType<AsciiFontProvider>, required: true },
    fg: String,
    bg: String,
    bold: Boolean,
  },
  setup(props) {
    const rows = computed(() => {
      const rendered = [...props.provider.render(props.text)];
      return validateRows(props.provider.name, rendered, props.provider.height);
    });

    return () =>
      h(
        "box",
        {
          style: { flexDirection: "column" },
          "aria-label": props.text,
          "data-ascii-font-provider": props.provider.name,
        },
        rows.value.map((row, index) =>
          h("text", { key: index, text: row || " ", fg: props.fg, bg: props.bg, bold: props.bold }),
        ),
      );
  },
});
