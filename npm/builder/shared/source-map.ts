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

export function countNewlines(text: string): number {
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

/** Prefix a native JSX module with unmapped generated whole lines.
 * The actual style injection supplies the prefix; authored UTF-16 columns are
 * unchanged because the module resumes at column zero on a new generated line.
 */
export function prependMappedJsxCode(
  code: string,
  mapJson: string | undefined | null,
  prefix: string,
): { code: string; map: string | null } {
  const output = prefix + code;
  if (!prefix) return { code: output, map: mapJson ?? null };
  const map = parseSourceMap(mapJson);
  // This contract only accepts whole-line prefixes, never approximate columns.
  if (!map || !prefix.endsWith("\n")) return { code: output, map: null };
  return {
    code: output,
    map: JSON.stringify(shiftMappedLines(map, 0, countNewlines(prefix))),
  };
}
