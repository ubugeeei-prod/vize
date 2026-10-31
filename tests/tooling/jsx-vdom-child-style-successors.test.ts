import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { contract as previous } from "./support/jsx-child-style-successors.ts";
import {
  vdomStyleContract,
  assertVdomChildStylePackets,
  assertVdomChildPreservedStylePacket,
} from "./support/jsx-vdom-child-style-successors.ts";
const predecessor = (group: (typeof previous.groups)[number]) => {
  const archived = JSON.parse(fs.readFileSync(group.path, "utf8"));
  return archived.map((row: unknown, index: number) =>
    structuredClone(group.records.find((entry) => entry.index === index)?.after ?? row),
  );
};
const current = (group: (typeof previous.groups)[number]) => {
  const rows = predecessor(group);
  for (const entry of vdomStyleContract.groups.find((g) => g.name === group.name)!.records)
    rows[entry.index] = structuredClone(entry.after);
  return rows;
};
test("all 328 current packets retain 316 complete predecessors and exactly twelve VDOM code/preamble/module successors", () => {
  let population = 0,
    count = 0;
  for (const group of previous.groups) {
    const rows = current(group);
    population += rows.length;
    count += assertVdomChildStylePackets(group.name, rows);
  }
  assert.equal(population, 328);
  assert.equal(count, 42);
});
test("whole-output admission refuses source, diagnostic, code and population mutations", () => {
  const group = previous.groups.find((g) => g.name === "main")!;
  for (const index of [0, 8, 9, 141, 179]) {
    for (const field of ["source", "output"]) {
      const rows = current(group);
      if (field === "source") rows[index].source += "changed";
      else rows[index].output.extra = "not admitted";
      assert.throws(() => assertVdomChildStylePackets("main", rows));
    }
  }
  for (const entry of vdomStyleContract.groups[0]!.records) {
    for (const field of ["code", "preamble"]) {
      const rows = current(group);
      rows[entry.index].output.components[0][field] += "changed";
      assert.throws(() => assertVdomChildStylePackets("main", rows));
    }
    const rows = current(group);
    rows[entry.index].output.module += "changed";
    assert.throws(() => assertVdomChildStylePackets("main", rows));
  }
  assert.throws(() => assertVdomChildStylePackets("main", current(group).slice(1)));
});
test("separate historical producer validation retains the original before side", () => {
  const group = previous.groups[0]!;
  const prior = predecessor(group);
  const rows = current(group);
  for (const entry of vdomStyleContract.groups[0]!.records) {
    assertVdomChildPreservedStylePacket(
      "main",
      entry.index,
      prior[entry.index].output,
      rows[entry.index].output,
    );
    assert.throws(() =>
      assertVdomChildPreservedStylePacket(
        "main",
        entry.index,
        { ...prior[entry.index].output, extra: "bad producer" },
        rows[entry.index].output,
      ),
    );
  }
});
