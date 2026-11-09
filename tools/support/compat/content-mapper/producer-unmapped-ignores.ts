/** Separate producer-owned virtual gaps from authored diagnostic directives. */
import assert from "node:assert/strict";
import { isDeepStrictEqual } from "node:util";

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

const navigationNeedle = Buffer.from("const __vize_props_nav_");
const navigationIgnore = "// @ts-ignore Navigation-only references; no diagnostic owner.";

/** `navigation_blocks` in `unmapped_ignores.rs`: a navigation const, its
 * `@ts-ignore` line and the one line of its own references that comment covers. */
function navigationBlocks(text: Buffer): number[][] {
  const blocks: number[][] = [];
  for (
    let start = text.indexOf(navigationNeedle);
    start >= 0;
    start = text.indexOf(navigationNeedle, start + 1)
  ) {
    const lineStart = start === 0 ? 0 : text.lastIndexOf(10, start - 1) + 1;
    if (text.subarray(lineStart, start).some((byte) => byte !== 32)) continue;
    const declarationEnd = text.indexOf(10, start) + 1;
    const commentEnd = declarationEnd === 0 ? 0 : text.indexOf(10, declarationEnd) + 1;
    if (commentEnd === 0 || commentEnd >= text.length) continue;
    const newline = text.indexOf(10, commentEnd);
    const referencesEnd = newline < 0 ? text.length : newline + 1;
    const declaration = text.subarray(start + navigationNeedle.length, declarationEnd).toString();
    const separator = declaration.indexOf(" = ");
    const index = separator < 0 ? "" : declaration.slice(0, separator);
    if (!/^[0-9]+$/u.test(index)) continue;
    const owner = `void __vize_props_nav_${index}`;
    const statements = text.subarray(commentEnd, referencesEnd).toString().trim();
    const owned =
      statements.endsWith(";") &&
      statements
        .slice(0, -1)
        .split(";")
        .every((statement) => {
          const key = statement.trimStart();
          return key.startsWith(owner) && [".", "["].includes(key[owner.length] ?? "");
        });
    const comment = text.subarray(declarationEnd, commentEnd).toString().trim();
    if (comment !== navigationIgnore || !owned) continue;
    const references = text.subarray(commentEnd, referencesEnd).toString().trimEnd();
    blocks.push([start, commentEnd + Buffer.byteLength(references)]);
  }
  return blocks;
}

function producerRows(packet: Packet, explicit: Tuple[], navigation: boolean): Tuple[] {
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
  const explicitRanges = explicit.filter((row) => row[2] < row[3]).map((row) => [row[2], row[3]]);
  // A block an explicit tuple touches keeps its gaps.
  const blocks = (navigation ? navigationBlocks(text) : []).filter(
    ([start, end]) => !explicitRanges.some(([from, to]) => from < end && start < to),
  );
  const covered = [...explicitRanges, ...blocks].sort((a, b) => a[0] - b[0]);
  const rows: Tuple[] = blocks.map(([start, end]) => [0, 0, start, end, 0, 0]);
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
  // A navigation block and the gaps it touches form one interval.
  rows.sort((a, b) => a[2] - b[2]);
  return rows.reduce<Tuple[]>((merged, row) => {
    const previous = merged.at(-1);
    if (previous && previous[3] === row[2]) previous[3] = row[3];
    else merged.push(row);
    return merged;
  }, []);
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
        producerRows(packet, [], true),
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
    // Producers before #12531's blocks ignore each gap; both forms ignore only unowned text.
    const legacy = producerRows(packet, explicit, false);
    assert.deepEqual(
      generated,
      isDeepStrictEqual(generated, legacy) ? legacy : producerRows(packet, explicit, true),
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

/** Authenticate the complete producer population in every current transform. */
export function certifyCurrentProducerPopulation(packets: Iterable<Packet>): number {
  let count = 0;
  for (const packet of packets) {
    withoutProducerUnmappedIgnores(packet, { requireComplete: true });
    count++;
  }
  return count;
}

/** Authored directives after the producer's unmapped IGNORE rows are certified. */
export function authoredDirectives(packet: Packet, requireComplete = false) {
  const certified = withoutProducerUnmappedIgnores(packet, { requireComplete });
  const table = certified.diagnosticDirectives;
  if (table == null) return null;
  const { directives, unusedExpectDirectiveDiagnostics } = table as {
    directives: Tuple[];
    unusedExpectDirectiveDiagnostics: unknown;
  };
  // The current producer sorts the whole table. Historical authored tuples
  // keep their identity; order is not the comparison key.
  const sorted = [...directives].sort((left, right) => {
    for (let index = 0; index < left.length; index += 1) {
      if (left[index] !== right[index]) return left[index] - right[index];
    }
    return 0;
  });
  return { directives: sorted, unusedExpectDirectiveDiagnostics };
}
