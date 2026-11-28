import { dumpLines, linesCovering, type PageKind, type DumpLine } from "./dumpLines";
import type { LadderPage, Rung } from "./ladder";
import type { Range } from "./offsets";

export interface TracePage {
  rung: Rung;
  page: LadderPage;
  lines: DumpLine[];
}

/** Keep each layer's current structural page; auxiliary pages do not replace it. */
export function tracePages(rungs: Rung[], selected: LadderPage | null): TracePage[] {
  return rungs.flatMap((rung) => {
    const structural = rung.pages.filter((page) =>
      ["surface", "disegno", "impeto"].includes(page.kind),
    );
    const page = structural.find((page) => page.key === selected?.key) ?? structural.at(-1);
    return page ? [{ rung, page, lines: dumpLines(page.kind, page.text) }] : [];
  });
}

/**
 * Span correspondence, not node identity: exact matches first, then contained
 * operations, then the smallest enclosing operation. Never match siblings
 * merely because both overlap a larger ancestor.
 */
export function traceMatches(lines: DumpLine[], span: Range | null, kind?: PageKind): number[] {
  if (!span) return [];
  const mapped = lines.filter((line) => line.span && line.span.end > line.span.start);
  if (kind === "surface")
    return mapped
      .filter((line) => line.span!.start < span.end && line.span!.end > span.start)
      .map((line) => line.index);
  const exact = mapped.filter(
    (line) => line.span!.start === span.start && line.span!.end === span.end,
  );
  if (exact.length) return exact.map((line) => line.index);
  const inside = mapped.filter(
    (line) => line.span!.start >= span.start && line.span!.end <= span.end,
  );
  if (inside.length) return inside.map((line) => line.index);
  const enclosing = mapped.filter(
    (line) => line.span!.start <= span.start && line.span!.end >= span.end,
  );
  const width = enclosing.reduce(
    (smallest, line) => Math.min(smallest, line.span!.end - line.span!.start),
    Infinity,
  );
  return enclosing
    .filter((line) => line.span!.end - line.span!.start === width)
    .map((line) => line.index);
}

/** Prefer semantic operations to the coarser, line-based L1 surface. */
export function cursorSpan(pages: TracePage[], offset: number | null): Range | null {
  if (offset === null) return null;
  for (const id of ["l2", "l3", "l1"]) {
    const page = pages.find(({ rung }) => rung.id === id);
    if (!page) continue;
    const index = linesCovering(page.lines, offset)[0];
    if (index !== undefined) return page.lines[index].span;
  }
  return null;
}
