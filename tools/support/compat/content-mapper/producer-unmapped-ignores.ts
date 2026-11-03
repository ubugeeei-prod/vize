/** Separate producer-owned virtual gaps from authored diagnostic directives. */
import assert from "node:assert/strict";

type Tuple = [number, number, number, number, number, number];
type Packet = { text: string; mappings: number[][]; diagnosticDirectives?: unknown };

function tuples(value: unknown): Tuple[] {
  assert.ok(Array.isArray(value));
  return value.map((row) => {
    assert.ok(Array.isArray(row) && row.length === 6);
    assert.ok(row.every((n) => Number.isSafeInteger(n) && n >= 0));
    return row as Tuple;
  });
}

function producerRows(packet: Packet, explicit: Tuple[]): Tuple[] {
  const text = Buffer.from(packet.text);
  const mapped = packet.mappings
    .filter((row) => row[1] > 0)
    .map(([start, length]) => [start, start + length])
    .sort((a, b) => a[0] - b[0]);
  const gaps: number[][] = [];
  let cursor = 0;
  for (const [start, end] of mapped) {
    assert.ok(Number.isSafeInteger(start) && cursor <= text.length && end <= text.length);
    let stop = start;
    for (let position = start; position > cursor; position--) {
      const byte = text[position - 1];
      if (byte === 40) stop = position - 1;
      else if (![9, 10, 11, 12, 13, 32].includes(byte)) break;
    }
    if (cursor < stop) gaps.push([cursor, stop]);
    cursor = Math.max(cursor, end);
  }
  if (cursor < text.length) gaps.push([cursor, text.length]);
  const covered = explicit
    .filter((row) => row[2] < row[3])
    .map((row) => [row[2], row[3]])
    .sort((a, b) => a[0] - b[0]);
  const rows: Tuple[] = [];
  for (const [start, end] of gaps) {
    let position = start;
    for (const [coverStart, coverEnd] of covered) {
      if (coverEnd <= position) continue;
      if (coverStart >= end) break;
      if (position < coverStart) rows.push([0, 0, position, coverStart, 0, 0]);
      position = Math.max(position, coverEnd);
    }
    if (position < end) rows.push([0, 0, position, end, 0, 0]);
  }
  return rows;
}

export function withoutProducerUnmappedIgnores<T extends Packet>(
  packet: T,
  { requireComplete = false }: { requireComplete?: boolean } = {},
): T {
  const result = { ...packet };
  const table = packet.diagnosticDirectives;
  if (table === undefined) {
    if (requireComplete)
      assert.deepEqual(
        [],
        producerRows(packet, []),
        "synthetic ignores must retain every producer gap",
      );
    return result;
  }
  assert.ok(table !== null && typeof table === "object" && !Array.isArray(table));
  assert.deepEqual(Object.keys(table).sort(), ["directives", "unusedExpectDirectiveDiagnostics"]);
  const { directives, unusedExpectDirectiveDiagnostics } = table as {
    directives: unknown;
    unusedExpectDirectiveDiagnostics: unknown;
  };
  const rows = tuples(directives);
  assert.ok(Array.isArray(unusedExpectDirectiveDiagnostics));
  const generated = rows.filter((row) => row[0] === 0 && row[1] === 0);
  const explicit = rows.filter((row) => row[0] !== 0 || row[1] !== 0);
  // Historical packets may predate producer rows; authenticated current heads may not omit them.
  if (requireComplete || generated.length > 0) {
    assert.deepEqual(
      generated,
      producerRows(packet, explicit),
      "synthetic ignores must equal exactly the producer's unmapped gaps",
    );
  }
  if (explicit.length === 0 && unusedExpectDirectiveDiagnostics.length === 0) {
    delete result.diagnosticDirectives;
  } else {
    result.diagnosticDirectives = { directives: explicit, unusedExpectDirectiveDiagnostics };
  }
  return result;
}
