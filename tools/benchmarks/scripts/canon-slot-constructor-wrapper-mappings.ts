/** Expected-side admission for the known #11675 additive mapping transition. */
import assert from "node:assert/strict";
import { isDeepStrictEqual } from "node:util";
import { assertWrapperMappingProducer } from "../../support/compat/content-mapper/wrapper-mapping-producer.ts";

type Tuple = [number, number, number, number, number, number];
type Image = { text: string; rows: Tuple[] };
type Contract = {
  generated: string;
  original: number;
  originalLength: number;
  kind: number;
  features: number;
};
const bytes = (image: Image, start: number, length: number) =>
  Buffer.from(image.text)
    .subarray(start, start + length)
    .toString("utf8");
function contract(image: Image, row: Tuple, canonical: (text: string) => string): Contract {
  const [generated, length, original, originalLength, kind, features] = row;
  return {
    generated: canonical(bytes(image, generated, length)),
    original,
    originalLength,
    kind,
    features,
  };
}
function ordered(image: Image) {
  let end = 0;
  for (const [generated, length] of image.rows) {
    assert.ok(
      length > 0 && end <= generated && generated + length <= Buffer.byteLength(image.text),
      "strict wrapper mapping order and bounds",
    );
    end = generated + length;
  }
}
function wrapped(image: Image, row: Tuple, previous?: Tuple, next?: Tuple) {
  const [start, length] = row;
  return (
    row[4] === 0 &&
    start > 0 &&
    bytes(image, start - 1, 1) === "(" &&
    bytes(image, start + length, 1) === ")" &&
    (!previous || previous[0] + previous[1] <= start - 1) &&
    (!next || start + length < next[0])
  );
}
export function assertConstructorWrapperMappings(
  before: Image,
  current: Image,
  canonical: (text: string) => string,
  producer: Uint8Array,
): void {
  assertWrapperMappingProducer(producer);
  ordered(before);
  ordered(current);
  const expected: Contract[] = [];
  let cursor = 0;
  for (const [index, old] of before.rows.entries()) {
    const retained = contract(before, old, canonical);
    // Relocate an old row only by its complete existing contract and order.
    // A possible preceding atom is not itself accepted until expected comparison.
    const candidateIndex = [cursor, cursor + 1].find(
      (at) =>
        current.rows[at] &&
        isDeepStrictEqual(contract(current, current.rows[at]!, canonical), retained),
    );
    assert.ok(
      candidateIndex !== undefined,
      "every historical mapping contract must remain ordered and exact",
    );
    const candidate = current.rows[candidateIndex]!;
    const add = wrapped(before, old, before.rows[index - 1], before.rows[index + 1]);
    if (add) {
      assert.ok(
        wrapped(current, candidate),
        "certified wrapper parentheses must remain in current code",
      );
      const atom = (generated: string): Contract => ({
        generated,
        original: old[2],
        originalLength: old[3],
        kind: 1,
        features: 0,
      });
      expected.push(atom("("), retained, atom(")"));
      // Require exact byte adjacency as well as whole ordered contracts.
      assert.deepEqual(
        current.rows[cursor],
        [candidate[0] - 1, 1, old[2], old[3], 1, 0],
        "exact opening wrapper atom",
      );
      assert.deepEqual(
        current.rows[cursor + 2],
        [candidate[0] + candidate[1], 1, old[2], old[3], 1, 0],
        "exact closing wrapper atom",
      );
      cursor += 3;
    } else {
      assert.equal(candidateIndex, cursor, "no foreign mapping may precede an existing row");
      expected.push(retained);
      cursor++;
    }
  }
  assert.deepEqual(
    current.rows.map((row) => contract(current, row, canonical)),
    expected,
    "whole ordered #11675 mapping successor",
  );
}
