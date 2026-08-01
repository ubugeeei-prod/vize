/**
 * Keeping the compiler's Source Map v3 document valid through the edits
 * `generateOutput` makes to the emitted module (#3399).
 *
 * The native compiler returns a map for the module it emitted. `generateOutput`
 * then rewrites that module: it prepends a `<style>` injection, splices style
 * imports in after the last static import, inserts CSS-module wiring before the
 * default export, rewrites `export default` to `const _sfc_main =`, and appends
 * `__scopeId`/HMR code. All of those are whole-line insertions or same-line
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

/** A parsed Source Map v3 document, as Rollup/Vite want it on a hook result. */
export interface SourceMapV3 {
  version: number;
  file?: string;
  sources: string[];
  sourcesContent?: (string | null)[];
  names: string[];
  mappings: string;
}

function isSourceMapV3(value: unknown): value is SourceMapV3 {
  if (value === null || typeof value !== "object") {
    return false;
  }
  const map = value as Partial<SourceMapV3>;
  return (
    map.version === 3 &&
    Array.isArray(map.sources) &&
    Array.isArray(map.names) &&
    typeof map.mappings === "string"
  );
}

/**
 * Parse a compiler-produced map, or `null` when there is nothing usable.
 *
 * A malformed map is worse than no map — Vite would chain garbage into the
 * bundle's map — so anything that is not a v3 document is dropped.
 */
export function parseSourceMap(json: string | undefined | null): SourceMapV3 | null {
  if (!json) {
    return null;
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(json);
  } catch {
    return null;
  }
  return isSourceMapV3(parsed) ? parsed : null;
}

function countNewlines(text: string): number {
  let total = 0;
  for (let index = text.indexOf("\n"); index !== -1; index = text.indexOf("\n", index + 1)) {
    total++;
  }
  return total;
}

/**
 * Insert `count` unmapped generated lines at generated line `atLine`.
 *
 * Returns the map unchanged when the insertion lands past the last mapped line,
 * because nothing after it needs moving.
 */
export function shiftMappedLines(map: SourceMapV3, atLine: number, count: number): SourceMapV3 {
  if (count <= 0) {
    return map;
  }
  const groups = map.mappings.split(";");
  if (atLine >= groups.length) {
    return map;
  }
  const shifted = [...groups.slice(0, atLine), ...Array(count).fill(""), ...groups.slice(atLine)];
  return { ...map, mappings: shifted.join(";") };
}

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
}
