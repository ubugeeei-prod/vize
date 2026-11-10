/**
 * Expected-side admission for #11675 plus the named component-export successor
 * and the #12479 unchecked slot replay removal.
 */
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
function disjointPatternGaps(image: Image, row: Tuple, cursor: number): [Tuple, Tuple] | undefined {
  const [start, length, original, originalLength, kind, features] = row;
  if (original < 2 || kind !== 0 || features !== 1048575) return;
  const opening: Tuple = [start - 2, 2, original - 2, 2, kind, features];
  const closing: Tuple = [start + length, 2, original + originalLength, 2, kind, features];
  if (
    isDeepStrictEqual(image.rows[cursor], opening) &&
    isDeepStrictEqual(image.rows[cursor + 2], closing) &&
    bytes(image, opening[0], opening[1]) === "{ " &&
    bytes(image, closing[0], closing[1]) === " }"
  ) {
    return [opening, closing];
  }
}
// #12479: the component-props pass no longer replays a slot whose body checks
// no component. Such a replay (with the host older releases declared for it)
// held only `void` reads, so its rows leave with it.
const UNCHECKED_REPLAY =
  /\n( +)\/\/ Component props in v-slot scope: #[^\n]*\n(?:\1const __vize_slot_props_host_\d+ = (?:(?!\n\1(?:void function |\/\/ ))[^])*?\n\1void __vize_slot_props_host_\d+;\n)?\1void function _slot_props_[^\n]*\{\n(?:\1 {2}void [^\n]*;\n)*\1\};\n/gu;
function uncheckedReplays(image: Image): Array<[number, number]> {
  return [...image.text.matchAll(UNCHECKED_REPLAY)].map((match) => {
    const start = Buffer.byteLength(image.text.slice(0, match.index));
    return [start, start + Buffer.byteLength(match[0])];
  });
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
  assert.deepEqual(uncheckedReplays(current), [], "no unchecked slot replay remains");
  const removed = uncheckedReplays(before);
  const expected: Contract[] = [];
  let cursor = 0;
  for (const [index, old] of before.rows.entries()) {
    if (removed.some(([start, end]) => start <= old[0] && old[0] + old[1] <= end)) continue;
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
    const gaps = disjointPatternGaps(current, candidate, cursor);
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
    } else if (gaps) {
      assert.equal(candidateIndex, cursor + 1, "the retained binding stays between its exact gaps");
      expected.push(
        contract(current, gaps[0], canonical),
        retained,
        contract(current, gaps[1], canonical),
      );
      cursor += 3;
    } else {
      assert.equal(candidateIndex, cursor, "no foreign mapping may precede an existing row");
      expected.push(retained);
      cursor++;
    }
  }
  const declarations = [
    ...current.text.matchAll(/^declare const ([A-Za-z_$][\w$]*): typeof __vize_component__;$/gmu),
  ];
  const exported = declarations.find((match) =>
    current.text.includes(`type ${match[1]}Component =`),
  );
  if (exported) {
    assert.equal(
      declarations.filter((match) => match[1] === exported[1]).length,
      1,
      "one named component export owns the navigation successor",
    );
    const name = exported[1]!;
    const start = Buffer.byteLength(
      current.text.slice(0, exported.index! + "declare const ".length),
    );
    const navigation: Tuple = [start, Buffer.byteLength(name), 0, 0, 1, 248];
    assert.deepEqual(
      current.rows[cursor],
      navigation,
      "the named component export maps only its exact name to the SFC start",
    );
    expected.push(contract(current, navigation, canonical));
    cursor++;
  }
  assert.deepEqual(
    current.rows.map((row) => contract(current, row, canonical)),
    expected,
    "whole ordered #11675 mapping successor",
  );
}
