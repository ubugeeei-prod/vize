/**
 * Reply shape by projection family: the mapped spans, unmapped-gap ignores
 * and authored directives each emitter construct contributes. TypeScript
 * decodes every span and directive tuple separately, so a construct that
 * fragments its spans costs request wait even when its bytes shrink.
 *
 * The byte probe writes `<sfc>.tsx.shape.json` beside each dumped text; a span
 * or gap belongs to the family of the statement at its start
 * (`typecheck-cost-families.ts`).
 */

import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import { sectionStarts, siteFamily, sourceText } from "./typecheck-cost-families.ts";
import { familyLabel } from "./typecheck-cost-sites.ts";

/** Virtual starts of one reply's tuples. */
export type ShapeSidecar = { spans: number[]; ignoreGaps: number[]; directives: number[] };
export type FamilyShape = { spans: number; ignoreGaps: number; directives: number };

const rows = (value: unknown): unknown[][] =>
  Array.isArray(value) ? value.filter((row): row is unknown[] => Array.isArray(row)) : [];

export function shapeSidecar(result: Record<string, unknown>): ShapeSidecar {
  const directives = rows(
    (result.diagnosticDirectives as { directives?: unknown } | undefined)?.directives,
  );
  const gap = (row: unknown[]) => row[0] === 0 && row[1] === 0 && row[4] === 0;
  return {
    spans: rows(result.mappings).map((row) => Number(row[0])),
    ignoreGaps: directives.filter(gap).map((row) => Number(row[2])),
    directives: directives.filter((row) => !gap(row)).map((row) => Number(row[2])),
  };
}

function* shapeFiles(dir: string): Generator<string> {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) yield* shapeFiles(path);
    else if (entry.name.endsWith(".tsx.shape.json")) yield path;
  }
}

/** Tuple counts per family over every dumped SFC under `dir`. */
export function familyShapes(dir: string): Map<string, FamilyShape> {
  const totals = new Map<string, FamilyShape>();
  if (!existsSync(dir)) return totals;
  for (const shapePath of shapeFiles(dir)) {
    const textPath = shapePath.slice(0, -".shape.json".length);
    if (!existsSync(textPath)) continue;
    const text = sourceText(readFileSync(textPath));
    const sections = sectionStarts(text);
    const shape = JSON.parse(readFileSync(shapePath, "utf8")) as ShapeSidecar;
    for (const key of ["spans", "ignoreGaps", "directives"] as const) {
      for (const start of shape[key]) {
        const family = familyLabel(siteFamily(text, sections, start));
        const entry = totals.get(family) ?? { spans: 0, ignoreGaps: 0, directives: 0 };
        entry[key] += 1;
        totals.set(family, entry);
      }
    }
  }
  return totals;
}

const integer = (value: number) => value.toLocaleString("en-US");
const delta = (base: number, head: number) =>
  base === head ? "0" : `${head > base ? "+" : "−"}${integer(Math.abs(head - base))}`;

/** One Markdown table per corpus: base and head tuple counts per family, largest first. */
export function renderReplyShape(
  corpus: string,
  base: Map<string, FamilyShape>,
  head: Map<string, FamilyShape>,
): string {
  const empty: FamilyShape = { spans: 0, ignoreGaps: 0, directives: 0 };
  const families = [...new Set([...base.keys(), ...head.keys()])].sort((left, right) => {
    const size = (shape: FamilyShape) => shape.spans + shape.ignoreGaps + shape.directives;
    return (
      size(head.get(right) ?? empty) - size(head.get(left) ?? empty) || (left < right ? -1 : 1)
    );
  });
  const lines = [
    `#### ${corpus}: reply tuples by family (tsc probe)`,
    "",
    "| family | spans base | spans head | Δ | ignore gaps base | ignore gaps head | Δ | directives head |",
    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
  ];
  for (const family of families) {
    const before = base.get(family) ?? empty;
    const after = head.get(family) ?? empty;
    lines.push(
      `| ${family} | ${integer(before.spans)} | ${integer(after.spans)} | ${delta(before.spans, after.spans)} | ${integer(before.ignoreGaps)} | ${integer(after.ignoreGaps)} | ${delta(before.ignoreGaps, after.ignoreGaps)} | ${integer(after.directives)} |`,
    );
  }
  return `${lines.join("\n")}\n`;
}
