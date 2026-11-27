/**
 * Keeping the compiler's Source Map v3 document valid through the edits
 * `generateOutput` makes to the emitted module (#3399).
 *
 * The native compiler returns a map for the module it emitted. `generateOutput`
 * then rewrites that module: it prepends a `<style>` injection, splices style
 * imports in after the last static import, rewrites `export default` to
 * `const _sfc_main =`, and appends `__scopeId`, CSS-module wiring and HMR
 * code. All of those are whole-line insertions or same-line
 * substitutions, which is exactly the class of edit a v3 `mappings` string can
 * absorb without being decoded:
 *
 * - Lines are separated by `;` and each line's segments by `,`.
 * - A segment's generated column is relative to the previous segment **on the
 *   same line** and resets at every line, so an inserted line never disturbs the
 *   columns after it.
 * - The source index/line/column fields are relative to the previous *segment*,
 *   not the previous line, so inserting empty lines between segments leaves
 *   every delta untouched.
 *
 * Shifting generated lines is therefore literally inserting `;` characters, with
 * no VLQ decoding and no chance of a rounding error.
 *
 * Rather than have every call site declare where its edit landed — which would
 * silently rot the first time an edit moves — {@link MappedModule.edit} derives
 * it from the two strings: the common prefix and suffix bound the changed span
 * exactly, and the change in newline count inside that span is the line shift.
 * A same-line substitution comes out as a zero shift (columns within a line are
 * not tracked; a frame still resolves to the right authored line), and an edit
 * that *removes* lines drops the map rather than emit one that is wrong.
 */

import { countNewlines, shiftMappedLines, type SourceMapV3 } from "../../../shared/source-map.ts";
export { parseSourceMap, shiftMappedLines, type SourceMapV3 } from "../../../shared/source-map.ts";

/**
 * A module's code and the map that describes it, edited together.
 *
 * Every write goes through {@link edit}, so the map is corrected in the same
 * step that changes the code and the two cannot drift.
 */
export class MappedModule {
  code: string;
  map: SourceMapV3 | null;

  constructor(code: string, map: SourceMapV3 | null) {
    this.code = code;
    this.map = map;
  }

  /** Replace the module with `next`, realigning the map to the new line layout. */
  edit(next: string): void {
    const previous = this.code;
    this.code = next;
    if (this.map === null || next === previous) {
      return;
    }

    const limit = Math.min(previous.length, next.length);
    let prefix = 0;
    while (prefix < limit && previous.charCodeAt(prefix) === next.charCodeAt(prefix)) {
      prefix++;
    }
    let suffix = 0;
    while (
      suffix < limit - prefix &&
      previous.charCodeAt(previous.length - 1 - suffix) ===
        next.charCodeAt(next.length - 1 - suffix)
    ) {
      suffix++;
    }

    // A prologue and an epilogue written in one edit (the SSR registration)
    // leave a changed span from the first line to the last, so the epilogue's
    // lines would move every line too. The previous module survives intact in
    // such an edit, and only what precedes it moves its lines.
    const firstBreak = previous.indexOf("\n");
    if (
      firstBreak >= prefix &&
      previous.lastIndexOf("\n") < previous.length - suffix &&
      next.length > previous.length
    ) {
      const at = next.indexOf(previous);
      if (at !== -1) {
        this.map = shiftMappedLines(this.map, 0, countNewlines(next.slice(0, at)));
        return;
      }
    }

    const removed = previous.slice(prefix, previous.length - suffix);
    const inserted = next.slice(prefix, next.length - suffix);
    const addedLines = countNewlines(inserted) - countNewlines(removed);
    if (addedLines === 0) {
      return;
    }
    if (addedLines < 0) {
      // No edit in `generateOutput` removes lines. If one ever does, dropping
      // the map is the only honest answer: the shift cannot express it.
      this.map = null;
      return;
    }

    // Lines strictly after the edit point move down. When the edit starts at a
    // line start, that line moves too; otherwise its head stayed put, so the
    // shift begins on the following line.
    const editLine = countNewlines(previous.slice(0, prefix));
    const startsAtLineStart = prefix === 0 || previous.charCodeAt(prefix - 1) === 10;
    this.map = shiftMappedLines(this.map, startsAtLineStart ? editLine : editLine + 1, addedLines);
  }

  /** Replace `code[start, end)`, in UTF-16 indices. */
  replace(start: number, end: number, text: string): void {
    this.edit(`${this.code.slice(0, start)}${text}${this.code.slice(end)}`);
  }
}

const BASE64_DIGITS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

function vlq(value: number): string {
  let rest = value < 0 ? (-value << 1) | 1 : value << 1;
  let encoded = "";
  do {
    const digit = rest & 31;
    rest >>>= 5;
    encoded += BASE64_DIGITS[rest > 0 ? digit | 32 : digit];
  } while (rest > 0);
  return encoded;
}

/**
 * The map of a module whose code is `source` as written in `file`.
 *
 * A bundler names a module's original after its id unless `load` supplies a
 * map, and a Vize SFC id carries a query. A segment at every non-blank column
 * keeps each position a later map traces through it exact.
 */
export function identitySourceMap(file: string, source: string): SourceMapV3 {
  const lines: string[] = [];
  let previousLine = 0;
  let previousColumn = 0;
  for (const [line, text] of source.split("\n").entries()) {
    let segments = "";
    let generatedColumn = 0;
    for (let column = 0; column < text.length; column++) {
      const code = text.charCodeAt(column);
      if (code === 32 || code === 9 || code === 13) continue;
      segments += `${segments ? "," : ""}${vlq(column - generatedColumn)}A${vlq(line - previousLine)}${vlq(column - previousColumn)}`;
      generatedColumn = column;
      previousLine = line;
      previousColumn = column;
    }
    lines.push(segments);
  }
  return {
    version: 3,
    sources: [file],
    sourcesContent: [source],
    names: [],
    mappings: lines.join(";"),
  };
}

const BASE64_VALUES = new Map(
  Array.from({ length: BASE64_DIGITS.length }, (_, value) => [BASE64_DIGITS[value], value]),
);

/** A decoded segment: generated column, then source index, line and column when mapped. */
type Segment = number[];

function decodeMappings(mappings: string): Segment[][] {
  const lines: Segment[][] = [];
  const state = [0, 0, 0, 0, 0];
  for (const text of mappings.split(";")) {
    const segments: Segment[] = [];
    state[0] = 0;
    for (const field of text.split(",")) {
      if (!field) continue;
      const segment: Segment = [];
      let value = 0;
      let shift = 0;
      for (const digit of field) {
        const bits = BASE64_VALUES.get(digit) ?? 0;
        value += (bits & 31) << shift;
        if (bits & 32) {
          shift += 5;
          continue;
        }
        const index = segment.length;
        state[index] += value & 1 ? -(value >>> 1) : value >>> 1;
        segment.push(state[index]);
        value = 0;
        shift = 0;
      }
      segments.push(segment);
    }
    lines.push(segments);
  }
  return lines;
}

function encodeMappings(lines: Segment[][]): string {
  let source = 0;
  let sourceLine = 0;
  let sourceColumn = 0;
  return lines
    .map((segments) => {
      let column = 0;
      return segments
        .map(([generated, index, line, original]) => {
          const encoded = `${vlq(generated - column)}${vlq(index - source)}${vlq(line - sourceLine)}${vlq(original - sourceColumn)}`;
          column = generated;
          source = index;
          sourceLine = line;
          sourceColumn = original;
          return encoded;
        })
        .join(",");
    })
    .join(";");
}

/**
 * The map of a second compile step over code the `inner` map already traces.
 *
 * `outer` describes the step's output against its input, and `inner` that
 * input against the authored file, so each outer position is traced through
 * `inner` to the authored one. A position `inner` does not map stays unmapped
 * rather than borrow a neighbour's line.
 */
export function composeSourceMaps(outer: SourceMapV3, inner: SourceMapV3): SourceMapV3 {
  const traced = decodeMappings(inner.mappings);
  const lines = decodeMappings(outer.mappings).map((segments) => {
    const composed: Segment[] = [];
    for (const [generated, , line, column] of segments) {
      if (line === undefined) continue;
      // The inner segment that covers the column is the last one starting at or before it.
      let covering: Segment | undefined;
      for (const segment of traced[line] ?? []) {
        if (segment[0] > column) break;
        covering = segment;
      }
      if (covering && covering.length >= 4) {
        composed.push([generated, covering[1], covering[2], covering[3]]);
      }
    }
    return composed;
  });
  return {
    version: 3,
    ...(inner.file === undefined ? {} : { file: inner.file }),
    sources: inner.sources,
    ...(inner.sourcesContent === undefined ? {} : { sourcesContent: inner.sourcesContent }),
    names: [],
    mappings: encodeMappings(lines),
  };
}
