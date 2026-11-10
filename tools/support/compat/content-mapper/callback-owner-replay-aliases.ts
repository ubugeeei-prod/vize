/** #12520: a component-props `v-for` replay no longer maps its loop aliases. */
import assert from "node:assert/strict";

type Row = number[];

/** UTF-8 byte ranges of the alias lists of every component-props loop replay. */
function replayAliasRanges(text: string): Array<[number, number]> {
  const ranges: Array<[number, number]> = [];
  const replay = /^([ \t]*)\/\/ Component props in v-for scope: [^\n]*\n/gmu;
  for (const match of text.matchAll(replay)) {
    const at = match.index + match[0].length;
    const open = text.indexOf("for (const [", at);
    assert.notEqual(open, -1, "a loop replay opens its loop");
    const start = open + "for (const [".length;
    const close = text.indexOf("] of __vize_v_for_source_", start);
    assert.notEqual(close, -1, "a loop replay iterates its captured source");
    assert.equal(text.slice(start, close).includes("\n"), false);
    ranges.push([Buffer.byteLength(text.slice(0, start)), Buffer.byteLength(text.slice(0, close))]);
  }
  return ranges;
}

/**
 * `before` with the mapped alias rows of its loop replays removed and the
 * unmapped-gap ignore rows on either side of each removed row merged, or
 * `undefined` when it maps no replay alias.
 */
export function withoutReplayAliasRows(
  before: Record<string, unknown>,
): Record<string, unknown> | undefined {
  assert.equal(typeof before.text, "string");
  assert.ok(Array.isArray(before.mappings));
  const ranges = replayAliasRanges(before.text as string);
  const inside = (row: Row) =>
    ranges.some(([start, end]) => start <= row[0]! && row[0]! + row[1]! <= end);
  const removed = (before.mappings as Row[]).filter(inside);
  if (removed.length === 0) return undefined;
  const expected = structuredClone(before);
  expected.mappings = (before.mappings as Row[]).filter((row) => !inside(row));
  const table = expected.diagnosticDirectives as { directives: Row[] } | undefined;
  if (table) {
    let rows = table.directives;
    for (const [start, length] of removed) {
      const left = rows.findIndex((row) => row[4] === 0 && row[3] === start);
      const right = rows.findIndex((row) => row[4] === 0 && row[2] === start! + length!);
      assert.ok(left !== -1 && right === left + 1, "the removed row splits one ignored gap");
      rows = [
        ...rows.slice(0, left),
        [...rows[left]!.slice(0, 3), rows[right]![3]!, ...rows[left]!.slice(4)],
        ...rows.slice(right + 1),
      ];
    }
    table.directives = rows;
  }
  return expected;
}
