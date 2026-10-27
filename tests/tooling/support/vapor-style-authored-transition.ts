// The original #10749 packets stay immutable. #10701 adds complete authored
// modules while retaining each component, map and diagnostic producer packet.
import assert from "node:assert/strict";
import fs from "node:fs";

const root = new URL(
  "../../_fixtures/differential/compiler/vapor-literal-object-styles-10749/",
  import.meta.url,
);
const read = (file: string) => JSON.parse(fs.readFileSync(new URL(file, root), "utf8"));
const directory = "authored-module-successor-10701/";
const capture = read(`${directory}capture.json`);
type Row = Record<string, unknown> & { output: Record<string, unknown> };
export type StylePacketGroup = "main" | "attribute-case" | "review" | "priority" | "sfc";
const packetFile = (group: StylePacketGroup, side: "base" | "head") =>
  group === "main"
    ? side === "base"
      ? "observed-head.json"
      : `${directory}head.json`
    : `${directory}${side}-${group}.json`;
const samples = new Set(["arenaBytes", "expressionReparses", "counters", "phases"]);
const deterministic = (row: Row) =>
  Object.fromEntries(Object.entries(row).filter(([key]) => !samples.has(key)));

/** Check every complete current output against the retained actual successor. */
export function assertAuthoredStylePackets(group: StylePacketGroup, actual: readonly Row[]): void {
  const expected: Row[] = read(packetFile(group, "head"));
  assert.equal(actual.length, expected.length, `${group}: no missing or extra packets`);
  actual.forEach((row, index) =>
    assert.deepEqual(
      deterministic(row),
      deterministic(expected[index]),
      `${group}:${index}: complete input and output packet`,
    ),
  );
}

/** Permit only the explicitly archived authored-module boundary transition. */
export function assertPreservedStylePacket(
  group: StylePacketGroup,
  index: number,
  before: Record<string, unknown>,
  after: Record<string, unknown>,
): void {
  const transition = capture.transitions[group];
  if (!transition.indices.includes(index)) {
    assert.deepEqual(after, before, `${group}:${index}: unchanged complete packet`);
    return;
  }
  const historical: Row[] = read(packetFile(group, "base"));
  const successor: Row[] = read(packetFile(group, "head"));
  assert.deepEqual(
    before,
    historical[index].output,
    `${group}:${index}: immutable complete previous output`,
  );
  assert.deepEqual(
    after,
    successor[index].output,
    `${group}:${index}: immutable complete successor output`,
  );
  assert.deepEqual(Object.keys(after).sort(), Object.keys(before).sort());
  for (const key of Object.keys(before)) {
    if (transition.facets.includes(key))
      assert.notDeepEqual(
        after[key],
        before[key],
        `${group}:${index}:${key}: explicit changed facet`,
      );
    else assert.deepEqual(after[key], before[key], `${group}:${index}:${key}: unchanged facet`);
  }
}
