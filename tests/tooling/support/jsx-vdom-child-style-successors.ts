/** Exact VDOM successors; all old style producers, records and admission helpers remain intact. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import {
  contract as previous,
  assertChildStylePackets,
  assertChildPreservedStylePacket,
} from "./jsx-child-style-successors.ts";
import type { StylePacketGroup } from "./vapor-style-authored-transition.ts";
type Row = Record<string, unknown> & { output: Record<string, unknown> };
type Entry = { index: number; predecessorSha256: string; after: Row; changedPointers: string[] };
const hash = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
const bytes = fs.readFileSync(
  new URL(
    "../../_fixtures/differential/jsx/child-values-6890-vdom-style-successors.json",
    import.meta.url,
  ),
);
assert.equal(hash(bytes), "bfb4af161a436008fec5555be0e7ec9f8118df6fd20c564836b72e0aa55f71df");
export const vdomStyleContract = JSON.parse(bytes.toString()) as {
  schema: number;
  source: string[];
  run: number;
  artifact: number;
  archiveSha256: string;
  qualification: unknown;
  groups: {
    name: StylePacketGroup;
    population: number;
    member: string;
    memberSha256: string;
    records: Entry[];
  }[];
};
assert.equal(vdomStyleContract.schema, 1);
assert.equal(vdomStyleContract.source[0], "18ebd0556f55fc1fff4b7a993efa752bdabc7232");
assert.equal(vdomStyleContract.run, 41212287115);
assert.equal(vdomStyleContract.artifact, 13046741442);
assert.equal(
  vdomStyleContract.archiveSha256,
  "04bce594cb20c74ae2cc91a0a002425c3d3e9f6d39b10488a1c175d6c9528dd3",
);
assert.deepEqual(vdomStyleContract.qualification, {
  runtime: false,
  performance: false,
  wholeCorpusQualified: false,
  issue6890Complete: false,
});
assert.deepEqual(
  vdomStyleContract.groups.map((g) => [g.name, g.population]),
  previous.groups.map((g) => [g.name, g.population]),
);
const indices = [9, 21, 33, 45, 57, 69, 81, 93, 105, 117, 129, 141];
for (const group of vdomStyleContract.groups)
  assert.deepEqual(
    group.records.map((r) => r.index),
    group.name === "main" ? indices : [],
  );
// The same four measurement fields omitted by the original immutable admission.
const metrics = new Set(["arenaBytes", "expressionReparses", "counters", "phases"]);
const deterministic = (row: Row) =>
  Object.fromEntries(Object.entries(row).filter(([key]) => !metrics.has(key)));
const predecessors = new Map<StylePacketGroup, Row[]>();
for (const group of previous.groups) {
  const bytes = fs.readFileSync(new URL("../../../" + group.path, import.meta.url));
  assert.equal(hash(bytes), group.sha256);
  const archived = JSON.parse(bytes.toString()) as Row[];
  // This is a historical declaration assembled exclusively from previously frozen records,
  // independently admitted by the original helper. No current candidate is projected.
  const rows = archived.map(
    (row, index) => group.records.find((entry) => entry.index === index)?.after ?? row,
  ) as Row[];
  assertChildStylePackets(group.name, rows);
  predecessors.set(group.name, rows);
}
function differences(a: unknown, b: unknown, prefix = ""): string[] {
  if (JSON.stringify(a) === JSON.stringify(b)) return [];
  if (Array.isArray(a) && Array.isArray(b)) {
    assert.equal(a.length, b.length);
    return a.flatMap((value, index) => differences(value, b[index], `${prefix}/${index}`));
  }
  if (
    a &&
    b &&
    typeof a === "object" &&
    typeof b === "object" &&
    !Array.isArray(a) &&
    !Array.isArray(b)
  ) {
    assert.deepEqual(Object.keys(a).sort(), Object.keys(b).sort());
    return Object.keys(a).flatMap((key) =>
      differences((a as Row)[key], (b as Row)[key], `${prefix}/${key}`),
    );
  }
  return [prefix];
}
function record(group: StylePacketGroup, index: number) {
  const predecessor = predecessors.get(group)?.[index];
  assert.ok(predecessor);
  const entry = vdomStyleContract.groups
    .find((g) => g.name === group)
    ?.records.find((row) => row.index === index);
  return { predecessor, entry };
}
function assertSuccessor(group: StylePacketGroup, index: number, actual: Row["output"]): void {
  const { predecessor, entry } = record(group, index);
  assert.ok(entry);
  assert.equal(hash(JSON.stringify(deterministic(predecessor))), entry.predecessorSha256);
  assert.deepEqual(
    actual,
    entry.after.output,
    `${group}:${index}: whole actual VDOM successor output`,
  );
  assert.equal(entry.after.family, "jsx");
  assert.equal(entry.after.mode, "Vdom");
  assert.equal(entry.after.ssr, false);
  assert.equal(entry.after.fold, false);
  assert.deepEqual(differences(deterministic(predecessor), entry.after), entry.changedPointers);
  assert.deepEqual(entry.changedPointers, [
    "/output/components/0/code",
    "/output/components/0/preamble",
    "/output/module",
  ]);
}
export function assertVdomChildStylePackets(
  group: StylePacketGroup,
  actual: readonly Row[],
): number {
  const expected = predecessors.get(group);
  assert.ok(expected);
  assert.equal(actual.length, expected.length);
  let count = 0;
  actual.forEach((row, index) => {
    const { predecessor, entry } = record(group, index);
    if (entry) {
      assert.deepEqual(
        deterministic(row),
        entry.after,
        `${group}:${index}: complete actual packet`,
      );
      assertSuccessor(group, index, row.output);
      count++;
    } else
      assert.deepEqual(
        deterministic(row),
        deterministic(predecessor),
        `${group}:${index}: unchanged whole previous packet`,
      );
  });
  assert.equal(count, group === "main" ? 12 : 0);
  return count + (group === "main" ? 30 : 0);
}
export function assertVdomChildPreservedStylePacket(
  group: StylePacketGroup,
  index: number,
  before: Row["output"],
  after: Row["output"],
): void {
  const { predecessor, entry } = record(group, index);
  if (!entry) {
    assertChildPreservedStylePacket(group, index, before, after);
    return;
  }
  assertChildPreservedStylePacket(group, index, before, predecessor.output);
  assertSuccessor(group, index, after);
}
