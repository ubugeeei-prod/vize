import { dumpLines, surfaceTokens, type DumpLine, type Token } from "./dumpLines";
import { utf8ToUtf16Offset, type Range } from "./offsets";
import { traceMatches } from "./sourceTrace";

export interface SurfaceFragment {
  offset: number;
  text: string;
  tokens: Token[];
  unit: DumpLine | null;
}

/**
 * Project compiler-emitted semantic spans onto the lossless L1 text. Nested
 * attributes/expressions win over their enclosing element. No new parser or
 * line-sized synthetic analysis units are introduced by the viewer.
 */
export function surfaceLines(text: string, units: DumpLine[]) {
  const ordered = units
    .filter((unit) => unit.span)
    .sort((a, b) => a.span!.end - a.span!.start - (b.span!.end - b.span!.start));
  return dumpLines("surface", text).map((line) => {
    const fragments: SurfaceFragment[] = [];
    if (!line.span) return { ...line, fragments };
    const range = line.span;
    const candidates = ordered.filter(
      (unit) => unit.span!.start < range.end && unit.span!.end > range.start,
    );
    const boundaries = [
      ...new Set([
        range.start,
        range.end,
        ...candidates.flatMap(({ span }) => [
          Math.max(range.start, span!.start),
          Math.min(range.end, span!.end),
        ]),
      ]),
    ].sort((a, b) => a - b);
    for (let i = 0; i + 1 < boundaries.length; i++) {
      const start = boundaries[i];
      const end = boundaries[i + 1];
      const offset = utf8ToUtf16Offset(line.text, start - range.start);
      const content = line.text.slice(offset, utf8ToUtf16Offset(line.text, end - range.start));
      const unit = content.trim()
        ? (candidates.find(({ span }) => span!.start <= start && span!.end >= end) ?? null)
        : null;
      fragments.push({ offset, text: content, tokens: surfaceTokens(content), unit });
    }
    return { ...line, fragments };
  });
}

export function linkedSurfaceUnits(units: DumpLine[], span: Range | null): Set<number> {
  return new Set(traceMatches(units, span));
}
