import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  assertChildStylePackets,
  assertChildPreservedStylePacket,
  childStyleSuccessorCount,
  contract,
} from "./support/jsx-child-style-successors.ts";
type Group = (typeof contract.groups)[number];
type Row = Parameters<typeof assertChildStylePackets>[1][number];
function fixture(group: Group): Row[] {
  const old = JSON.parse(fs.readFileSync(group.path, "utf8")) as Row[];
  for (const record of group.records) old[record.index] = structuredClone(record.after);
  return old;
}
test("exact thirty child code/map successors retain all 328 complete style input/output rows", () => {
  assert.equal(childStyleSuccessorCount, 30);
  for (const group of contract.groups)
    assert.equal(assertChildStylePackets(group.name, fixture(group)), group.records.length);
});
test("changed and unchanged rows reject foreign code, inputs, diagnostics and populations", () => {
  const group = contract.groups[0];
  for (const index of [0, 8, 172])
    for (const key of ["source", "output"] as const) {
      const rows = fixture(group);
      (rows[index] as Record<string, unknown>)[key] = "foreign";
      assert.throws(() => assertChildStylePackets("main", rows), assert.AssertionError);
    }
  const fewer = fixture(group);
  fewer.pop();
  assert.throws(() => assertChildStylePackets("main", fewer));
  for (const key of ["errors", "module", "components"]) {
    const rows = fixture(group);
    rows[8].output[key] = "foreign";
    assert.throws(() => assertChildStylePackets("main", rows), assert.AssertionError);
  }
  for (const group of contract.groups.slice(1)) {
    const rows = fixture(group);
    rows[0].output.foreign = "extra";
    assert.throws(() => assertChildStylePackets(group.name, rows), assert.AssertionError);
  }
});
test("composition still rejects a different original producer or altered new complete output", () => {
  const old = JSON.parse(
    fs.readFileSync(
      "tests/_fixtures/differential/compiler/vapor-literal-object-styles-10749/observed-head.json",
      "utf8",
    ),
  ) as Row[];
  for (const row of contract.groups[0].records) {
    assertChildPreservedStylePacket("main", row.index, old[row.index].output, row.after.output);
    assert.throws(() =>
      assertChildPreservedStylePacket("main", row.index, { foreign: true }, row.after.output),
    );
    assert.throws(() =>
      assertChildPreservedStylePacket("main", row.index, old[row.index].output, {
        ...row.after.output,
        foreign: true,
      }),
    );
  }
});
