import assert from "node:assert/strict";
import type { Transform } from "./content-mapper-instance-globals-10617.ts";

/** The ordinary token, never a delimiter row, certifies its diagnostic owner. */
export function parenthesisDiagnosticMappings(result: Pick<Transform, "text" | "mappings">) {
  const bytes = Buffer.from(result.text);
  const owners = result.mappings.filter(
    ([at, length]) => !(length === 1 && (bytes[at] === 0x28 || bytes[at] === 0x29)),
  );
  return owners.flatMap((span, index) => {
    if (span[5] !== 1048575) return [];
    const [start, length, original, originalLength] = span;
    const open = start - 1;
    const close = start + length;
    const previousEnd = index === 0 ? 0 : owners[index - 1][0] + owners[index - 1][1];
    const nextStart = index + 1 === owners.length ? bytes.length : owners[index + 1][0];
    if (bytes[open] !== 0x28 || bytes[close] !== 0x29 || previousEnd > open || close >= nextStart)
      return [];
    return [
      [open, 1, original, originalLength, 1, 0],
      [close, 1, original, originalLength, 1, 0],
    ];
  });
}

/** Shared late assertion; the runtime caller supplies its genuine AST-owned prefixes. */
export function assertDiagnosticMappingRoster(result: Transform, receiverPrefixes: number[][]) {
  const expected = [...receiverPrefixes, ...parenthesisDiagnosticMappings(result)].sort(
    (left, right) => left[0] - right[0],
  );
  assert.deepEqual(
    result.mappings.filter((span) => span[5] === 0),
    expected,
    "the diagnostic-only roster contains exactly AST receiver prefixes and owned generated parentheses",
  );
}

/** Expected side only: retain historical bodies and account for #11675's two atoms. */
export function withParenthesisDiagnosticMappings(historical: Transform): Transform {
  const expected = structuredClone(historical);
  const atoms = parenthesisDiagnosticMappings(historical);
  expected.mappings = [...historical.mappings, ...atoms].sort((left, right) => left[0] - right[0]);
  const table = expected.diagnosticDirectives as { directives: number[][] } | undefined;
  if (table) {
    assert.ok(Array.isArray(table.directives));
    table.directives = table.directives.flatMap((row) => {
      // Authored directives and every opaque field retain their literal values.
      if (row[0] !== 0 || row[1] !== 0) return [row];
      let ranges = [[row[2], row[3]]];
      for (const [at] of atoms)
        ranges = ranges.flatMap(([start, end]) =>
          at >= end || at + 1 <= start
            ? [[start, end]]
            : [
                [start, at],
                [at + 1, end],
              ].filter(([left, right]) => left < right),
        );
      return ranges.map(([start, end]) => [row[0], row[1], start, end, row[4], row[5]]);
    });
  }
  return expected;
}
