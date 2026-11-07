import type { NativeLintFix } from "./model.js";

/** Native UTF-8 text cannot preserve an unpaired UTF-16 surrogate. */
export function assertSourceText(source: string): void {
  if (/[\uD800-\uDFFF]/u.test(source)) {
    throw new Error("Vize cannot lint source containing an unpaired UTF-16 surrogate");
  }
}

/** Convert only requested UTF-8 boundaries, with one scan of the UTF-16 text. */
export function mapSourceOffsets(source: string, offsets: readonly number[]): Map<number, number> {
  const result = new Map<number, number>();
  let index = 0;
  let bytes = 0;
  for (const offset of [...new Set(offsets)].sort((left, right) => left - right)) {
    if (!Number.isSafeInteger(offset) || offset < 0) {
      throw new Error("Vize returned an invalid source offset");
    }
    while (bytes < offset && index < source.length) {
      const point = source.codePointAt(index)!;
      bytes += point <= 0x7f ? 1 : point <= 0x7ff ? 2 : point <= 0xffff ? 3 : 4;
      index += point > 0xffff ? 2 : 1;
    }
    if (bytes !== offset) throw new Error("Vize source offset is not a UTF-8 boundary");
    result.set(offset, index);
  }
  return result;
}

/** Refuse invalid or overlapping native edits before handing them to a fixer. */
export function mapTextEdits(
  edits: NativeLintFix["edits"],
  offsets: ReadonlyMap<number, number>,
): { range: [number, number]; text: string }[] {
  let previousEnd = -1;
  let previousStart = -1;
  return [...edits]
    .sort((left, right) => left.start - right.start || left.end - right.end)
    .map((edit) => {
      const start = offsets.get(edit.start);
      const end = offsets.get(edit.end);
      if (
        start === undefined ||
        end === undefined ||
        start > end ||
        start < previousEnd ||
        start === previousStart ||
        typeof edit.newText !== "string"
      )
        throw new Error("Vize returned invalid or overlapping ESLint text edits");
      previousStart = start;
      previousEnd = end;
      return { range: [start, end], text: edit.newText };
    });
}
