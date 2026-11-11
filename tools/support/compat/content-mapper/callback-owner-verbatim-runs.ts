import assert from "node:assert/strict";
import { joinVerbatimRuns } from "./constructor-projection-parity.ts";

type Row = number[];

/** Apply the exact packet transition caused by joining copied LF-only runs. */
export function withJoinedVerbatimRuns(
  before: Record<string, unknown>,
  authoredSource: string,
): Record<string, unknown> {
  assert.equal(typeof before.text, "string");
  assert.ok(Array.isArray(before.mappings));
  const original = before.mappings as Row[];
  const expected = structuredClone(before);
  const joined = joinVerbatimRuns(original, authoredSource, before.text as string);
  expected.mappings = joined;

  const table = expected.diagnosticDirectives as { directives: Row[] } | undefined;
  if (table && joined.length < original.length) {
    const coveredBy = (rows: Row[], start: number, end: number) =>
      rows.some(([at, length]) => at! <= start && end <= at! + length!);
    table.directives = table.directives.filter((row) => {
      assert.equal(row.length, 6);
      assert.ok(row.every(Number.isSafeInteger));
      const [sourceStart, sourceLength, start, end, kind, features] = row;
      const producerIgnore =
        sourceStart === 0 && sourceLength === 0 && kind === 0 && features === 0;
      const newlyCovered =
        producerIgnore &&
        start! < end! &&
        !coveredBy(original, start!, end!) &&
        coveredBy(joined, start!, end!);
      return !newlyCovered;
    });
  }
  return expected;
}
