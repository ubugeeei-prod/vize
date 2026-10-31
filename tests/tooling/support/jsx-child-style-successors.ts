/** Finite complete code/map successors; original style inputs and admissions stay immutable. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import {
  assertAuthoredStylePackets,
  assertPreservedStylePacket,
  type StylePacketGroup,
} from "./vapor-style-authored-transition.ts";
type Output = Record<string, unknown>;
type Row = Output & { output: Output };
type Entry = { index: number; after: Row; changedPointers: string[] };
type Group = {
  name: StylePacketGroup;
  path: string;
  sha256: string;
  population: number;
  records: Entry[];
};
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const bytes = fs.readFileSync(
  new URL(
    "../../_fixtures/differential/jsx/child-values-6890-style-successors.json",
    import.meta.url,
  ),
);
assert.equal(hash(bytes), "f04d2714a136ca7882a6709bc32ff0ec2f3b296c3a5783b4b5709c5265a75ab3");
export const contract = JSON.parse(bytes.toString()) as {
  schema: number;
  qualification: unknown;
  groups: Group[];
};
assert.equal(contract.schema, 1);
assert.deepEqual(contract.qualification, {
  runtime: false,
  nativeHandled: 0,
  wholeCorpusQualified: false,
  issue6890Complete: false,
});
assert.deepEqual(
  contract.groups.map((group) => [group.name, group.population]),
  [
    ["main", 180],
    ["attribute-case", 48],
    ["review", 48],
    ["priority", 48],
    ["sfc", 4],
  ],
);
const mainIndices = [
  8, 11, 20, 23, 32, 35, 44, 47, 56, 59, 68, 71, 80, 83, 92, 95, 104, 107, 116, 119, 128, 131, 140,
  143, 172, 173, 174, 175, 176, 177,
];
for (const group of contract.groups)
  assert.deepEqual(
    group.records.map((row) => row.index),
    group.name === "main" ? mainIndices : [],
  );
const archived = new Map<StylePacketGroup, Row[]>();
for (const group of contract.groups) {
  const bytes = fs.readFileSync(new URL("../../../" + group.path, import.meta.url));
  assert.equal(hash(bytes), group.sha256, "complete historical style packets are immutable");
  const rows = JSON.parse(bytes.toString()) as Row[];
  assert.equal(rows.length, group.population);
  archived.set(group.name, rows);
}
const metrics = new Set(["arenaBytes", "expressionReparses", "counters", "phases"]);
const deterministic = (row: Row) =>
  Object.fromEntries(Object.entries(row).filter(([key]) => !metrics.has(key)));
const input = (row: Row) =>
  Object.fromEntries(Object.entries(deterministic(row)).filter(([key]) => key !== "output"));
function differences(before: unknown, after: unknown, prefix = ""): string[] {
  if (JSON.stringify(before) === JSON.stringify(after)) return [];
  if (Array.isArray(before) && Array.isArray(after)) {
    assert.equal(before.length, after.length);
    return before.flatMap((value, index) => differences(value, after[index], `${prefix}/${index}`));
  }
  if (
    before &&
    after &&
    typeof before === "object" &&
    typeof after === "object" &&
    !Array.isArray(before) &&
    !Array.isArray(after)
  ) {
    assert.deepEqual(Object.keys(before).sort(), Object.keys(after).sort());
    const a = before as Output,
      b = after as Output;
    return Object.keys(a).flatMap((key) => differences(a[key], b[key], `${prefix}/${key}`));
  }
  return [prefix];
}
function registration(group: StylePacketGroup, index: number) {
  const archive = contract.groups.find((row) => row.name === group);
  assert(archive);
  const previous = archived.get(group)?.[index];
  assert(previous, `${group}:${index}: original row`);
  return { previous, entry: archive.records.find((row) => row.index === index) };
}
function assertSuccessor(group: StylePacketGroup, index: number, actual: Output): void {
  const { previous, entry } = registration(group, index);
  assert(entry);
  assert.deepEqual(
    actual,
    entry.after.output,
    `${group}:${index}: complete authenticated child successor`,
  );
  const changed = differences(previous.output, actual);
  assert.deepEqual(changed, entry.changedPointers, "only exact observed code/map pointers change");
  assert(
    changed.every((pointer) =>
      /^\/(?:components\/\d+\/code|module|code|(?:components\/\d+\/)?(?:moduleMap|map)\/mappings)$/.test(
        pointer,
      ),
    ),
  );
}
/** Compare actual packets directly; no field is removed or replaced in the producer output. */
export function assertChildStylePackets(group: StylePacketGroup, actual: readonly Row[]): number {
  const record = contract.groups.find((row) => row.name === group);
  assert(record);
  if (!record.records.length) {
    assertAuthoredStylePackets(group, actual);
    return 0;
  }
  assert.equal(
    actual.length,
    record.population,
    "the complete original style population is retained",
  );
  let validated = 0;
  actual.forEach((row, index) => {
    const { previous, entry } = registration(group, index);
    assert.deepEqual(input(row), input(previous), `${group}:${index}: all original input fields`);
    if (entry) {
      assert.deepEqual(
        deterministic(row),
        entry.after,
        `${group}:${index}: complete current packet`,
      );
      assertSuccessor(group, index, row.output);
      validated++;
    } else
      assert.deepEqual(
        deterministic(row),
        deterministic(previous),
        `${group}:${index}: unchanged original whole packet`,
      );
  });
  assert.equal(validated, record.records.length);
  return validated;
}
/** Validate the real old producer against its original admission, then this exact new output. */
export function assertChildPreservedStylePacket(
  group: StylePacketGroup,
  index: number,
  before: Output,
  after: Output,
): void {
  const { previous, entry } = registration(group, index);
  if (!entry) {
    assertPreservedStylePacket(group, index, before, after);
    return;
  }
  assertPreservedStylePacket(group, index, before, previous.output);
  assertSuccessor(group, index, after);
}
export const childStyleSuccessorCount = mainIndices.length;
