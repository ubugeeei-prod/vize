import remapping from "@jridgewell/remapping";
import {
  TraceMap,
  decodedMappings,
  encodedMap,
  presortedDecodedMap,
  type DecodedSourceMap,
  type EncodedSourceMap,
  type SourceMapInput,
  type SourceMapSegment,
} from "@jridgewell/trace-mapping";

export type RawSourceMap = EncodedSourceMap;

/** One contiguous range in a synthetic source and its original provenance. */
export interface SourceProvenanceSpan {
  generatedStart: number;
  generatedEnd: number;
  source: string;
  sourceContent: string;
  sourceStart: number;
}

/** Parse an encoded v3 map without accepting partial or malformed objects. */
export function parseSourceMap(map: unknown): RawSourceMap | null {
  let candidate = map;
  if (typeof candidate === "string") {
    try {
      candidate = JSON.parse(candidate);
    } catch {
      return null;
    }
  }
  if (!candidate || typeof candidate !== "object") {
    return null;
  }
  const record = candidate as Record<string, unknown>;
  if (
    record.version !== 3 ||
    typeof record.mappings !== "string" ||
    !Array.isArray(record.sources) ||
    !Array.isArray(record.names)
  ) {
    return null;
  }
  return candidate as RawSourceMap;
}

/**
 * Relocate a map when host code surrounds, but does not mutate, generated code.
 * Returns null instead of exposing a known-wrong map if the code is not an
 * exact embedded span.
 */
export function offsetEmbeddedSourceMap(
  generatedCode: string,
  hostCode: string,
  map: unknown,
): RawSourceMap | null {
  const parsed = parseSourceMap(map);
  if (!parsed) return null;
  const offset = hostCode.indexOf(generatedCode);
  if (offset < 0) return null;
  if (offset === 0) return parsed;

  const prefix = hostCode.slice(0, offset);
  const lastNewline = prefix.lastIndexOf("\n");
  const lineOffset = countNewlines(prefix);
  const columnOffset = prefix.length - (lastNewline + 1);
  const mappingLines = parsed.mappings.split(";");
  if (columnOffset > 0 && mappingLines[0]) {
    mappingLines[0] = offsetFirstGeneratedColumn(mappingLines[0], columnOffset);
  }
  return {
    ...parsed,
    mappings: `${";".repeat(lineOffset)}${mappingLines.join(";")}`,
  };
}

/** Compose newest-to-oldest transformation maps into one original-source map. */
export function composeSourceMaps(...maps: unknown[]): RawSourceMap | null {
  const parsed = maps.map(parseSourceMap).filter((map): map is RawSourceMap => map !== null);
  if (parsed.length === 0) return null;
  if (parsed.length === 1) return parsed[0];
  try {
    const composed = remapping(parsed as SourceMapInput[], () => null);
    return parseSourceMap(composed.toString());
  } catch {
    return null;
  }
}

/**
 * Replace synthetic-source coordinates with their true file provenance.
 * Generated coordinates and symbol names remain byte-for-byte equivalent.
 */
export function applySourceProvenance(
  map: unknown,
  syntheticSource: string,
  spans: readonly SourceProvenanceSpan[],
): RawSourceMap | null {
  const parsed = parseSourceMap(map);
  if (!parsed || spans.length === 0) return parsed;

  const decoded = decodedMappings(new TraceMap(parsed));
  const syntheticStarts = lineStarts(syntheticSource);
  const sources: string[] = [];
  const sourcesContent: Array<string | null> = [];
  const sourceIndexes = new Map<string, number>();
  const internSource = (source: string, content: string | null): number => {
    const existing = sourceIndexes.get(source);
    if (existing !== undefined) return existing;
    const index = sources.length;
    sources.push(source);
    sourcesContent.push(content);
    sourceIndexes.set(source, index);
    return index;
  };

  const mappings: SourceMapSegment[][] = decoded.map((line) =>
    line.map((readonlySegment) => {
      const segment = [...readonlySegment] as SourceMapSegment;
      if (segment.length < 4) return segment;
      const syntheticOffset = offsetAt(syntheticStarts, segment[2], segment[3]);
      const span = syntheticOffset == null ? null : findSpan(spans, syntheticOffset);
      if (!span || syntheticOffset == null) {
        const source = parsed.sources[segment[1]] ?? null;
        if (source === null) return [segment[0]];
        const content = parsed.sourcesContent?.[segment[1]] ?? null;
        segment[1] = internSource(source, content);
        return segment;
      }

      const originalOffset = span.sourceStart + (syntheticOffset - span.generatedStart);
      const [line, column] = lineColumnAt(span.sourceContent, originalOffset);
      segment[1] = internSource(span.source, span.sourceContent);
      segment[2] = line;
      segment[3] = column;
      return segment;
    }),
  );

  const decodedMap: DecodedSourceMap = {
    version: 3,
    file: parsed.file,
    names: parsed.names,
    sources,
    sourcesContent,
    mappings,
  };
  return encodedMap(presortedDecodedMap(decodedMap));
}

function findSpan(
  spans: readonly SourceProvenanceSpan[],
  offset: number,
): SourceProvenanceSpan | null {
  let low = 0;
  let high = spans.length - 1;
  while (low <= high) {
    const middle = (low + high) >> 1;
    const span = spans[middle]!;
    if (offset < span.generatedStart) high = middle - 1;
    else if (offset >= span.generatedEnd) low = middle + 1;
    else return span;
  }
  return null;
}

function lineStarts(source: string): number[] {
  const starts = [0];
  for (let index = 0; index < source.length; index += 1) {
    if (source.charCodeAt(index) === 10) starts.push(index + 1);
  }
  return starts;
}

function offsetAt(starts: readonly number[], line: number, column: number): number | null {
  const start = starts[line];
  return start === undefined ? null : start + column;
}

function lineColumnAt(source: string, requestedOffset: number): [number, number] {
  const offset = Math.max(0, Math.min(requestedOffset, source.length));
  const starts = lineStarts(source);
  let low = 0;
  let high = starts.length;
  while (low < high) {
    const middle = (low + high) >> 1;
    if (starts[middle]! <= offset) low = middle + 1;
    else high = middle;
  }
  const line = Math.max(0, low - 1);
  return [line, offset - starts[line]!];
}

function countNewlines(value: string): number {
  let count = 0;
  for (let index = 0; index < value.length; index += 1) {
    if (value.charCodeAt(index) === 10) count += 1;
  }
  return count;
}

function offsetFirstGeneratedColumn(line: string, offset: number): string {
  const segmentEnd = line.indexOf(",");
  const firstSegment = segmentEnd < 0 ? line : line.slice(0, segmentEnd);
  const decoded = decodeVlq(firstSegment);
  if (!decoded) return line;
  const shifted = `${encodeVlq(decoded.value + offset)}${firstSegment.slice(decoded.length)}`;
  return segmentEnd < 0 ? shifted : `${shifted}${line.slice(segmentEnd)}`;
}

function decodeVlq(value: string): { value: number; length: number } | null {
  let result = 0;
  let shift = 0;
  for (let index = 0; index < value.length; index += 1) {
    const digit = decodeBase64(value.charCodeAt(index));
    if (digit < 0) return null;
    result |= (digit & 31) << shift;
    if ((digit & 32) === 0) {
      const magnitude = result >> 1;
      return { value: (result & 1) === 1 ? -magnitude : magnitude, length: index + 1 };
    }
    shift += 5;
  }
  return null;
}

function encodeVlq(value: number): string {
  let encoded = (Math.abs(value) << 1) | (value < 0 ? 1 : 0);
  let output = "";
  do {
    let digit = encoded & 31;
    encoded >>>= 5;
    if (encoded > 0) digit |= 32;
    output += encodeBase64(digit);
  } while (encoded > 0);
  return output;
}

function decodeBase64(code: number): number {
  if (code >= 65 && code <= 90) return code - 65;
  if (code >= 97 && code <= 122) return code - 97 + 26;
  if (code >= 48 && code <= 57) return code - 48 + 52;
  if (code === 43) return 62;
  if (code === 47) return 63;
  return -1;
}

function encodeBase64(value: number): string {
  if (value < 26) return String.fromCharCode(65 + value);
  if (value < 52) return String.fromCharCode(97 + value - 26);
  if (value < 62) return String.fromCharCode(48 + value - 52);
  return value === 62 ? "+" : "/";
}
