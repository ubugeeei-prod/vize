/**
 * Minimal source map decoder.
 *
 * Hand-rolled rather than pulled from `@jridgewell/sourcemap-codec` so the
 * benchmark harness adds no dependency to `pnpm-lock.yaml` and runs from a
 * clean checkout. Only what the scale verification needs is implemented:
 * decode `mappings` into per-generated-line segments, then look up the segment
 * covering a generated position.
 */

const BASE64_CHARS = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
// The alphabet is ASCII by definition, so indexing by code unit is exact and
// avoids iterating the string as grapheme clusters.
const BASE64_LOOKUP = new Map();
for (let index = 0; index < BASE64_CHARS.length; index++) {
  BASE64_LOOKUP.set(BASE64_CHARS[index], index);
}

function decodeVlqs(segment) {
  const values = [];
  let shift = 0;
  let value = 0;

  for (let cursor = 0; cursor < segment.length; cursor++) {
    const char = segment[cursor];
    const digit = BASE64_LOOKUP.get(char);
    if (digit === undefined) {
      throw new Error(`invalid base64 VLQ character: ${JSON.stringify(char)}`);
    }
    value += (digit & 31) << shift;
    if (digit & 32) {
      shift += 5;
      continue;
    }
    const negative = value & 1;
    value >>>= 1;
    values.push(negative ? -value : value);
    value = 0;
    shift = 0;
  }

  return values;
}

/**
 * Decode `mappings` into `lines[generatedLine] = segments[]`.
 *
 * Source index, source line, and source column are cumulative across the whole
 * mappings string; only the generated column resets per line. Segments with
 * fewer than four fields carry no source and are kept as gaps so a lookup
 * landing on one reports "unmapped" instead of silently borrowing a neighbour.
 */
export function decodeMappings(mappings) {
  const lines = [];
  let sourceIndex = 0;
  let sourceLine = 0;
  let sourceColumn = 0;

  for (const lineText of mappings.split(";")) {
    const segments = [];
    let generatedColumn = 0;

    for (const segmentText of lineText.split(",")) {
      if (segmentText === "") continue;
      const fields = decodeVlqs(segmentText);
      generatedColumn += fields[0];
      if (fields.length >= 4) {
        sourceIndex += fields[1];
        sourceLine += fields[2];
        sourceColumn += fields[3];
        segments.push({ generatedColumn, sourceIndex, sourceLine, sourceColumn });
      } else {
        segments.push({ generatedColumn, sourceIndex: null, sourceLine: 0, sourceColumn: 0 });
      }
    }

    lines.push(segments);
  }

  return lines;
}

/** Last segment at or before `column` on `line`, or `null`. */
export function lookupSegment(decodedLines, line, column) {
  const segments = decodedLines[line];
  if (!segments || segments.length === 0) {
    return null;
  }

  let found = null;
  for (const segment of segments) {
    if (segment.generatedColumn > column) break;
    found = segment;
  }
  return found;
}

/** Generated `{ line, column }` (0-based) of `index` in `text`. */
export function positionOf(text, index) {
  let line = 0;
  let lastNewline = -1;
  for (let cursor = 0; cursor < index; cursor++) {
    if (text.charCodeAt(cursor) === 10) {
      line += 1;
      lastNewline = cursor;
    }
  }
  return { line, column: index - lastNewline - 1 };
}
