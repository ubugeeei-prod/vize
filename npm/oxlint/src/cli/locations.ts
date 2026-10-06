export interface BridgeLocations {
  source: string;
  scriptStart: number;
  originalStart: number;
}

interface Span {
  offset: number;
  length: number;
  line: number;
  column: number;
}

interface Report {
  diagnostics: Array<{ filename?: string; labels?: Array<{ span: Span }> }>;
}

/** Restore ranges before temporary filenames are replaced or files are removed. */
export function rewriteReportedLocations(
  output: string,
  locations: ReadonlyMap<string, BridgeLocations>,
): string {
  if (locations.size === 0 || output === "") return output;
  let report: Report;
  try {
    report = JSON.parse(output) as Report;
  } catch {
    return rewriteTextLocations(output, locations);
  }
  if (report == null || !Array.isArray(report.diagnostics)) return output;

  const spans = report.diagnostics.flatMap((diagnostic) =>
    (diagnostic.labels ?? []).map((label) => ({
      original: label.span,
      mapped: mapSpan(label.span, locations.get(diagnostic.filename ?? "")),
    })),
  );
  let index = 0;
  // Miette's JSON reporter emits these complete flat label objects. Parse the
  // document first and bind every match to its actual diagnostic/label ordinal.
  // Replace only numeric fields: all messages, metadata, whitespace and order stay exact.
  const rewritten = output.replace(/"span"\s*:\s*(\{[^{}]*\})/gu, (whole, raw: string) => {
    const span = spans[index++];
    if (span == null || JSON.stringify(JSON.parse(raw)) !== JSON.stringify(span.original))
      throw new Error("Oxlint JSON label ownership changed; refusing a partial range rewrite.");
    const mapped = span.mapped;
    if (mapped == null) return whole;
    return whole.replace(
      /("(offset|length|line|column)"\s*:\s*)\d+/gu,
      (_value, prefix: string, key: keyof Span) => `${prefix}${mapped[key]}`,
    );
  });
  if (index !== spans.length)
    throw new Error("Oxlint JSON labels could not all be bound to their reported spans.");
  return rewritten;
}

function mapSpan(span: Span, map: BridgeLocations | undefined): Span | undefined {
  if (map == null) return undefined;
  const start = originalIndex(span.offset, map);
  const end = originalIndex(span.offset + span.length, map);
  if (start == null || end == null || end < start) return undefined;
  const before = map.source.slice(0, start);
  return {
    offset: Buffer.byteLength(before, "utf8"),
    length: Buffer.byteLength(map.source.slice(start, end), "utf8"),
    line: before.split("\n").length,
    column: start - before.lastIndexOf("\n"),
  };
}

function originalIndex(offset: number, map: BridgeLocations): number | undefined {
  if (!Number.isSafeInteger(offset) || offset < 0) return undefined;
  // The mirror is ASCII and CR/LF: byte offsets there are original UTF-16 indices.
  if (offset >= map.scriptStart && offset <= map.scriptStart + map.source.length) {
    const index = offset - map.scriptStart;
    const previous = map.source.charCodeAt(index - 1);
    const current = map.source.charCodeAt(index);
    if (previous >= 0xd800 && previous <= 0xdbff && current >= 0xdc00 && current <= 0xdfff)
      return undefined;
    return index;
  }
  // Ordinary JS diagnostics refer to the exact copied source after the bridge.
  const relative = offset - map.originalStart;
  const bytes = Buffer.from(map.source, "utf8");
  if (relative < 0 || relative > bytes.length) return undefined;
  const before = bytes.subarray(0, relative).toString("utf8");
  if (!Buffer.from(before, "utf8").equals(bytes.subarray(0, relative))) return undefined;
  return before.length;
}

function rewriteTextLocations(
  output: string,
  locations: ReadonlyMap<string, BridgeLocations>,
): string {
  let active: BridgeLocations | undefined;
  return output
    .split("\n")
    .map((line) => {
      if (line !== "" && line.trimStart() === line) active = locations.get(line);
      // Stylish's file header establishes ownership of following diagnostic rows.
      const stylish = /^(\s+)(\d+):(\d+)(\s+(?:error|warning)\s+.*\bvize\()/u.exec(line);
      if (stylish != null && active != null) {
        const mapped = textPosition(Number(stylish[2]), Number(stylish[3]), active);
        if (mapped != null)
          return line.replace(/^(\s+)\d+:\d+/u, `$1${mapped.line}:${mapped.column}`);
      }
      // Auto/unix output carries the owning filename on each diagnostic line.
      const plain = /^(.*):(\d+):(\d+)(: (?:error|warning) vize\()/u.exec(line);
      const map = plain == null ? undefined : locations.get(plain[1]);
      if (plain != null && map != null) {
        const mapped = textPosition(Number(plain[2]), Number(plain[3]), map);
        if (mapped != null)
          return `${plain[1]}:${mapped.line}:${mapped.column}${plain[4]}${line.slice(plain[0].length)}`;
      }
      return line;
    })
    .join("\n");
}

function textPosition(line: number, column: number, map: BridgeLocations) {
  const lines = map.source.split("\n");
  const originalLine = lines.length + 1;
  const mappedLine = line >= originalLine ? line - originalLine + 1 : line;
  const mappedColumn = line === 1 ? column - map.scriptStart : column;
  if (
    mappedLine < 1 ||
    mappedLine > lines.length ||
    mappedColumn < 1 ||
    mappedColumn > lines[mappedLine - 1].length + 1
  )
    return undefined;
  return { line: mappedLine, column: mappedColumn };
}
