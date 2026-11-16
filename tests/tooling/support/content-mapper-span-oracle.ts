import assert from "node:assert/strict";

type Span = readonly [number, number, number, number, number, number];
type Access = { length: number; originalStart: number; generatedStart: number };

/** Require split and joined verbatim spans to project the same authored bytes. */
export function assertLinearVerbatimMappings(
  authored: Buffer,
  generated: Buffer,
  mappings: readonly Span[],
  accesses: readonly Access[],
  features: number,
): void {
  for (const { length, originalStart, generatedStart } of accesses) {
    const owners = mappings.filter(
      ([, , at, mappedLength, kind, spanFeatures]) =>
        kind === 0 &&
        spanFeatures === features &&
        at <= originalStart &&
        at + mappedLength >= originalStart + length,
    );
    assert.equal(owners.length, 1, "one verbatim mapping owns the authored range");
    const [at, mappedLength, originalAt, originalLength] = owners[0];
    assert.equal(mappedLength, originalLength, "verbatim mapping keeps equal byte lengths");
    assert.equal(
      at + originalStart - originalAt,
      generatedStart,
      "split and joined spans project the same exact UTF-8 byte range",
    );
    assert.deepEqual(
      generated.subarray(generatedStart, generatedStart + length),
      authored.subarray(originalStart, originalStart + length),
    );
  }
}
