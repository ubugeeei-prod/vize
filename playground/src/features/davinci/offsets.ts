// Offset arithmetic between the compiler's coordinates and the editor's.
//
// Every Davinci stage span is a UTF-8 byte offset into the template content
// (the S1 page text, by the TS-19 fidelity law). Monaco and JavaScript strings
// count UTF-16 code units. These helpers convert exactly in both directions
// without re-deriving anything from the compiler.

/** Half-open range. */
export interface Range {
  start: number;
  end: number;
}

function utf8Length(codePoint: number): number {
  if (codePoint < 0x80) return 1;
  if (codePoint < 0x800) return 2;
  if (codePoint < 0x10000) return 3;
  return 4;
}

/**
 * The UTF-16 offset in `text` of UTF-8 byte offset `byteOffset`. Offsets past
 * the end clamp to `text.length`; an offset inside a multi-byte character
 * resolves to that character's start.
 */
export function utf8ToUtf16Offset(text: string, byteOffset: number): number {
  let bytes = 0;
  let index = 0;
  while (index < text.length) {
    const codePoint = text.codePointAt(index)!;
    const width = utf8Length(codePoint);
    if (bytes + width > byteOffset) return index;
    bytes += width;
    index += codePoint > 0xffff ? 2 : 1;
  }
  return text.length;
}

/** The UTF-8 byte offset of UTF-16 offset `utf16Offset` in `text`. */
export function utf16ToUtf8Offset(text: string, utf16Offset: number): number {
  let bytes = 0;
  let index = 0;
  const limit = Math.min(utf16Offset, text.length);
  while (index < limit) {
    const codePoint = text.codePointAt(index)!;
    bytes += utf8Length(codePoint);
    index += codePoint > 0xffff ? 2 : 1;
  }
  return bytes;
}

/**
 * Where the template content starts in the SFC, in UTF-16 units, given the
 * descriptor's byte offset (`template.loc.start`).
 */
export function templateStartInSfc(sfc: string, templateByteStart: number): number {
  return utf8ToUtf16Offset(sfc, templateByteStart);
}

/** A template-relative byte range as an SFC UTF-16 range. */
export function templateBytesToSfcRange(
  template: string,
  templateStart: number,
  range: Range,
): Range {
  return {
    start: templateStart + utf8ToUtf16Offset(template, range.start),
    end: templateStart + utf8ToUtf16Offset(template, range.end),
  };
}

/**
 * An SFC UTF-16 offset as a template-relative byte offset, or null when the
 * offset lies outside the template content.
 */
export function sfcOffsetToTemplateBytes(
  template: string,
  templateStart: number,
  sfcOffset: number,
): number | null {
  const relative = sfcOffset - templateStart;
  if (relative < 0 || relative > template.length) return null;
  return utf16ToUtf8Offset(template, relative);
}
